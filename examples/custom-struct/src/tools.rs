use std::error::Error;

use talin::tool;
use schemars::JsonSchema;
use serde::Deserialize;

/// The rooms that can be booked.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Room {
    // Enum variants must not have doc comment, or the schema is generated with `oneOf` instead of
    // `enum`.
    Small,
    Large,
}

// The name is fixed by the tool's name in `tools.toml`: `book_room` -> `BookRoomTool` ->
// `BookRoomToolInput`. Doc comments become the descriptions the model reads.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct BookRoomToolInput {
    /// The room to book.
    pub room: Room,
    /// The hour of the day the meeting starts, from 0 to 23.
    #[schemars(with = "i32")]
    pub start_hour: u8,
    /// How many hours the meeting lasts. One hour when omitted.
    #[schemars(required, with = "Option<i32>")]
    pub hours: Option<u8>,
    /// The people who will attend.
    pub attendees: Vec<String>,
}

// And `BookRoomToolConfig` for the configuration. It also has to be `Default`.
#[derive(Debug, Default, Deserialize)]
pub struct BookRoomToolConfig {
    pub building: String,
    pub max_hours: u8,
}

pub struct BookRoomTool {
    building: String,
    max_hours: u8,
}

#[tool]
impl BookRoomTool {
    async fn call(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        let hours = input.hours.unwrap_or(1);
        if hours > self.max_hours {
            return Err(format!(
                "meetings in this building last at most {} hours",
                self.max_hours
            )
            .into());
        }
        Ok(format!(
            "Booked the {:?} room in the {} building from {}:00 for {hours}h, for {}.",
            input.room,
            self.building,
            input.start_hour,
            input.attendees.join(", "),
        ))
    }

    fn from_config(config: BookRoomToolConfig) -> Self {
        Self {
            building: config.building,
            max_hours: config.max_hours,
        }
    }
}
