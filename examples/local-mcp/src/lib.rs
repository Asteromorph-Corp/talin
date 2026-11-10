//! Logic shared by the D&D MCP example servers.
#![cfg(feature = "server")]

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData as McpError, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Clone, Default)]
pub struct DiceRoller;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RollDiceParams {
    /// Number of sides on each die (e.g., 20 for a d20, 6 for a d6).
    #[schemars(with = "i32")]
    pub sides: u32,
    /// Number of dice to roll.
    #[schemars(with = "i32")]
    pub count: u32,
}

#[tool_router(server_handler)]
impl DiceRoller {
    /// Roll one or more dice with the specified number of sides.
    ///
    /// Returns each individual die result and the total sum.
    #[tool(description = "Roll dice. Returns individual results and the sum.\n\
        Example: roll_dice(sides=20, count=1) for a d20, or roll_dice(sides=6, count=4) for 4d6.")]
    fn roll_dice(
        &self,
        Parameters(RollDiceParams { sides, count }): Parameters<RollDiceParams>,
    ) -> Result<CallToolResult, McpError> {
        if sides == 0 || count == 0 {
            return Ok(CallToolResult::success(vec![ContentBlock::text(
                "Invalid input: sides and count must be greater than 0",
            )]));
        }

        let rolls: Vec<u32> = (0..count)
            .map(|_| rand::random::<u32>() % sides + 1)
            .collect();
        let sum: u32 = rolls.iter().sum();

        let result = if count == 1 {
            format!("🎲 d{sides}: {sum}")
        } else {
            let rolls_str = rolls
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("🎲 {count}d{sides}: [{rolls_str}] = {sum}")
        };

        Ok(CallToolResult::success(vec![ContentBlock::text(result)]))
    }
}
