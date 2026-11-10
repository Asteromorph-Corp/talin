use serde::{Deserialize, Serialize};

use crate::config::StructDef;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentResponseFormat {
    pub name: String,
    pub description: Option<String>,
    pub fields: StructDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SchemaMode {
    Prompt,
    JsonObject,
    JsonSchema,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_agent_response_format() {
        let json_json_schema = r#"{
            "name": "TestSchema",
            "description": "A test JSON schema",
            "fields": {
                "field1": { "type": "String", "description": "A string field" },
                "field2": "u8"
            }
        }"#;
        let format: AgentResponseFormat = serde_json::from_str(json_json_schema).unwrap();
        assert_eq!(format.name, "TestSchema");
        assert_eq!(format.description.unwrap(), "A test JSON schema");
    }
}
