# `custom-struct`

This example shows how to replace a struct that the build script would generate with one you write yourself.
The [custom struct guide](/docs/advanced/custom_struct.md) has the reference.

```sh
OPENAI_API_KEY=... cargo run
```

## Motivation

`talin` allows users to define typed struct directly in the TOML configurationW.
Then, a Rust struct that matches the schema is generated for you along with the other build artifacts.

```toml
# tools.toml
[book_room]
input = { field = { type = "u8", description = "..." } }
```

While the syntax works well for simple types, it is considerably less expressive than defining the structs directly in Rust.
In particular, it cannot represent enums, optional fields, nested structs, or custom attributes.
If you want full control over the struct instead of relying on the generated struct, set the TOML value to `"custom"`.

```toml
# tools.toml
[book_room]
input = "custom"
config = "custom"
```

The custom struct must follow a particular naming pattern and be implemented manually where it would be used.

## Tool Input

You can use custom structs for a tool's input type.

For the tool named `[tool_name]`, the name of the struct must be `[ToolName]ToolInput`.
For example, the input struct name for the tool `book_room` must be `BookRoomToolInput`.
Additionally, it has to implement `Deserialize` and `JsonSchema`.

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BookRoomToolInput {
    /// The room to book.
    pub room: Room,
    /// How many hours the meeting lasts. One hour when omitted.
    pub hours: Option<u8>,
    // ...
}
```

## Tool Config

You can also use custom struct for a tool's config type.

For the tool named `[tool_name]`, the name of the struct must be `[ToolName]ToolConfig`.
For example, the config struct name for the tool `book_room` must be `BookRoomToolConfig`.
Additionally, it must implement `Deserialize` and `Default`.

Notably, this is the argument of `from_config` method of the `#[tool]` implemntation

```rust
fn from_config(config: BookRoomToolConfig) -> Self { /* ... */ }
```

## Output Structs

Lastly, an agent's output type can be replaced with a custom structa s welll.

For an agent named `[agent_name]`, the name of the struct must be `[AgentName]AgentOutput`.
That one has its own derive macros and attributes.
See [custom struct guide](/docs/advanced/custom_struct.md#agent-output) for more details.

## Provider Requirements

Some providers require the supplied JSON schema to follow certain rules.
For example, OpenAI requires `additionalProperties` to be `false` and `required` to be set to an array of every key in `properties`.
The `#[schemars]` attribute can help you get the schema that satisfies the provider's requirements, but manual tweaks may be necessary.

```rust
/// The rooms that can be booked.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)] // OpenAI do not allow `$ref` fields to have a `description`, so we inline instead of using `$ref`.
pub enum Room {
    // Adding comments to the enum variants cause the schema to be generated as 
    // `oneOf` instead of `enum`, which is not compatible with OpenAI.
    Small,
    Large,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)] // Set `additionalProperties` to `false`.
pub struct BookRoomToolInput {
    /// The room to book.
    pub room: Room,
    /// The hour of the day the meeting starts, from 0 to 23.
    pub start_hour: u8,
    /// How many hours the meeting lasts. One hour when omitted.
    #[schemars(required)] // Set `required` to include this field.
    pub hours: Option<u8>,
    /// The people who will attend.
    pub attendees: Vec<String>,
}
```

## Next

- More ways to write tools: [`custom-tools`](../custom-tools).
