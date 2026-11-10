use crate::agent::{Agent, AgentOptions};
use crate::chain::{InputCtor, OutputCtor};
use crate::llm::Llm;
use crate::mcp::{McpServer, McpServerSet};
use crate::prompt_template;
use crate::skill::SkillSet;
use crate::strategy::Strategy;
use crate::template::{MessageTemplate, TemplateContent, TemplateRole};
use crate::tools::Tool;

const DEFAULT_USER_MESSAGE: &str = r#"{{input}}"#;

/// A builder for constructing an [`Agent`].
///
/// Used to set provider-agnostic configurations.
/// To set provider specific configurations such as default [`strategy`](Strategy) or built-in
/// tools, call [`specialize::<L>`](Self::specialize) to obtain a [`SpecializedAgentBuilder`].
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type.
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type.
pub struct AgentBuilder<I: InputCtor, O: OutputCtor> {
    /// The id of the agent
    id: Option<String>,
    /// The system prompt to be used by the agent. The agent has no system prompt when unset.
    system_message: Option<TemplateContent>,
    /// The initial user prompt to be used by the agent.
    user_message: Option<TemplateContent>,
    /// The tools to be used by the agent.
    tools: Option<Vec<Tool>>,
    /// The skills the model can load on demand.
    skills: Option<SkillSet>,
    /// The MCP servers to be used by the agent.
    mcp_servers: Option<McpServerSet>,
    // /// The toolboxes containing additional tools for the agent.
    // toolboxes: Option<Vec<Arc<dyn Toolbox>>>,
    /// The options for the agent.
    options: Option<AgentOptions>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I: InputCtor, O: OutputCtor> AgentBuilder<I, O> {
    /// Constructs a new [`AgentBuilder`].
    ///
    /// This is the same as calling [`Agent::builder()`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: None,
            tools: None,
            skills: None,
            mcp_servers: None,
            // toolboxes: None,
            system_message: None,
            user_message: None,
            options: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Sets the id.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the system message.
    pub fn system_message(mut self, system_message: impl Into<TemplateContent>) -> Self {
        self.system_message = Some(system_message.into());
        self
    }

    /// Sets the initial user message.
    pub fn user_message(mut self, user_message: impl Into<TemplateContent>) -> Self {
        self.user_message = Some(user_message.into());
        self
    }

    /// Adds tools.
    pub fn tools(mut self, tools: impl IntoIterator<Item = Tool>) -> Self {
        self.tools = Some(tools.into_iter().collect());
        self
    }

    /// Sets the MCP servers.
    pub fn mcp_servers(mut self, servers: impl IntoIterator<Item = impl Into<McpServer>>) -> Self {
        self.mcp_servers = Some(servers.into_iter().collect());
        self
    }

    /// Sets the skills the model can load on demand.
    ///
    /// The model sees each skill's name and description in the system prompt and loads the full
    /// instructions on demand. The session's skills can be edited afterwards through
    /// [`AgentSession::skills`](crate::agent::AgentSession::skills).
    pub fn skills(mut self, skills: SkillSet) -> Self {
        self.skills = Some(skills);
        self
    }

    // /// Adds toolboxes.
    // pub fn toolboxes(mut self, toolboxes: Vec<Arc<dyn Toolbox>>) -> Self {
    //     self.toolboxes = Some(toolboxes);
    //     self
    // }

    /// Sets the agent options.
    pub fn options(mut self, options: AgentOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Locks in the LLM type `L`, yielding a [`SpecializedAgentBuilder`] provider specific
    pub fn specialize<L: Llm>(self) -> SpecializedAgentBuilder<L, I, O> {
        SpecializedAgentBuilder {
            id: self.id,
            system_message: self.system_message,
            user_message: self.user_message,
            tools: self.tools,
            builtin_tools: Vec::new(),
            skills: self.skills,
            mcp_servers: self.mcp_servers,
            strategy: (),
            options: self.options,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Returns an [`Agent`] that uses this [`AgentBuilder`] configuration.
    pub fn build<L: Llm>(self, llm: L) -> Agent<L, I, O> {
        self.specialize::<L>().build(llm)
    }
}

impl<I: InputCtor, O: OutputCtor> Default for AgentBuilder<I, O> {
    fn default() -> Self {
        Self::new()
    }
}

/// An [`AgentBuilder`] specialized to a concrete LLM type `L`.
///
/// Used to set provider specific configurations such as default [`strategy`](Strategy) or built-in
/// tools
///
/// # Type Parameters
/// - `L`: The LLM driver, implementing [`Llm`].
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type.
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type.
/// - `S`: The accumulated default [`Strategy`], composed on every turn (defaults to `()`).
pub struct SpecializedAgentBuilder<L, I, O, S = ()>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
{
    id: Option<String>,
    system_message: Option<TemplateContent>,
    user_message: Option<TemplateContent>,
    tools: Option<Vec<Tool>>,
    builtin_tools: Vec<L::BuiltinTool>,
    skills: Option<SkillSet>,
    mcp_servers: Option<McpServerSet>,
    strategy: S,
    options: Option<AgentOptions>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<L, I, O, S> SpecializedAgentBuilder<L, I, O, S>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
{
    /// Sets the id.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the system message.
    pub fn system_message(mut self, system_message: impl Into<TemplateContent>) -> Self {
        self.system_message = Some(system_message.into());
        self
    }

    /// Sets the initial user message.
    pub fn user_message(mut self, user_message: impl Into<TemplateContent>) -> Self {
        self.user_message = Some(user_message.into());
        self
    }

    /// Adds tools.
    pub fn tools(mut self, tools: impl IntoIterator<Item = Tool>) -> Self {
        self.tools = Some(tools.into_iter().collect());
        self
    }

    /// Sets the provider built-in tools.
    pub fn builtin_tools(mut self, builtin_tools: Vec<L::BuiltinTool>) -> Self {
        self.builtin_tools = builtin_tools;
        self
    }

    /// Sets the skills the model can load on demand.
    pub fn skills(mut self, skills: SkillSet) -> Self {
        self.skills = Some(skills);
        self
    }

    /// Sets the MCP servers.
    pub fn mcp_servers(mut self, servers: impl IntoIterator<Item = impl Into<McpServer>>) -> Self {
        self.mcp_servers = Some(servers.into_iter().collect());
        self
    }

    /// Sets the agent options.
    pub fn options(mut self, options: AgentOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Composes `strategy` into the agent's default strategy, run on every turn.
    pub fn strategy<S2>(self, strategy: S2) -> SpecializedAgentBuilder<L, I, O, S2>
    where
        S2: Strategy<L, O> + Clone,
    {
        SpecializedAgentBuilder {
            id: self.id,
            system_message: self.system_message,
            user_message: self.user_message,
            tools: self.tools,
            builtin_tools: self.builtin_tools,
            skills: self.skills,
            mcp_servers: self.mcp_servers,
            strategy,
            options: self.options,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Returns an [`Agent`] that uses this configuration.
    pub fn build(self, llm: L) -> Agent<L, I, O, S> {
        let id = self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let system = self
            .system_message
            .map(|system| system.role(TemplateRole::System));
        let user = self
            .user_message
            .unwrap_or_else(|| MessageTemplate::jinja2(DEFAULT_USER_MESSAGE))
            .role(TemplateRole::User);

        let prompt = match system {
            Some(system) => prompt_template![system, user],
            None => prompt_template![user],
        };

        let tools = self.tools.into_iter().flatten().collect::<Vec<_>>();
        let skills = self.skills.unwrap_or_default();
        let mcp_servers = self.mcp_servers.unwrap_or_default();
        let options = self.options.unwrap_or_default();

        Agent {
            id,
            prompt,
            llm,
            tools: Tool::build_index(tools),
            builtin_tools: self.builtin_tools,
            mcp_servers,
            skills,
            strategy: self.strategy,
            options,
            _phantom: std::marker::PhantomData,
        }
    }
}
