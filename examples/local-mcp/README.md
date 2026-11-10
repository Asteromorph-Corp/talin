# `local-mcp`

This example shows an agent using the tools of an [MCP](https://modelcontextprotocol.io/) server through a local MCP client.
The MCP logic is handled by the `talin` crate, and the tools are advertised as regular function tools to the provider.

The agent is a combat narrator for a D&D game, and uses the dice roller tool provided by the MCP server.

The example offers two MCP server implementations, one using the HTTP transport and one using stdio.
The servers are part of the example at [`src/bin/server_http.rs`](src/bin/server_http.rs) and [`src/bin/server_stdio.rs`](src/bin/server_stdio.rs), which share the dice logic in [`src/lib.rs`](src/lib.rs).

## Why a local client

For providers that do not offer support, we need a local MCP client to connect to a MCP server.
However, even for providers that do (see [`remote-mcp`](../remote-mcp)), a local client may be desirable in certain situations:

- **It can reach servers the provider cannot.** That includes servers on `localhost` or a private network, and servers behind a firewall.
- **Credentials stay with you.** A server's authorization token is not shared with the provider.
- **It can use stdio servers.** A server that runs as a child process on your machine is only reachable from your machine.


## HTTP transport

For an MCP server with HTTP transport, if the provider offers MCP support, `talin` will try to delegate MCP support to the client instead of managing a local client itself.
In order to use local client regardless of the provider, you can set `always_local = true` in the server's configuration.

```toml
[dice_http]
url = "http://localhost:8000/mcp"
tools = ["roll_dice"]
always_local = true
```

In order to run the example with HTTP transport, you need to start the server first, then run the agent in another terminal:

```sh
cargo run --features server-http --bin server-http
OPENAI_API_KEY=... cargo run -- gpt_4o_mini http
```

## Stdio transport

A stdio server is always accessed through a local client.

The agent starts the server as a child process, so nothing else needs to be running:

```sh
OPENAI_API_KEY=... cargo run -- gpt_4o_mini stdio
```

```toml
# servers.toml
[dice_stdio]
command = "cargo"
args = ["run", "--quiet", "--features", "server-stdio", "--bin", "server-stdio"]
tools = ["roll_dice"]
```

Note that this will run an any command that is specified in `servers.toml`.
You should always verify the commands in `servers.toml`, and prefer containerized environments for untrusted servers.

For this reason, the stdio transport is disabled by default, and requires the `mcp-stdio` feature of `talin`:

```toml
talin = { version = "...", features = ["mcp-stdio"] }
```

## Tool Configuration

An agent lists the tools it may call as `mcp.<server>/<tool>`, or `mcp.<server>/*` for all of the server's tools:

```toml
# agents.toml
[combat_http]
system_prompt = "{{config.DND_SYSTEM}}"
user_prompt = "{{config.DND_USER}}"
tools = ["mcp.dice_http/*"]

[combat_stdio]
system_prompt = "{{config.DND_SYSTEM}}"
user_prompt = "{{config.DND_USER}}"
tools = ["mcp.dice_stdio/*"]
```

The two agents share their prompts through `prompts.toml`, see [`shared-templates`](../shared-templates).
To the model, the tools are the same either way.

## Next

- Servers that the provider connects to itself: [`remote-mcp`](../remote-mcp).
