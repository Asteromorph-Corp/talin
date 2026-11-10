use serde::{Deserialize, Serialize};

use crate::config::Merge;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Attachments {
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default)]
    pub audios: Vec<String>,
    #[serde(default)]
    pub videos: Vec<String>,
    #[serde(default)]
    pub documents: Vec<String>,
}

impl Merge<Self> for Attachments {
    fn merge(self, parent: Self) -> Self {
        let images = self.images.into_iter().chain(parent.images).collect();
        let audios = self.audios.into_iter().chain(parent.audios).collect();
        let videos = self.videos.into_iter().chain(parent.videos).collect();
        let documents = self.documents.into_iter().chain(parent.documents).collect();

        Self {
            images,
            audios,
            videos,
            documents,
        }
    }
}
