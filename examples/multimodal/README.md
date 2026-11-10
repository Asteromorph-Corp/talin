# `multimodal`

This example shows how to send images, audio, video, and documents to a multimodal model and
receive a text description.

## Configuration

### `agents.toml`

The `multimodal` agent declares one attachment slot per media kind. Each slot name becomes a field
on the generated input struct, typed by its kind.

```toml
[multimodal]
system_prompt = "You are a helpful AI assistant"
user_prompt = "Describe each attachment you weregiven."

attachments.images = ["images"]
attachments.audios = ["audios"]
attachments.videos = ["videos"]
attachments.documents = ["documents"]
```

### `models.toml`

Several multimodal models are pre-configured: OpenAI, Anthropic, Gemini, and xAI. 

The list of supported modalities varies by model.
See [modality support](/docs/models.md#modality-support) for the full matrix.

## Attachments

The example reads its attachments from `assets/`, and leaves a slot empty when the file is missing:

| Slot        | File                |
| ----------- | ------------------- |
| `images`    | `assets/sample.png` |
| `audios`    | `assets/sample.mp3` |
| `videos`    | `assets/sample.mp4` |
| `documents` | `assets/sample.pdf` |

Swap in your own files, or delete the ones a provider does not accept.

## Running

```bash
# Default (Google Gemini Flash)
cargo run

# OpenAI gpt-4o-mini (delete assets/sample.mp3 first)
cargo run -- gpt_4o_mini

# Anthropic Claude Haiku (delete assets/sample.mp3 first)
cargo run -- claude_haiku_45
```

The model is selected via the first CLI argument, using a model name from `configs/models.toml`. When omitted, it defaults to Gemini.
