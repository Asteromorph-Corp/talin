//! A strategy that gives the agent a notebook.
//!
//! The agent writes notes through a tool, and the strategy hands the notes back next to the
//! agent's answer, as the strategy's output.

use std::convert::Infallible;
use std::error::Error;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use talin::chain::OutputCtor;
use talin::llm::Llm;
use talin::schemas::{ToolResult, ToolResultMessage};
use talin::strategy::Strategy;
use talin::tools::{Function, Tool, ToolResolver};
use schemars::JsonSchema;
use serde::Deserialize;

/// What the strategy returns once the run is over.
#[derive(Debug)]
pub struct Notebook {
    /// The notes the agent wrote, in order.
    pub notes: Vec<String>,
    /// Every tool call made during the run, and whether it succeeded.
    pub calls: Vec<(String, bool)>,
}

#[derive(Deserialize, JsonSchema)]
pub struct JotInput {
    /// One key fact from the text, in a single sentence.
    pub note: String,
}

/// The tool the agent writes notes with. It shares the notes with the strategy.
struct Jot {
    notes: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl Function for Jot {
    type Input = JotInput;
    type Output = String;

    fn name(&self) -> String {
        "jot".into()
    }

    fn description(&self) -> String {
        "Writes one note in the notebook.".into()
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        if input.note.trim().is_empty() {
            return Err("a note cannot be empty".into());
        }
        let mut notes = self.notes.lock().unwrap();
        notes.push(input.note);
        Ok(format!("Noted. The notebook has {} notes.", notes.len()))
    }
}

pub struct NotebookStrategy {
    notes: Arc<Mutex<Vec<String>>>,
    tools: Vec<Tool>,
    calls: Vec<(String, bool)>,
}

impl NotebookStrategy {
    pub fn new() -> Self {
        let notes = Arc::new(Mutex::new(Vec::new()));
        let tools = vec![Tool::from(Jot {
            notes: notes.clone(),
        })];
        Self {
            notes,
            tools,
            calls: Vec::new(),
        }
    }
}

// The strategy works with any model (`L`) and any agent output (`O`), because it only touches the
// tools and the tool results.
#[async_trait]
impl<L: Llm, O: OutputCtor> Strategy<L, O> for NotebookStrategy {
    type Output = Notebook;
    type Error = Infallible;

    // Injection: the tools that exist for this run only. A tuple of strategies exposes the tools
    // of all of its members.
    fn additional_tools<'a>(&'a self) -> impl ToolResolver<'a> + 'a {
        self.tools.as_slice()
    }

    // Observation: called with every tool result, and may rewrite it. This one only keeps a record
    // and passes the result on.
    async fn process_tool_result(
        &mut self,
        message: ToolResultMessage,
    ) -> Result<ToolResult, Self::Error> {
        self.calls.push((message.name, message.result.is_ok()));
        Ok(message.result)
    }

    // The output: consumes the strategy once the run is over.
    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(Notebook {
            notes: self.notes.lock().unwrap().clone(),
            calls: self.calls,
        })
    }
}
