use std::collections::hash_map::Entry as HashMapEntry;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use futures::{StreamExt, TryStreamExt};

use crate::agent::AgentError;
use crate::agent::error::McpSyncError;
use crate::mcp::{
    McpClient, McpFunctionTool, McpRouting, McpServer, McpServerSet, McpService, ServiceKey,
};
use crate::skill::SkillSet;
use crate::skill::wire::skill_tools;
use crate::tools::{FunctionTool, Tool, ToolRef, ToolResolver, normalize_tool_name};

/// A collection of resources managed for a session or a turn.
#[derive(Default)]
pub(super) struct Inventory {
    /// The list of skills available to the agent.
    skill_catalog: SkillCatalog,
    /// The clients for the MCP servers handled locally.
    mcp_clients: McpClients,
}

/// A view of an [`Inventory`] with the entries that another inventory replaces hidden.
pub(super) struct MaskedInventory<'a> {
    inventory: &'a Inventory,
    /// The names of the MCP servers whose clients are hidden.
    server_masks: HashSet<&'a str>,
    /// Whether the skill loader tools are hidden.
    skill_mask: bool,
}

/// The list of skills available to the agent.
#[derive(Default)]
pub(crate) struct SkillCatalog {
    skills: Arc<SkillSet>,
    tools: HashMap<String, Tool>,
}

/// The local MCP clients for a group of servers, and the names of the servers handled by the
/// provider instead.
#[derive(Default)]
struct McpClients {
    /// The open connections, shared across clients of the same server.
    services: HashMap<ServiceKey, Arc<McpService>>,
    clients: HashMap<String, McpClient>,
    /// The names of the servers handled by the provider, which have no client.
    remote: HashSet<String>,
}

impl Inventory {
    /// Syncs the skill loader tools and the local MCP clients with `skills` and `mcp_servers`.
    ///
    /// Entries that `parent` already holds are skipped, so they are neither rebuilt nor
    /// reconnected, and are dropped from this inventory if it held them before. For skills, that is
    /// when `parent` was built from the same skills. For MCP servers, that is when `parent` has a
    /// client for a server with the same name, connection and allow-list.
    pub(super) async fn sync(
        &mut self,
        mcp_servers: &McpServerSet,
        skills: &SkillSet,
        native_mcp: bool,
        parent: Option<&Inventory>,
    ) -> Result<(), AgentError> {
        if parent.is_some_and(|parent| parent.skill_catalog.reflects(skills)) {
            self.skill_catalog.sync(&SkillSet::default());
        } else {
            self.skill_catalog.sync(skills);
        }

        self.mcp_clients
            .sync(
                mcp_servers.route(native_mcp),
                parent.map(|parent| &parent.mcp_clients),
            )
            .await?;

        Ok(())
    }
}

impl<'a> ToolResolver<'a> for &'a Inventory {
    fn list_tools(&self) -> Vec<ToolRef<'a>> {
        (&self.mcp_clients, self.skill_catalog.tools()).list_tools()
    }

    fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn FunctionTool> {
        (&self.mcp_clients, self.skill_catalog.tools()).resolve_tool(tool_name)
    }
}

impl Inventory {
    /// This inventory without the entries that `child` replaces.
    pub(super) fn masked_by<'a>(&'a self, child: &'a Inventory) -> MaskedInventory<'a> {
        MaskedInventory {
            inventory: self,
            server_masks: child.mcp_clients.claimed_names().collect(),
            skill_mask: !child.skill_catalog.tools().is_empty(),
        }
    }
}

impl<'a> MaskedInventory<'a> {
    fn functions(&self) -> impl Iterator<Item = &'a McpFunctionTool> + '_ {
        self.inventory
            .mcp_clients
            .clients
            .iter()
            .filter(|(name, _)| !self.server_masks.contains(name.as_str()))
            .flat_map(|(_, client)| client.functions.values())
    }

    fn skill_tools(&self) -> Option<&'a HashMap<String, Tool>> {
        (!self.skill_mask).then(|| self.inventory.skill_catalog.tools())
    }
}

impl<'a> ToolResolver<'a> for MaskedInventory<'a> {
    fn list_tools(&self) -> Vec<ToolRef<'a>> {
        let mcp = list_functions(self.functions());
        mcp.into_iter()
            .chain(self.skill_tools().list_tools())
            .collect()
    }

    fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn FunctionTool> {
        find_function(self.functions(), tool_name)
            .or_else(|| self.skill_tools().resolve_tool(tool_name))
    }
}

