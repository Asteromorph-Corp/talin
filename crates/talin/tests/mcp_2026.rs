//! Tests MCP tool support against a local streamable-HTTP server speaking the 2026 protocol
//! revision, including task creation, polling and cancellation.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, CancelTaskParams, ContentBlock,
    CreateTaskResult, DetailedTask, DiscoverResult, GetTaskParams, GetTaskResult, JsonObject,
    ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerConfig,
    Task, TaskPayload, TaskStatus,
};
use rmcp::service::RequestContext;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde_json::json;
use talin::agent::Agent;
use talin::llm::GenericChat;
use talin::mcp::McpServer;
use talin::tools::ToolResolver;

#[derive(Default)]
struct TaskServerState {
    discovered: AtomicUsize,
    saw_2026_context: AtomicBool,
    saw_tasks_capability: AtomicBool,
    saw_task_request_context: AtomicBool,
    polled: AtomicUsize,
    cancelled: AtomicUsize,
    completes: bool,
}

#[derive(Clone)]
struct TaskMcpServer {
    state: Arc<TaskServerState>,
}

impl ServerHandler for TaskMcpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_tasks()
                .build(),
        )
    }

    async fn discover(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<DiscoverResult, ErrorData> {
        self.state.discovered.fetch_add(1, Ordering::Relaxed);
        Ok(DiscoverResult::from_server_info(
            ProtocolVersion::KNOWN_VERSIONS.to_vec(),
            self.get_info(),
        ))
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let schema: JsonObject = serde_json::from_value(json!({ "type": "object" })).unwrap();
        Ok(ListToolsResult::with_all_items(vec![
            rmcp::model::Tool::new(
                "deferred",
                "Completes through an MCP task.",
                Arc::new(schema),
            ),
        ]))
    }

    async fn call_tool(
        &self,
        _request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        self.state.saw_2026_context.store(
            context.protocol_version() == Some(ProtocolVersion::V_2026_07_28),
            Ordering::Relaxed,
        );
        self.state.saw_tasks_capability.store(
            context
                .client_capabilities()
                .is_some_and(|capabilities| capabilities.supports_tasks()),
            Ordering::Relaxed,
        );
        Ok(CallToolResponse::Task(CreateTaskResult::new(
            Task::new(
                "task-1",
                TaskStatus::Working,
                "2026-08-24T00:00:00Z",
                "2026-08-24T00:00:00Z",
            )
            .with_poll_interval_ms(1),
        )))
    }

    async fn get_task(
        &self,
        request: GetTaskParams,
        context: RequestContext<RoleServer>,
    ) -> Result<GetTaskResult, ErrorData> {
        self.state.polled.fetch_add(1, Ordering::Relaxed);
        self.state.saw_task_request_context.store(
            context.protocol_version() == Some(ProtocolVersion::V_2026_07_28)
                && context
                    .client_capabilities()
                    .is_some_and(|capabilities| capabilities.supports_tasks()),
            Ordering::Relaxed,
        );
        if !self.state.completes {
            return Ok(GetTaskResult::new(DetailedTask::new(
                Task::new(
                    request.task_id,
                    TaskStatus::Working,
                    "2026-08-24T00:00:00Z",
                    "2026-08-24T00:00:01Z",
                )
                .with_poll_interval_ms(1),
                TaskPayload::Working,
            )));
        }
        let result = CallToolResult::success(vec![ContentBlock::text("deferred complete")]);
        let result = serde_json::to_value(result)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        Ok(GetTaskResult::new(DetailedTask::new(
            Task::new(
                request.task_id,
                TaskStatus::Completed,
                "2026-08-24T00:00:00Z",
                "2026-08-24T00:00:01Z",
            ),
            TaskPayload::Completed { result },
        )))
    }

    async fn cancel_task(
        &self,
        _request: CancelTaskParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        self.state.cancelled.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

async fn spawn_task_server(completes: bool) -> (String, Arc<TaskServerState>) {
    let state = Arc::new(TaskServerState {
        completes,
        ..TaskServerState::default()
    });
    let service = StreamableHttpService::new(
        {
            let state = state.clone();
            move || {
                Ok(TaskMcpServer {
                    state: state.clone(),
                })
            }
        },
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );
    let app = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}/mcp"), state)
}

#[tokio::test]
async fn sync_negotiates_2026_and_polls_proscenium_shaped_tasks() {
    let (url, state) = spawn_task_server(true).await;
    let llm = GenericChat::builder().with_model("unused").build();
    let agent: Agent<_> = Agent::builder()
        .mcp_servers([McpServer::http(url, "task-server")])
        .build(llm);
    let mut session = agent.session();

    session.sync_inventory().await.unwrap();
    let output = session
        .tool_resolver()
        .resolve_tool("deferred")
        .unwrap()
        .call(json!({}))
        .await
        .unwrap()
        .to_string();

    assert!(output.contains("deferred complete"));
    assert!(state.discovered.load(Ordering::Relaxed) > 0);
    assert!(state.saw_2026_context.load(Ordering::Relaxed));
    assert!(state.saw_tasks_capability.load(Ordering::Relaxed));
    assert!(state.saw_task_request_context.load(Ordering::Relaxed));
    assert!(state.polled.load(Ordering::Relaxed) > 0);
    assert_eq!(state.cancelled.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn dropping_a_task_call_requests_cooperative_cancellation() {
    let (url, state) = spawn_task_server(false).await;
    let llm = GenericChat::builder().with_model("unused").build();
    let agent: Agent<_> = Agent::builder()
        .mcp_servers([McpServer::http(url, "task-server")])
        .build(llm);
    let mut session = agent.session();

    session.sync_inventory().await.unwrap();
    let deferred = session.tool_resolver().resolve_tool("deferred").unwrap();
    let timed_out =
        tokio::time::timeout(Duration::from_millis(100), deferred.call(json!({}))).await;
    assert!(timed_out.is_err());

    tokio::time::timeout(Duration::from_secs(1), async {
        while state.cancelled.load(Ordering::Relaxed) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(state.polled.load(Ordering::Relaxed) > 0);
    assert_eq!(state.cancelled.load(Ordering::Relaxed), 1);
}
