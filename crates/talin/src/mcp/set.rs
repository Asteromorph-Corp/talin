use std::borrow::Cow;

use indexmap::IndexMap;

use crate::mcp::{McpServer, McpServerHttp};

/// A collection of [`McpServer`]s with distinct names, kept in insertion order.
#[derive(Debug, Clone, Default)]
pub struct McpServerSet {
    servers: IndexMap<String, McpServer>,
}

impl McpServerSet {
    /// Creates an empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// The servers in the set together with `additional`, which win over the set's servers of the
    /// same name.
    ///
    /// Without `additional` servers, the set is borrowed as it is.
    pub(crate) fn with_additional(
        &self,
        additional: Option<McpServerSet>,
    ) -> Cow<'_, McpServerSet> {
        match additional {
            None => Cow::Borrowed(self),
            Some(additional) => Cow::Owned(self.clone().extended(additional)),
        }
    }

    /// Splits the servers into those that are handled locally and those that are handled by the
    /// provider.
    ///
    /// If the model does not support native MCP, all servers are handled locally.
    /// Otherwise, servers that use the `stdio` transport are handled locally, and servers that use
    /// the `http` transport are handled remotely unless they opt in with [`always_local`].
    ///
    /// [`always_local`]: crate::mcp::McpServerHttp::always_local
    pub(crate) fn route(&self, native_mcp: bool) -> McpRouting {
        let (local, remote): (Vec<_>, Vec<_>) = self
            .iter()
            .partition(|server| !native_mcp || server.is_local());
        let local = local.into_iter().cloned().collect();
        let remote = remote.into_iter().filter_map(McpServer::as_http).collect();
        McpRouting { local, remote }
    }

    pub(crate) fn extend(&mut self, other: McpServerSet) {
        self.servers.extend(other.servers);
    }

    /// The servers in the set, with `other`'s added and winning over those of the same name.
    pub(crate) fn extended(mut self, other: McpServerSet) -> McpServerSet {
        self.extend(other);
        self
    }

    /// Adds a server, keyed by its name.
    ///
    /// If the set already has a server with the same name, it is replaced, keeping its position,
    /// and returned.
    pub fn insert(&mut self, server: impl Into<McpServer>) -> Option<McpServer> {
        let server = server.into();
        self.servers.insert(server.name().to_owned(), server)
    }

    /// Removes the server with the given name, preserving the order of the remaining servers.
    pub fn remove(&mut self, name: &str) -> Option<McpServer> {
        self.servers.shift_remove(name)
    }

    /// Looks up a server by name.
    pub fn get(&self, name: &str) -> Option<&McpServer> {
        self.servers.get(name)
    }

    /// Iterates over the servers in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &McpServer> {
        self.servers.values()
    }

    /// The number of servers in the set.
    pub fn len(&self) -> usize {
        self.servers.len()
    }

    /// Whether the set contains no servers.
    pub fn is_empty(&self) -> bool {
        self.servers.is_empty()
    }
}

/// The servers of an [`McpServerSet`], by who handles them.
pub(crate) struct McpRouting {
    /// The servers handled by local clients.
    pub(crate) local: Vec<McpServer>,
    /// The servers handled by the provider.
    pub(crate) remote: Vec<McpServerHttp>,
}

impl<S: Into<McpServer>> FromIterator<S> for McpServerSet {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        let servers = iter
            .into_iter()
            .map(Into::into)
            .map(|server: McpServer| (server.name().to_owned(), server))
            .collect();
        Self { servers }
    }
}

impl<'a> IntoIterator for &'a McpServerSet {
    type Item = &'a McpServer;
    type IntoIter = indexmap::map::Values<'a, String, McpServer>;

    fn into_iter(self) -> Self::IntoIter {
        self.servers.values()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "mcp-stdio")]
    use rstest::rstest;
    use secrecy::ExposeSecret;

    use super::*;
    use crate::mcp::McpServerHttp;
    use crate::mcp::transport::McpTransport;

    fn names(set: &McpServerSet) -> Vec<&str> {
        set.iter().map(McpServer::name).collect()
    }

    #[test]
    fn keeps_insertion_order_and_replaces_by_name() {
        let mut set = McpServerSet::new();
        set.insert(McpServerHttp::new("http://localhost:1", "b"));
        set.insert(McpServerHttp::new("server", "a"));
        set.insert(McpServerHttp::new("http://localhost:1", "c"));
        assert_eq!(names(&set), ["b", "a", "c"]);

        let replaced = set.insert(McpServerHttp::new("http://localhost:2", "a"));

        match replaced {
            Some(McpServer {
                transport: McpTransport::Http(s),
                ..
            }) => assert_eq!(s.url.expose_secret(), "server"),
            _ => panic!("expected an HTTP server to be replaced"),
        }
        assert_eq!(names(&set), ["b", "a", "c"], "the position is kept");
        assert!(!set.get("a").unwrap().is_local());

        assert!(set.remove("b").is_some());
        assert_eq!(names(&set), ["a", "c"]);
        assert_eq!(set.len(), 2);
    }

    #[cfg(feature = "mcp-stdio")]
    #[rstest]
    #[case::native_support(true, vec!["pinned", "process"], vec!["remote"])]
    #[case::without_native_support(false, vec!["pinned", "process", "remote"], vec![])]
    fn routes_servers(
        #[case] native_mcp: bool,
        #[case] expected_local: Vec<&str>,
        #[case] expected_remote: Vec<&str>,
    ) {
        let set = McpServerSet::from_iter([
            McpServer::http("http://localhost:1/mcp", "remote"),
            McpServer::from(
                McpServerHttp::new("http://localhost:1/mcp", "pinned").always_local(true),
            ),
            McpServer::stdio("server", "process"),
        ]);
        let routing = set.route(native_mcp);

        let mut local = routing
            .local
            .iter()
            .map(McpServer::name)
            .collect::<Vec<_>>();
        let mut remote = routing
            .remote
            .iter()
            .map(|server| server.name())
            .collect::<Vec<_>>();
        local.sort();
        remote.sort();

        assert_eq!(local, expected_local);
        assert_eq!(remote, expected_remote);
    }
}
