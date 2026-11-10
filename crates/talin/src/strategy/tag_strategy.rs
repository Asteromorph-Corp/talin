//! Tag strategy for assigning unique identifiers to tool outputs.
//!
//! This module provides strategies that automatically tag tool output entries
//! with memorable, unique identifiers. This allows agents to reference specific
//! entries in their reasoning and tool calls.

use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_trait::async_trait;
use indoc::formatdoc;
use once_cell::sync::Lazy;
use random_word::Lang;
use tokio::sync::Mutex;
use twox_hash::XxHash64;

use crate::chain::OutputCtor;
use crate::llm::Llm;
use crate::schemas::{ContentPart, ToolResult, ToolResultMessage};
use crate::strategy::Strategy;

/// A [strategy](crate::strategy::Strategy) that tags tool output entries with unique,
/// memorable identifiers.
///
/// `TagStrategy` assigns a unique English word to each entry in tool outputs,
/// wrapping entries in XML-like tags. This allows agents to easily reference
/// specific entries by their tag instead of repeating full content.
#[derive(Default)]
pub struct TagStrategy {
    index: HashMap<&'static str, String>,
}

impl TagStrategy {
    /// Creates a new [`TagStrategy`] instance.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Implementation of [`Strategy`] for [`TagStrategy`].
///
/// This strategy adds tags to tool output entries and maintains an index of
/// tagged entries for reference.
#[async_trait]
impl<L: Llm, O: OutputCtor> Strategy<L, O> for TagStrategy {
    type Output = HashMap<&'static str, String>;
    type Error = Infallible;

    async fn process_tool_result(
        &mut self,
        message: ToolResultMessage,
    ) -> Result<ToolResult, Self::Error> {
        let Ok(mut output) = message.result else {
            return Ok(message.result);
        };

        let tagged = tag_content(&self.index, &mut output.content);
        self.index.extend(tagged);

        Ok(Ok(output))
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(self.index)
    }
}

/// A [strategy](crate::strategy::Strategy) that tags tool output entries with shared
/// state across agents.
///
/// Similar to [`TagStrategy`], but maintains a shared tag index that can be
/// accessed by multiple agents. This ensures consistent tagging across an
/// entire crew of collaborating agents.
#[derive(Default, Clone)]
pub struct SharedTagStrategy {
    shared_index: Arc<Mutex<HashMap<&'static str, String>>>,
    index: HashMap<&'static str, String>,
}

impl SharedTagStrategy {
    /// Creates a new [`SharedTagStrategy`] with a new shared index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a [`SharedTagStrategy`] from an existing shared index.
    pub fn from_shared(shared_index: Arc<Mutex<HashMap<&'static str, String>>>) -> Self {
        Self {
            shared_index,
            index: HashMap::new(),
        }
    }

    /// Gets a clone of the shared index reference.
    pub fn get_shared(&self) -> Arc<Mutex<HashMap<&'static str, String>>> {
        Arc::clone(&self.shared_index)
    }
}

/// Implementation of [`Strategy`] for [`SharedTagStrategy`].
///
/// This strategy adds tags to tool output entries using a shared index across
/// multiple agents, ensuring consistent tagging.
#[async_trait]
impl<L: Llm, O: OutputCtor> Strategy<L, O> for SharedTagStrategy {
    type Output = HashMap<&'static str, String>;
    type Error = Infallible;

    async fn process_tool_result(
        &mut self,
        message: ToolResultMessage,
    ) -> Result<ToolResult, Self::Error> {
        let Ok(mut output) = message.result else {
            return Ok(message.result);
        };

        let tagged = {
            let mut shared_index = self.shared_index.lock().await;
            let tagged = tag_content(&shared_index, &mut output.content);
            shared_index.extend(tagged.clone());
            tagged
        };
        self.index.extend(tagged);

        Ok(Ok(output))
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(self.index)
    }
}

/// Lazily initialized list of all English words for use as tags.
static WORDS: Lazy<&'static [&'static str]> = Lazy::new(|| random_word::all(Lang::En));

/// First seed for double hashing.
const SEED0: u64 = 0;

/// Second seed for double hashing, using a well-known constant.
const SEED1: u64 = 0x9E3779B185EBCA87;

