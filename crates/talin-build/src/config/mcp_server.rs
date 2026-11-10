use std::collections::BTreeMap;
use std::convert::Infallible;

use serde::Deserialize;

use crate::config::{FromConfig, InlineOrEnv};

/// How to reach an MCP server: a `url` (streamable HTTP) or a `command` (stdio child process).
#[derive(Debug)]
pub(crate) enum McpTransport {
    Http {
        url: InlineOrEnv<String>,
        authorization: Option<InlineOrEnv<String>>,
        always_local: Option<bool>,
    },
    Stdio {
        command: String,
        args: Vec<String>,
        env: BTreeMap<String, InlineOrEnv<String>>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "RawMcpServer")]
pub(crate) struct McpServer {
    pub transport: McpTransport,
    pub description: Option<String>,
    pub tools: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMcpServer {
    url: Option<InlineOrEnv<String>>,
    authorization: Option<InlineOrEnv<String>>,
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<BTreeMap<String, InlineOrEnv<String>>>,
    description: Option<String>,
    tools: Vec<String>,
    always_local: Option<bool>,
}

impl TryFrom<RawMcpServer> for McpServer {
    type Error = &'static str;

    fn try_from(raw: RawMcpServer) -> Result<Self, Self::Error> {
        let transport = match (raw.url, raw.command) {
            (Some(url), None) => {
                if raw.args.is_some() || raw.env.is_some() {
                    return Err("`args` and `env` are only valid with `command`");
                }
                McpTransport::Http {
                    url,
                    authorization: raw.authorization,
                    always_local: raw.always_local,
                }
            }
            (None, Some(command)) => {
                if raw.authorization.is_some() {
                    return Err("`authorization` is only valid with `url`");
                }
                if raw.always_local.is_some() {
                    return Err("`always_local` is only valid with `url` (stdio is always local)");
                }
                McpTransport::Stdio {
                    command,
                    args: raw.args.unwrap_or_default(),
                    env: raw.env.unwrap_or_default(),
                }
            }
            (Some(_), Some(_)) => return Err("`url` and `command` are mutually exclusive"),
            (None, None) => return Err("one of `url` or `command` is required"),
        };
        Ok(Self {
            transport,
            description: raw.description,
            tools: raw.tools,
        })
    }
}

impl FromConfig for McpServer {
    type Config = Self;
    type Error = Infallible;

    fn from_config(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::neither(r#"tools = []"#, "one of `url` or `command`")]
    #[case::both(
        r#"url = "https://a.com"
command = "x"
tools = []"#,
        "mutually exclusive"
    )]
    #[case::args_with_url(
        r#"url = "https://a.com"
args = ["x"]
tools = []"#,
        "only valid with `command`"
    )]
    #[case::always_local_with_command(
        r#"command = "x"
always_local = true
tools = []"#,
        "stdio is always local"
    )]
    #[case::auth_with_command(
        r#"command = "x"
authorization = "t"
tools = []"#,
        "only valid with `url`"
    )]
    fn rejects_invalid_transport(#[case] toml: &str, #[case] expected: &str) {
        let err = toml::from_str::<McpServer>(toml).unwrap_err().to_string();
        assert!(err.contains(expected), "{err}");
    }
}
