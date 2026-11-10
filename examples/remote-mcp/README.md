# `remote-mcp`

This example shows an agent using the tools of an [MCP](https://modelcontextprotocol.io/) server through a remote MCP client.
The provider connects to the server and calls the tools itself, and this program only passes along the server's address.

```sh
OPENAI_API_KEY=...    cargo run
ANTHROPIC_API_KEY=... cargo run -- claude_haiku_45
GEMINI_API_KEY=...    cargo run -- gemini_flash_latest
```

The agent connects to [DeepWiki](https://mcp.deepwiki.com), a public MCP server that answers questions about GitHub repositories.

## MCP Client

An MCP server in `servers.toml` is used through one of two clients:

| | Remote client (the provider) | Local client (this program) |
| --- | --- | --- |
| MCP logic | Handled by the provider. | Handled by the `talin` crate. |
| Request payload | Carries only the server URL and configuration parameters | Advertise MCP tools as regular tools to the provider. |
| The server must be | Reachable from the provider's servers: public, with no firewall in the way. | Reachable from this program. |
| Provider | Must offer MCP support. | Works with every provider. |

See [`local-mcp`](../local-mcp) to learn more about how a client is selected.

### Provider Support

| `model_type` | |
| --- | --- |
| `openai_responses`, `xai_responses` | Yes. |
| `claude_messages` | Yes (requires the beta flag `mcp-client-2025-11-20`). |
| `openai_chat`, `generic_chat`, `gemini_generate_content`, `gemini_interactions`, `sglang_generate` | No. |

## Server & Tool Configuration

```toml
# servers.toml
[deepwiki]
url = "https://mcp.deepwiki.com/mcp"
tools = ["read_wiki_structure", "ask_wiki_question"]
```

```toml
# agents.toml
[repo_guide]
# ...
tools = ["mcp.deepwiki/read_wiki_structure", "mcp.deepwiki/ask_wiki_question"]
```

A tool is written `mcp.<server>/<tool>`.

## Authorization

If a server needs a token, add `authorization.env = "MY_SERVER_TOKEN"` to its entry in `servers.toml`.
Note that this is sent to the provider for remote clients.
If this bothers ou, use a [local client](../local-mcp) instead.

## Next

- A local client, over HTTP and over stdio: [`local-mcp`](../local-mcp).