/// Tags every text part of a tool output with a unique identifier.
///
/// Non-text parts (images, audio, video, documents) are passed through untouched, and the original
/// order of `content` is preserved. Returns the rewritten content alongside the tags minted for it.
fn tag_content(
    index: &HashMap<&'static str, String>,
    content: &mut Vec<ContentPart>,
) -> HashMap<&'static str, String> {
    let mut tagged = HashMap::new();
    for part in content {
        let ContentPart::Text(entry) = part else {
            continue;
        };
        let tag = get_tag(&tagged, index, entry);
        let formatted = format_entry(tag, entry);
        let raw = std::mem::replace(entry, formatted);
        tagged.insert(tag, raw);
    }
    tagged
}

/// Gets or generates a unique tag for the given entry.
///
/// Uses double hashing with open addressing to find an available tag.
/// If the entry already exists in the index or tagged map, returns its existing tag.
fn get_tag(
    tagged: &HashMap<&'static str, String>,
    index: &HashMap<&'static str, String>,
    entry: &str,
) -> &'static str {
    let n = WORDS.len();
    debug_assert!(n >= 2);

    let (h1, h2) = get_hashes(entry);

    let start = (h1 % n as u64) as usize;
    let step = (h2 % (n as u64 - 1)) as usize + 1;

    let mut i = start;
    let mut j = 0;

    loop {
        let tag = WORDS[i];

        match index.get(tag).or(tagged.get(tag)) {
            None => return tag,
            Some(occupant) if *occupant == entry => return tag,
            Some(_) => {
                j += 1;
                i = (start + step * j) % n;
            }
        }
    }
}

/// Computes two independent hash values for double hashing.
fn get_hashes(entry: &str) -> (u64, u64) {
    let mut h1 = XxHash64::with_seed(SEED0);
    entry.hash(&mut h1);

    let mut h2 = XxHash64::with_seed(SEED1);
    entry.hash(&mut h2);

    (h1.finish(), h2.finish())
}

/// Formats an entry with its tag in XML-like markup.
fn format_entry(tag: &str, entry: &str) -> String {
    formatdoc! {"
        <entry>
        <entry_tag>{tag}</entry_tag>
        {entry}
        </entry>"
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::schemas::{ImageMimeType, MediaSource};

    #[test]
    fn test_get_tag_consistent() {
        let empty_tagged = HashMap::new();
        let empty_index = HashMap::new();

        let tag_pikachu = get_tag(&empty_tagged, &empty_index, "Pikachu");
        assert_eq!(tag_pikachu, "trudged");

        let tag_raichu = get_tag(&empty_tagged, &empty_index, "Raichu");
        assert_eq!(tag_raichu, "bronchoscopies");
    }

    #[test]
    fn test_get_tag_reuse() {
        let empty_tagged = HashMap::from_iter([("trudged", "Pikachu".into())]);
        let empty_index = HashMap::new();

        let tag_pikachu = get_tag(&empty_tagged, &empty_index, "Pikachu");
        assert_eq!(tag_pikachu, "trudged");
    }

    #[test]
    fn test_tag_content_preserves_order() {
        let image = ContentPart::Image(
            MediaSource::from_url("https://e.com/a.png").image(ImageMimeType::Png),
        );
        let mut content = vec![
            ContentPart::text("Pikachu"),
            image.clone(),
            ContentPart::text("Raichu"),
        ];

        let tagged = tag_content(&HashMap::new(), &mut content);

        assert_eq!(content.len(), 3);
        assert_eq!(
            content[0],
            ContentPart::text(format_entry("trudged", "Pikachu"))
        );
        assert_eq!(content[1], image);
        assert_eq!(
            content[2],
            ContentPart::text(format_entry("bronchoscopies", "Raichu"))
        );

        assert_eq!(tagged.len(), 2);
        assert_eq!(tagged["trudged"], "Pikachu");
        assert_eq!(tagged["bronchoscopies"], "Raichu");
    }

    #[test]
    fn test_get_tag_probing() {
        let empty_tagged = HashMap::from_iter([("trudged", "Other".into())]);
        let empty_index = HashMap::new();

        let tag_pikachu = get_tag(&empty_tagged, &empty_index, "Pikachu");
        assert_ne!(tag_pikachu, "trudged");
    }
}