impl McpClients {
    /// Whether there is a client for the server that reflects its configuration.
    fn reflects(&self, server: &McpServer) -> bool {
        self.clients
            .get(server.name())
            .is_some_and(|client| client.reflects(server))
    }

    /// The names of the servers this handles, whether with a client or by the provider.
    ///
    /// A group that holds a name replaces the entry of the same name in its parent.
    fn claimed_names(&self) -> impl Iterator<Item = &str> {
        self.clients.keys().chain(&self.remote).map(String::as_str)
    }

    /// Syncs the clients so that they match the local servers of `routing`, except for servers
    /// `parent` already reflects, and records the names of its remote servers.
    ///
    /// Servers `parent` has a matching client for are skipped and dropped from `self`. Services
    /// are reused from `self`, then from `parent`, and connected otherwise. Servers without a
    /// client get one over their service, clients whose allow-list changed are re-synced over
    /// their existing MCP session, and clients whose server is no longer in `local` are dropped.
    async fn sync(
        &mut self,
        routing: McpRouting,
        parent: Option<&McpClients>,
    ) -> Result<(), McpSyncError> {
        self.remote = routing
            .remote
            .iter()
            .map(|server| server.name().to_owned())
            .collect();

        let local = routing
            .local
            .into_iter()
            .filter(|server| !parent.is_some_and(|parent| parent.reflects(server)))
            .collect::<Vec<_>>();

        // Drop outdated clients.
        self.clients.retain(|name, client| {
            local
                .iter()
                .find(|server| server.name() == name)
                .is_some_and(|server| client.key == server.service_key())
        });

        // Reuse the services of `self`, then those of `parent`, and connect the rest.
        let mut to_connect = HashMap::new();
        for server in &local {
            let key = server.service_key();
            if self.services.contains_key(&key) {
                continue;
            }
            match parent.and_then(|parent| parent.services.get(&key)) {
                Some(service) => {
                    self.services.insert(key, Arc::clone(service));
                }
                None => {
                    to_connect.entry(key).or_insert_with(|| server.clone());
                }
            }
        }
        let connected = futures::stream::iter(to_connect)
            .map(|(key, mcp)| async move {
                match McpClient::connect(&mcp).await {
                    Ok(service) => Ok((key, service)),
                    Err(e) => Err(McpSyncError::new(mcp.name, e)),
                }
            })
            .buffer_unordered(8)
            .try_collect::<Vec<_>>()
            .await?;
        self.services.extend(connected);
        let in_use = local
            .iter()
            .map(|server| server.service_key())
            .collect::<HashSet<_>>();
        self.services.retain(|key, _| in_use.contains(key));

        let local = local
            .iter()
            .map(|mcp| (mcp.name(), mcp))
            .collect::<HashMap<_, _>>();

        // Initialize uninitialized servers
        let uninitialized = local
            .iter()
            .filter(|(name, _)| !self.clients.contains_key(**name))
            .map(|(_, config)| {
                (
                    self.services[&config.service_key()].clone(),
                    (*config).clone(),
                )
            })
            .collect::<Vec<_>>();
        let initialized = futures::stream::iter(uninitialized)
            .map(|(service, mcp)| async move {
                match McpClient::from_service(service, &mcp).await {
                    Ok(client) => Ok((mcp.name, client)),
                    Err(e) => Err(McpSyncError::new(mcp.name, e)),
                }
            })
            .buffer_unordered(8)
            .try_collect::<Vec<_>>()
            .await?;
        self.clients.extend(initialized);

        // Re-sync clients whose allow-list changed, reusing each existing MCP session. Ownership is
        // moved out of the map so the syncs can run concurrently without aliasing `clients`.
        let stale = local
            .iter()
            .filter_map(|(name, config)| {
                let name = (*name).to_owned();
                let (name, client) = match self.clients.entry(name) {
                    HashMapEntry::Occupied(e) if !e.get().reflects_allow_list(config) => {
                        e.remove_entry()
                    }
                    _ => return None,
                };
                Some((name, client, (*config).clone()))
            })
            .collect::<Vec<_>>();
        let synced = futures::stream::iter(stale)
            .map(|(name, mut client, config)| async move {
                match client.sync(&config).await {
                    Ok(_) => Ok((name, client)),
                    Err(e) => Err(McpSyncError::new(name, e)),
                }
            })
            .buffer_unordered(8)
            .try_collect::<Vec<_>>()
            .await?;
        self.clients.extend(synced);

        Ok(())
    }

