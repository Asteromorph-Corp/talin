use std::collections::BTreeMap;
use std::hash::Hash;

use heck::ToSnakeCase;
use indexmap::IndexMap;

use crate::config::Prompt;
use crate::error::DuplicateConfig;

pub(crate) fn collect_configs<S, K, C>(
    configs: IndexMap<S, BTreeMap<K, C>>,
    config_type: &'static str,
) -> Result<BTreeMap<K, C>, DuplicateConfig>
where
    S: AsRef<str> + Eq + Hash,
    K: AsRef<str> + Ord,
{
    configs.iter().try_fold(
        BTreeMap::new(),
        |mut acc: BTreeMap<String, &str>, (path, config)| {
            for key in config.keys() {
                let key = key.as_ref().to_snake_case();
                if let Some(existing) = acc.get(&key).cloned() {
                    return Err(DuplicateConfig {
                        config_type,
                        key,
                        source1: path.as_ref().to_owned(),
                        source2: existing.to_owned(),
                    });
                }
                acc.insert(key, path.as_ref());
            }
            Ok(acc)
        },
    )?;

    let result = configs.into_iter().flat_map(|(_, config)| config).collect();

    Ok(result)
}

pub(crate) fn collect_prompts<'a>(
    agent_id: &str,
    shared: &'a BTreeMap<String, Prompt>,
    agent: &'a BTreeMap<String, Prompt>,
) -> Result<BTreeMap<&'a str, &'a Prompt>, DuplicateConfig> {
    let shared = shared
        .iter()
        .map(|(k, v)| (k.as_str(), v))
        .collect::<BTreeMap<_, _>>();
    let agent = agent
        .iter()
        .map(|(k, v)| (k.as_str(), v))
        .collect::<BTreeMap<_, _>>();

    let configs = IndexMap::from_iter([
        (String::from("shared prompts"), shared),
        (format!("{agent_id}.prompts"), agent),
    ]);
    collect_configs(configs, "prompt")
}
