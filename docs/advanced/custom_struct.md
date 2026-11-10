# Custom Struct

Couple fields in the configuration files uses the syntax `{ <field> = "<type>" }` or `{ <field> = { type = "<type>", description = "<description>" }`.
These fields include:
- `input` in `tools.toml` for tool input
- `init.<init_id>` in `tools.toml` for tool initializers
- `response_format.fields` in `agents.toml` for agent output

Internally, these are used to generate a corresponding rust struct.

However, if you don't like the generated struct, you can define your own custom struct for these fields.
This is usually done by setting the field to `"custom"` instead of using the usual syntax.
The struct should follow a specific naming convention, and implement necessary traits.

## Tool Input

In order to use a custom struct for a tool's input, set the `input` field to `"custom"` in `tools.toml`:

```toml
# tools.toml
[<tool_name>]
# Other tool configuration...
input = "custom"
```

Then define the input struct in your code:

```rust
// src/tools.rs
#[derive(serde::Deserialize, schemars::JsonSchema)]
struct ToolNameToolInput {
    /// The field description.
    pub field1: String,
    /// The field description.
    pub field2: i64,
}
```

The struct must have the name `[ToolName]ToolInput`, where `[ToolName]` is the pascal case version of the tool name defined in `tools.toml`. 
If the tool name already contains the substring `"tool"` (case insensitive), the `Tool` suffix is not added, and the struct name should simply be `[ToolName]Input`.

The struct should not be a tuple struct, and all fields in the struct must be public. Furthermore, the struct must implement the traits `serde::Deserialize` and `schemars::JsonSchema`.

## Tool Initializer

In order to use a custom struct for a tool's initializer, set the `init.<init_id>` field to `"custom"` in `tools.toml`:

```toml
# tools.toml
[<tool_name>]
# Other tool configuration...
[tool_name.init]
<init_id> = "custom"
```

Then define the initializer struct in your code:

```rust
// src/tools.rs
struct ToolNameToolInitializerInitId {
    pub param1: String,
    pub param2: u32,
}
```

The name of the struct must be `[ToolName]ToolInitializer[InitId]`, where `[ToolName]` and `[InitId]` is the pascal case version of the tool name and init id defined in `tools.toml`.
Like before, If the tool name already contains the substring `"tool"` (case insensitive), the `Tool` suffix is not added, and the struct name should simply be `[ToolName]Input` and `[ToolName]Initializer[InitId]`.

The struct should not be a tuple struct, and all fields in the struct must be public.

## Agent Output

In order to use a custom struct for an agent's response format, set the `response_format` field to `"custom"` in `agents.toml`:

```toml
# agents.toml
[<agent_name>]
# Other agent configuration...
response_format = "custom"
```

Then define the response format struct in your code:

```rust
// src/types.rs
use talin::chain::{ChainOutput, Ctor};

#[derive(Debug, ChainOutput, Ctor)]
#[chain(derive_schema)]
#[serde(deny_unknown_fields)]
pub struct CustomOutputAgentOutput {
    /// The user's full name
    pub name: String,
    /// The user's location
    pub location: String,
    /// The user's email address
    pub email: String,
}
```

The name of the struct must be `[AgentName]AgentOutput`, where `[AgentName]` is the name of the agent in `agents.toml` converted to PascalCase.
Once again, if the agent name already contains the substring `"agent"` (case insensitive), the suffix is not added.

The struct should not be a tuple struct, and all fields in the struct must be public. Furthermore, the struct must implement the traits `serde::Deserialize`, and `schemars::JsonSchema`, `talin::chain::ChainOutput`. It should also be accompanied by a constructor struct, which implements `talin::chain::Ctor`.

It is recommended to use the derive macros to implement the necessary traits, as shown in the example above.

When using the derive macros, you must use the attributes `#[chain(derive_schema)]` and `#[serde(deny_unknown_fields)]`.

### Constructor Import

The `#[derive(Ctor)]` will generate a constructor struct for the struct with the `Ctor` suffix.
In the case of the above example, this constructor would be `CustomOutputAgentOutputCtor`.
You must import this where you call the `talin::include_agents!()` macro.

```rust
use crate::types::CustomOutputAgentOutputCtor;

talin::include_agents!();
```

### `ChainOutput` Attributes

When using the `ChainOutput` derive, all standard `#[serde(...)]` and `#[schemars(...)]` attributes are fully supported and behave exactly as expected.

In addition, you can use the `#[chain(from = ...)]` attribute to control the data source for individual fields.

By default, the model response is deserialized as a JSON object, and each field is populated from the corresponding key in that object.
You can override this behavior per-field using `#[chain(from = ...)]`.

#### Capturing the Raw Model Response

To capture the raw model output, annotate a `String` field with `#[chain(from = response)]`.

```rust
use talin::chain::{ChainOutput, Ctor};

#[derive(Debug, ChainOutput, Ctor)]
#[chain(derive_schema)]
#[serde(deny_unknown_fields)]
pub struct CustomOutputAgentOutput {
    /// The user's full name
    pub name: String,
    /// The user's location
    pub location: String,
    /// The user's email address
    pub email: String,

    #[chain(from = response)]
    /// The raw model response
    pub raw_response: String,
}
```

Then, `raw_response` will hold the unmodified model output as-is, while the other fields are populated from a deserialized JSON object.

#### Deserializing the Raw Response into a Single Field

When `#[chain(from = response)]` is applied to non-`String` fields, the entire model response is deserialized into that field.

```rust
use talin::chain::{ChainOutput, Ctor};

#[derive(Debug, ChainOutput, Ctor)]
#[chain(derive_schema)]
#[serde(deny_unknown_fields)]
pub struct CustomOutputAgentOutput {
    #[chain(from = response)]
    pub names: Vec<String>,
}
```

Although this struct would normally represent a wrapped object:
```json
{
  "names": ["Alice", "Bob", "Charlie"]
}
```

It instead represents a plain json array:
```json
["Alice", "Bob", "Charlie"]
```

The generated JSON schema and the deserialization logic will reflect this behavior.

### Propagating Fields from the Input Struct

Output fields can also be populated directly from the agent's input.

To do so, you must:

1. Declare the input type at the struct level using `#[chain(from_input = InputType)]`.

2. Annotate the individual fields with `#[chain(from = input.<field_name>)]`.

  - If `<field_name>` is omitted, the field with the same name is used automatically.

```rust
use talin::chain::{ChainOutput, Ctor};

pub struct CustomAgentInput {
    pub text: String,
    pub other_field: u32,
}

#[derive(Debug, ChainOutput, Ctor)]
#[chain(derive_schema, from_input = CustomAgentInput)]
#[serde(deny_unknown_fields)]
pub struct CustomOutputAgentOutput {
    /// The user's full name
    pub name: String,
    /// The user's location
    pub location: String,
    /// The user's email address
    pub email: String,

    /// This field is populated with the value of `input.text`
    #[chain(from = input)]
    pub text: String,

    /// This field is populated with the value of `input.other_field`
    #[chain(from = input.other_field)]
    pub renamed_field: u32,
}
```