    fn functions(&self) -> impl Iterator<Item = &McpFunctionTool> {
        self.clients
            .values()
            .flat_map(|client| client.functions.values())
    }
}

impl<'a> ToolResolver<'a> for &'a McpClients {
    fn list_tools(&self) -> Vec<ToolRef<'a>> {
        list_functions(self.functions())
    }

    fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn FunctionTool> {
        find_function(self.functions(), tool_name)
    }
}

impl SkillCatalog {
    /// Rebuilds the tools if `skills` differs from the snapshot they were built from.
    pub(crate) fn sync(&mut self, skills: &SkillSet) {
        if self.reflects(skills) {
            return;
        }
        self.skills = Arc::new(skills.clone());
        self.tools = skill_tools(&self.skills);
    }

    /// Whether the tools were built from `skills`.
    fn reflects(&self, skills: &SkillSet) -> bool {
        *self.skills == *skills
    }

    /// The tools, keyed by normalized name.
    pub(crate) fn tools(&self) -> &HashMap<String, Tool> {
        &self.tools
    }
}

fn list_functions<'a>(functions: impl Iterator<Item = &'a McpFunctionTool>) -> Vec<ToolRef<'a>> {
    functions.map(|tool| ToolRef::Function(tool)).collect()
}

fn find_function<'a>(
    mut functions: impl Iterator<Item = &'a McpFunctionTool>,
    tool_name: &str,
) -> Option<&'a dyn FunctionTool> {
    let normalized = normalize_tool_name(tool_name);
    functions.find_map(|entry| {
        let entry: &dyn FunctionTool = entry;
        (entry.name() == normalized).then_some(entry)
    })
}

#[cfg(test)]
mod tests {
    use crate::agent::inventory::Inventory;
    use crate::mcp::test_server::{local_server, spawn_mock_server};
    use crate::mcp::{McpServer, McpServerSet};
    use crate::skill::{MemorySkill, SkillSet};

    /// A set of in-memory skills with the given names.
    ///
    /// Skill sets compare by identity, so two sets from separate calls are never equal. Build a
    /// set once and reuse it where a test needs equal sets.
    fn skills(names: &[&str]) -> SkillSet {
        let mut skills = SkillSet::new();
        for name in names {
            let markdown = format!("---\nname: {name}\ndescription: d\n---\n");
            skills.insert(MemorySkill::from_markdown(&markdown).unwrap());
        }
        skills
    }

    /// A set with the mock server as its only server.
    fn servers(url: &str, allowed_tools: Option<&[&str]>) -> McpServerSet {
        McpServerSet::from_iter([local_server(url, allowed_tools)])
    }

    /// Syncs a new inventory, which is the parent's child if there is one.
    async fn synced(
        servers: &McpServerSet,
        skills: &SkillSet,
        parent: Option<&Inventory>,
    ) -> Inventory {
        let mut inventory = Inventory::default();
        inventory.sync(servers, skills, true, parent).await.unwrap();
        inventory
    }

