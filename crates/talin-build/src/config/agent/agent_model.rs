use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::config::{CallOptions, Merge, Model};

/// Default model for an agent.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub(crate) enum AgentModel {
    /// A reference to a model defined in `models.toml`.
    Reference(String),
    /// An inline model definition.
    Inline(Box<Model>),
}

/// The agent `model` field, split into the parts it may contain.
#[derive(Debug, Clone)]
pub(crate) struct ModelField {
    pub model: Option<AgentModel>,
    pub call_options: Option<CallOptions>,
}

#[derive(Serialize, Deserialize)]
struct LegacyModel<'a> {
    id: Cow<'a, str>,
    #[serde(flatten, default)]
    call_options: Cow<'a, CallOptions>,
}

impl Serialize for ModelField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match (&self.model, &self.call_options) {
            (Some(model), None) => model.serialize(serializer),
            (None, Some(call_options)) => call_options.serialize(serializer),
            (Some(AgentModel::Reference(id)), Some(call_options)) => LegacyModel {
                id: Cow::Borrowed(id),
                call_options: Cow::Borrowed(call_options),
            }
            .serialize(serializer),
            (Some(AgentModel::Inline(model)), Some(call_options)) => {
                let mut model = model.as_ref().clone();
                model.call_options = call_options.clone().merge(model.call_options);
                model.serialize(serializer)
            }
            (None, None) => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for ModelField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = toml::Value::deserialize(deserializer)?;
        match v {
            // `model = "model_id"`
            toml::Value::String(s) => Ok(ModelField {
                model: Some(AgentModel::Reference(s)),
                call_options: None,
            }),
            // Inline model definition.
            toml::Value::Table(ref table)
                if table.contains_key("api_base") && table.contains_key("model") =>
            {
                let model = Model::deserialize(v).map_err(serde::de::Error::custom)?;
                Ok(ModelField {
                    model: Some(AgentModel::Inline(Box::new(model))),
                    call_options: None,
                })
            }
            // Legacy `model.id = "model_id"` with optional call options mixed in.
            toml::Value::Table(ref table) if table.contains_key("id") => {
                let model = LegacyModel::deserialize(v).map_err(serde::de::Error::custom)?;
                Ok(ModelField {
                    model: Some(AgentModel::Reference(model.id.into_owned())),
                    call_options: Some(model.call_options.into_owned()),
                })
            }
            _ if let Ok(call_options) = CallOptions::deserialize(v) => Ok(ModelField {
                model: None,
                call_options: Some(call_options),
            }),
            _ => Err(serde::de::Error::custom(
                "expected a string or a table `{ api_base = \"..\", model = \"..\" }`",
            )),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::config::{AgentModel, InlineOrEnv, ModelField};

    #[test]
    fn deserializes_string() {
        let value = serde_json::json!("model_id");
        let field: ModelField = serde_json::from_value(value).unwrap();
        let Some(AgentModel::Reference(id)) = field.model else {
            panic!("expected a model reference");
        };
        assert_eq!(id, "model_id");
    }

    #[test]
    fn deserializes_inline() {
        let value = serde_json::json!({
            "api_base": "https://api.example.com",
            "model": "example-model",
            "top_k": 30,
        });
        let field: ModelField = serde_json::from_value(value).unwrap();
        let Some(AgentModel::Inline(model)) = field.model else {
            panic!("expected an inline model");
        };
        assert_eq!(
            model.api_base,
            InlineOrEnv::Inline("https://api.example.com".to_string())
        );
        assert_eq!(
            model.model,
            InlineOrEnv::Inline("example-model".to_string())
        );
        assert_eq!(model.call_options.top_k, Some(30));
        assert!(field.call_options.is_none());
    }

    #[test]
    fn deserializes_legacy() {
        let value = serde_json::json!({
            "id": "legacy-model",
            "top_k": 30,
        });
        let field: ModelField = serde_json::from_value(value).unwrap();
        let Some(AgentModel::Reference(id)) = field.model else {
            panic!("expected a model reference");
        };
        assert_eq!(id, "legacy-model");
        assert_eq!(field.call_options.unwrap().top_k, Some(30));
    }
}
