use std::time::Duration;

use rmcp::model::{CallToolResult, CancelTaskParams, GetTaskParams, Task, TaskPayload};
use rmcp::{Peer, RoleClient};

use crate::mcp::{McpError, McpService};
use crate::tools::ToolError;

const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(1);

pub(super) async fn poll_task(
    service: &McpService,
    task: Task,
) -> Result<CallToolResult, ToolError> {
    let task_id = task.task_id;
    let mut cancellation = TaskCancellation::new(service.peer().clone(), task_id.clone());
    let mut poll_interval = task_poll_interval(task.poll_interval_ms);

    loop {
        tokio::time::sleep(poll_interval).await;
        let result = service
            .get_task(GetTaskParams::new(task_id.clone()))
            .await
            .map_err(McpError::from)?;
        poll_interval = task_poll_interval(result.task.task.poll_interval_ms);
        match result.task.payload {
            TaskPayload::Working => {}
            TaskPayload::Completed { result } => {
                cancellation.disarm();
                let result =
                    serde_json::from_value::<CallToolResult>(serde_json::Value::Object(result))
                        .map_err(McpError::from)?;
                return Ok(result);
            }
            TaskPayload::Failed { error } => {
                cancellation.disarm();
                let error = serde_json::Value::Object(error);
                return Err(ToolError::Execution(error.to_string().into()));
            }
            TaskPayload::Cancelled => {
                cancellation.disarm();
                return Err(ToolError::Execution("tool call cancelled".into()));
            }
            TaskPayload::InputRequired { .. } => return Err(McpError::InputRequired.into()),
            _ => return Err(McpError::invalid_content("unsupported `tasks/poll` response").into()),
        }
    }
}

struct TaskCancellation {
    peer: Peer<RoleClient>,
    task_id: Option<String>,
}

impl TaskCancellation {
    fn new(peer: Peer<RoleClient>, task_id: String) -> Self {
        Self {
            peer,
            task_id: Some(task_id),
        }
    }

    fn disarm(&mut self) {
        self.task_id = None;
    }
}

impl Drop for TaskCancellation {
    fn drop(&mut self) {
        let Some(task_id) = self.task_id.take() else {
            return;
        };
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            tracing::warn!(task_id, "could not cancel MCP task outside a Tokio runtime");
            return;
        };
        let peer = self.peer.clone();
        drop(runtime.spawn(async move {
            if let Err(error) = peer
                .cancel_task(CancelTaskParams::new(task_id.clone()))
                .await
            {
                tracing::warn!(task_id, %error, "failed to cancel MCP task");
            }
        }));
    }
}

fn task_poll_interval(poll_interval_ms: Option<u64>) -> Duration {
    poll_interval_ms
        .map(|milliseconds| Duration::from_millis(milliseconds.max(1)))
        .unwrap_or(DEFAULT_POLL_INTERVAL)
}