    fn client_names(inventory: &Inventory) -> Vec<&str> {
        let mut names = inventory
            .mcp_clients
            .clients
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    mod sync {
        use std::sync::Arc;

        use super::*;
        use crate::tools::ToolResolver;

        #[tokio::test]
        async fn syncs_everything_without_parent() {
            let url = spawn_mock_server().await;
            let inventory = synced(&servers(&url, None), &skills(&["pdf"]), None).await;

            assert_eq!(client_names(&inventory), ["mock"]);
            assert!(!inventory.skill_catalog.tools.is_empty());
        }

        #[tokio::test]
        async fn leaves_remote_servers_to_the_provider() {
            let url = spawn_mock_server().await;
            let remote = McpServerSet::from_iter([McpServer::http(url, "mock")]);
            let inventory = synced(&remote, &SkillSet::new(), None).await;

            assert!(client_names(&inventory).is_empty());
        }

        #[tokio::test]
        async fn skips_what_the_parent_reflects() {
            let url = spawn_mock_server().await;
            let (servers, skills) = (servers(&url, Some(&["echo"])), skills(&["pdf"]));
            let parent = synced(&servers, &skills, None).await;
            let child = synced(&servers, &skills, Some(&parent)).await;

            assert!(client_names(&child).is_empty());
            assert!(child.skill_catalog.tools.is_empty());
            assert_eq!(client_names(&parent), ["mock"]);
        }

        #[tokio::test]
        async fn syncs_what_the_parent_does_not_reflect() {
            let url = spawn_mock_server().await;
            let parent = synced(&servers(&url, Some(&["echo"])), &skills(&["pdf"]), None).await;

            // Both the allow-list and the skills differ from the parent's.
            let child = synced(&servers(&url, None), &skills(&["xlsx"]), Some(&parent)).await;

            assert_eq!(client_names(&child), ["mock"]);
            assert!(!child.skill_catalog.tools.is_empty());
        }

        #[tokio::test]
        async fn drops_what_the_parent_comes_to_reflect() {
            let url = spawn_mock_server().await;
            let (narrow, pdf) = (servers(&url, Some(&["echo"])), skills(&["pdf"]));
            let parent = synced(&narrow, &pdf, None).await;

            let mut child = synced(&servers(&url, None), &skills(&["xlsx"]), Some(&parent)).await;
            assert_eq!(client_names(&child), ["mock"]);
            assert!(!child.skill_catalog.tools.is_empty());

            // The next step asks for what the parent holds.
            child
                .sync(&narrow, &pdf, true, Some(&parent))
                .await
                .unwrap();
            assert!(client_names(&child).is_empty());
            assert!(child.skill_catalog.tools.is_empty());
        }

        #[tokio::test]
        async fn reuses_the_parents_service() {
            let url = spawn_mock_server().await;
            let parent = synced(&servers(&url, Some(&["echo"])), &SkillSet::new(), None).await;

            // The allow-list differs, so the child needs its own client but not its own connection.
            let child = synced(&servers(&url, None), &SkillSet::new(), Some(&parent)).await;

            assert_eq!(client_names(&child), ["mock"]);
            let key = local_server(&url, None).service_key();
            assert!(Arc::ptr_eq(
                &child.mcp_clients.services[&key],
                &parent.mcp_clients.services[&key]
            ));
        }

        #[tokio::test]
        async fn does_not_reuse_a_service_for_another_url() {
            let (url_a, url_b) = (spawn_mock_server().await, spawn_mock_server().await);
            let parent = synced(&servers(&url_a, None), &SkillSet::new(), None).await;

            // Same name and allow-list, different server: the parent must not stand in for it.
            let child = synced(&servers(&url_b, None), &SkillSet::new(), Some(&parent)).await;

            assert_eq!(client_names(&child), ["mock"]);
            let key_b = local_server(&url_b, None).service_key();
            assert!(child.mcp_clients.services.contains_key(&key_b));
            assert!(!Arc::ptr_eq(
                &child.mcp_clients.services[&key_b],
                &parent.mcp_clients.services[&local_server(&url_a, None).service_key()]
            ));
        }

        #[tokio::test]
        async fn reconnects_when_the_url_changes() {
            let (url_a, url_b) = (spawn_mock_server().await, spawn_mock_server().await);
            let skills = SkillSet::new();
            let mut inventory = synced(&servers(&url_a, None), &skills, None).await;

            inventory
                .sync(&servers(&url_b, None), &skills, true, None)
                .await
                .unwrap();

            assert_eq!(client_names(&inventory), ["mock"]);
            assert_eq!(inventory.mcp_clients.services.len(), 1);
            let key_b = local_server(&url_b, None).service_key();
            assert!(Arc::ptr_eq(
                &inventory.mcp_clients.clients["mock"].service,
                &inventory.mcp_clients.services[&key_b]
            ));
        }

        #[tokio::test]
        async fn is_idempotent() {
            let url = spawn_mock_server().await;
            let (servers, skills) = (servers(&url, None), skills(&["pdf"]));
            let mut inventory = synced(&servers, &skills, None).await;
            inventory.sync(&servers, &skills, true, None).await.unwrap();

            assert_eq!(client_names(&inventory), ["mock"]);
            let resolver = &inventory;
            assert!(resolver.resolve_tool("echo").is_some());
            assert!(resolver.resolve_tool("greet").is_some());
        }

        #[tokio::test]
        async fn resyncs_when_the_allow_list_changes() {
            let url = spawn_mock_server().await;
            let skills = SkillSet::new();
            let mut inventory = synced(&servers(&url, Some(&["echo"])), &skills, None).await;
            assert!((&inventory).resolve_tool("echo").is_some());
            assert!((&inventory).resolve_tool("greet").is_none());

            inventory
                .sync(&servers(&url, None), &skills, true, None)
                .await
                .unwrap();
            assert!((&inventory).resolve_tool("echo").is_some());
            assert!((&inventory).resolve_tool("greet").is_some());
        }

        #[tokio::test]
        async fn drops_clients_of_removed_servers() {
            let url = spawn_mock_server().await;
            let skills = SkillSet::new();
            let mut inventory = synced(&servers(&url, None), &skills, None).await;
            assert_eq!(client_names(&inventory), ["mock"]);

            inventory
                .sync(&McpServerSet::new(), &skills, true, None)
                .await
                .unwrap();
            assert!(client_names(&inventory).is_empty());
            assert!((&inventory).resolve_tool("echo").is_none());
        }
    }

    mod resolver {
        use super::*;
        use crate::tools::{ToolExt, ToolResolver};

        #[tokio::test]
        async fn resolves_client_and_skill_tools() {
            let url = spawn_mock_server().await;
            let inventory = synced(&servers(&url, None), &skills(&["pdf"]), None).await;

            let resolver = &inventory;
            let mut names = resolver
                .list_tools()
                .iter()
                .map(|spec| spec.name().to_owned())
                .collect::<Vec<_>>();
            names.sort();
            assert_eq!(names, ["echo", "greet", "load_skill", "read_skill_file"]);

            assert!(resolver.resolve_tool("echo").is_some());
            assert!(resolver.resolve_tool("load_skill").is_some());
            assert!(resolver.resolve_tool("missing").is_none());
        }

        #[tokio::test]
        async fn calls_client_tools_through_the_server() {
            let url = spawn_mock_server().await;
            let inventory = synced(&servers(&url, None), &SkillSet::new(), None).await;

            let echo = (&inventory).resolve_tool("echo").unwrap();
            let output = echo.call(serde_json::json!({ "text": "hi there" })).await;
            assert!(output.unwrap().to_string().contains("hi there"));
        }
    }

    mod masked {
        use super::*;
        use crate::tools::{ToolExt, ToolResolver};

        fn tool_names<'a>(resolver: &impl ToolResolver<'a>) -> Vec<String> {
            let mut names = resolver
                .list_tools()
                .iter()
                .map(|spec| spec.name().to_owned())
                .collect::<Vec<_>>();
            names.sort();
            names
        }

        #[tokio::test]
        async fn hides_the_servers_and_skills_the_child_replaces() {
            let url = spawn_mock_server().await;
            let parent = synced(&servers(&url, None), &skills(&["pdf"]), None).await;
            let child = synced(
                &servers(&url, Some(&["echo"])),
                &skills(&["xlsx"]),
                Some(&parent),
            )
            .await;

            let masked = parent.masked_by(&child);
            assert!(tool_names(&masked).is_empty());
            assert!(masked.resolve_tool("greet").is_none());
        }

        #[tokio::test]
        async fn keeps_what_the_child_does_not_replace() {
            let url = spawn_mock_server().await;
            let (servers, skills) = (servers(&url, None), skills(&["pdf"]));
            let parent = synced(&servers, &skills, None).await;
            // The child asks for what the parent holds, so it holds nothing.
            let child = synced(&servers, &skills, Some(&parent)).await;

            let masked = parent.masked_by(&child);
            assert_eq!(
                tool_names(&masked),
                ["echo", "greet", "load_skill", "read_skill_file"]
            );
            assert!(masked.resolve_tool("greet").is_some());
        }

        #[tokio::test]
        async fn hides_the_servers_the_child_hands_to_the_provider() {
            let url = spawn_mock_server().await;
            let parent = synced(&servers(&url, None), &SkillSet::new(), None).await;
            // The child's server has the same name, but the provider handles it.
            let remote = McpServerSet::from_iter([McpServer::http(url, "mock")]);
            let child = synced(&remote, &SkillSet::new(), Some(&parent)).await;
            assert!(client_names(&child).is_empty());

            let masked = parent.masked_by(&child);
            assert!(tool_names(&masked).is_empty());
            assert!(masked.resolve_tool("echo").is_none());
        }

        #[tokio::test]
        async fn stops_hiding_once_the_child_no_longer_hands_the_server_over() {
            let url = spawn_mock_server().await;
            let parent = synced(&servers(&url, None), &SkillSet::new(), None).await;
            let remote = McpServerSet::from_iter([McpServer::http(url.clone(), "mock")]);
            let mut child = synced(&remote, &SkillSet::new(), Some(&parent)).await;

            // The next step asks for what the parent holds.
            child
                .sync(&servers(&url, None), &SkillSet::new(), true, Some(&parent))
                .await
                .unwrap();
            assert!(tool_names(&parent.masked_by(&child)).contains(&"echo".to_owned()));
        }
    }
}
