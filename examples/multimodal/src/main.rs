use std::path::Path;

use talin::schemas::{
    AudioMimeType, DocumentMimeType, ImageMimeType, MediaContent, MediaSource, VideoMimeType,
};

talin::include_models!();
talin::include_agents!();

/// Loads an attachment from `assets/`, leaving the slot empty when the file is absent.
fn load<M>(file: &str, mime: M) -> Vec<MediaContent<M>> {
    let path = Path::new("assets").join(file);
    match std::fs::read(&path) {
        Ok(bytes) => vec![MediaContent::new(MediaSource::from_raw(bytes), mime).with_name(file)],
        Err(e) => {
            log::warn!("Skipping {}: {e}", path.display());
            Vec::new()
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let model = std::env::args()
        .nth(1)
        .unwrap_or("gemini_flash_latest".into());

    let images = load("sample.png", ImageMimeType::Png);
    let audios = load("sample.mp3", AudioMimeType::Mp3);
    let videos = load("sample.mp4", VideoMimeType::Mp4);
    let documents = load("sample.pdf", DocumentMimeType::Pdf);

    // The generated `run_immed` takes one argument per attachment slot, ordered by slot name.
    log::info!("Using model: {model}");
    let preset = Models::any(&model).unwrap_or_else(|e| panic!("{e}"));
    let agent = MultimodalAgent::any(preset).await;
    let result = agent.run_immed(&audios, &documents, &images, &videos).await;

    println!("------------FINISH------------");

    match result {
        Ok(output) => println!("{output:#?}"),
        Err(e) => eprintln!("{e}"),
    }
}
