# Compaction compatibility matrix

Live results for every compaction strategy against every provider endpoint, recorded on
2026-10-20 at commit `57b4d79` (`feat/auto-compaction`).

## Setup

| Endpoint             | Base URL                       | Model                        |
| -------------------- | ------------------------------ | ---------------------------- |
| Anthropic            | `https://api.anthropic.com/v1` | `claude-haiku-5-5`           |
| OpenAI Responses     | `https://api.openai.com/v1`    | `gpt-5-nano`                 |
| OpenAI Chat          | `https://api.openai.com/v1`    | `gpt-5-nano`                 |
| OpenRouter Messages  | `https://openrouter.ai/api/v1` | `anthropic/claude-haiku-5.5` |
| OpenRouter Responses | `https://openrouter.ai/api/v1` | `openai/gpt-5-nano`          |
| OpenRouter Chat      | `https://openrouter.ai/api/v1` | `openai/gpt-5-nano`          |
| xAI (see below)      | `https://api.x.ai/v1`          | `grok-4.20-0309-non-reasoning` |

Claude clients send `anthropic-beta: compact-2026-01-12` for automatic compaction and
`compact-2026-09-04` for on-demand compaction.

Each case runs three turns in one session:

1. The user states a random project codename and asks the agent to call `load_archive` once.
   The tool returns generated archive rows; a random revision appears only in that tool result.
   A second `load_archive` call fails, and each turn is capped at 6 iterations.
2. Without tools, the agent replies with the codename and the revision.
3. The same question again.

Thresholds are sized so that compaction actually triggers:

| Policy                              | Trigger                                          | Archive size              |
| ----------------------------------- | ------------------------------------------------ | ------------------------- |
| Claude automatic                    | `threshold = 50_000` input tokens                | 2400 rows (~188 KB)       |
| OpenAI automatic                    | `compact_threshold = 4000` input tokens          | 400 rows (~35 KB)         |
| `TokenBudget`                       | `threshold = 3000`, `retain_tokens = 1000`       | 400 rows                  |
| `MessageCount`                      | `threshold = 4`, `retain_messages = 2`           | 400 rows                  |
| `CompactionCutoff`                  | the archive tool result, set after turn 1        | 400 rows                  |

"Own summarizer" appends a summarization instruction to the lowered prefix, forces a text
answer (`force_final`), and stores the reply as the native summary. The matrix ran it as a test
compactor; the same logic now ships as `InstructionCompactor`, which additionally uses default
call options and rejects truncated or tool-calling replies.

## Verdicts

A cell is `OK` only when all three checks hold:

- Triggered: an on-demand `CompactionMarker` plus native summary was inserted, or a response
  carried a compaction block/item with a non-empty summary.
- Archive dropped: the last outgoing request contains no archive rows. Rows are counted, not
  the `ARCHIVE_NOISE_START` marker, because summaries may quote the marker.
- Recall: both answers after compaction contain the codename and the revision.

| Verdict   | Meaning                                                                    |
| --------- | -------------------------------------------------------------------------- |
| `OK`      | Compacted, archive dropped from later requests |
| `FAIL`    | The provider rejected or ignored the compaction request |
| `n/a`     | The strategy does not apply to the endpoint (does not compile or exist) |

## Matrix

| Strategy                                                  | Anthropic Messages | OpenAI Resp. | OpenAI Chat | OR Messages | OR Resp.   | OR Chat |
| --------------------------------------------------------- | ----------- | ------------ | ----------- | ----------- | ---------- | ------- |
| `ClaudeMessagesAutoCompactStrategy` / `OpenAIResponsesAutoCompactStrategy` | OK | OK  | n/a         | OK          | FAIL [1]   | n/a     |
| `AutoCompactStrategy`                                     | OK          | OK           | n/a         | OK          | FAIL [1]   | n/a     |
| `ClaudeMessagesCompactStrategy` / `OpenAIResponsesCompactStrategy` (`TokenBudget`) | OK | OK | n/a | FAIL [2] | FAIL [3] | n/a |
| `CustomCompactStrategy(native compactor, MessageCount)`   | OK          | OK           | n/a         | FAIL [4]    | FAIL [3]   | n/a     |
| `CustomCompactStrategy(native compactor, CompactionCutoff)` | OK        | OK           | n/a         | FAIL [4]    | FAIL [3]   | n/a     |
| `CustomCompactStrategy(own summarizer, TokenBudget)`      | OK [5] | OK           | n/a [6]     | FAIL [2]    | FAIL [3]   | n/a [6] |
| `CustomCompactStrategy(own summarizer, MessageCount)`     | OK [5]      | OK           | OK          | OK          | OK         | OK      |
| `CustomCompactStrategy(own summarizer, CompactionCutoff)` | OK [5]      | OK           | OK          | OK          | OK         | OK      |

"Native compactor" is `ClaudeMessagesCompactor` for Claude endpoints and
`OpenAIResponsesCompactor` for Responses endpoints. "OR" is OpenRouter.

## xAI

xAI was tested through three clients against `https://api.x.ai/v1`: the library's own
`XaiResponses` backend, `OpenAIResponses` (xAI's Responses API is OpenAI-compatible, including
`POST /v1/responses/compact`), and `OpenAIChat`.

| Strategy                                                  | `XaiResponses` | `OpenAIResponses` | `OpenAIChat` |
| --------------------------------------------------------- | -------------- | ----------------- | ------------ |
| `OpenAIResponsesAutoCompactStrategy`                      | n/a            | FAIL [7]          | n/a          |
| `OpenAIResponsesCompactStrategy` (`TokenBudget`)          | n/a            | FAIL [8]          | n/a          |
| `CustomCompactStrategy(OpenAIResponsesCompactor, MessageCount)`     | n/a  | OK [9]       | n/a          |
| `CustomCompactStrategy(OpenAIResponsesCompactor, CompactionCutoff)` | n/a  | OK [9]       | n/a          |
| `CustomCompactStrategy(InstructionCompactor, MessageCount)`         | OK   | OK           | OK           |
| `CustomCompactStrategy(InstructionCompactor, CompactionCutoff)`     | OK   | OK           | OK           |

`InstructionCompactor` recalled both facts in every xAI case.

Gemini (`GeminiGenerateContent`, `GeminiInteractions`, and the OpenAI-compatible endpoint) was not
tested: the project behind the available key returned `429: Your project has exceeded its
monthly spending cap`.

## Notes

1. OpenRouter Responses ignores `context_management`. Every request carried
   `compact_threshold: 4000` and grew from 37 KB to 51 KB with all 400 archive rows, yet no
   response contained a compaction item. The run then stopped because `gpt-5-nano` kept calling
   `load_archive` again, which the test tool rejects.
2. OpenRouter Messages has no `/messages/count_tokens`: HTTP 404, so `TokenBudget` cannot run.
3. OpenRouter Responses has no `/responses/input_tokens` or `/responses/compact`: HTTP 404.
   The error surfaces as `failed to deserialize response: invalid type: integer 404, expected a
   string`, because OpenRouter sends a numeric error `code` while `async_openai::error::ApiError`
   expects a string. This parsing issue predates the branch (same in `async-openai` 0.41 and
   0.42) and hides the provider's message.
4. Claude on-demand compaction is not a separate endpoint: it is a normal `POST /messages`
   with a top-level `compaction: {"type": "summarize"}` field and the `compact-2026-09-04` beta
   header. OpenRouter Messages silently drops request fields it does not know, while Anthropic
   rejects them: a request with an invented top-level field returns HTTP 200 from OpenRouter
   but `400 invalid_request_error: ... Extra inputs are not permitted` from Anthropic. The
   `compaction` field is dropped the same way, so the model returns an ordinary answer
   (`stop_reason: end_turn`, no compaction block) and `ClaudeMessagesCompactor` reports
   `on-demand compaction returned no summary (stop reason: Some(EndTurn))`. Automatic compaction
   (`context_management` with `compact-2026-01-12`) is forwarded and works through the same
   endpoint.
5. Summary quality, not library behavior. With `TokenBudget`, the first compaction runs in turn 1
   before the tool call and summarizes only the user message. Claude answered the instruction
   instead of summarizing ("I haven't loaded the archive, so I don't have a revision to
   record..."), so the codename was lost; later requests still had 0 of 400 archive rows. The
   built-in `ClaudeMessagesCompactStrategy` passed the same scenario. With `MessageCount` and
   `CompactionCutoff`, the own summarizer lost recall in one of three runs on Anthropic.
6. `TokenBudget` requires `CompactionModel::count_tokens`, which Chat Completions does not
   implement, so the combination does not compile.
7. xAI accepts `context_management` without error but never compacts: all four requests still
   carried the 400 archive rows and no response contained a compaction item. xAI documents only
   on-demand compaction.
8. xAI has no request token-count endpoint. `POST /v1/responses/input_tokens` returns 405 with
   an empty body, the same status xAI returns for any unknown `POST` path (a `GET` gives 404), so
   the client reports `failed to deserialize response: EOF while parsing a value`. The only
   token API xAI documents is `POST /v1/tokenize-text`, which tokenizes a single string.
9. `/responses/compact` works: the compaction item (`type`, `id`, `encrypted_content`) is stored
   and sent back verbatim, and later requests drop the archive. Recall was 0/2, though: the
   codename survived but the revision, which existed only in the tool result, was replaced by an
   invented value. A raw API call with no library code shows the same loss, matching xAI's
   documentation that compaction keeps system prompts, files, reasoning, and a record of the
   turns "while dropping the verbose tool output".

## Evidence for the thresholds

Per-request traces (bytes, archive rows, compaction summary length) from the final run:

```
anthropic / ClaudeMessagesAutoCompactStrategy
  request 0:   1981 bytes, archive rows    0
  request 1: 188447 bytes, archive rows 2400            (response carries the compaction)
  request 2:   3206 bytes, archive rows    1, summary 1133 chars   (row quoted by the summary)
  request 3:   3424 bytes, archive rows    1, summary 1133 chars

anthropic / CustomCompactStrategy(ClaudeMessagesCompactor, CompactionCutoff)
  request 0:   1841 bytes, archive rows    0
  request 1:  32907 bytes, archive rows  400
  request 2:   5448 bytes, archive rows    1, summary  789 chars
  request 3:   5666 bytes, archive rows    1, summary  789 chars

openrouter-responses / OpenAIResponsesAutoCompactStrategy
  request 0:   1954 bytes, archive rows    0
  request 1:  37703 bytes, archive rows  400
  request 2:  43991 bytes, archive rows  400            (no compaction item returned)
  request 3:  51666 bytes, archive rows  400
```

On-demand compaction stores a `CompactionMarker` followed by an `Entry::Native` summary right
after the selected prefix; automatic compaction leaves the compaction block inside the native
response record of the step that crossed the threshold.

## Summary

- Direct Anthropic and OpenAI Responses support every strategy.
- Chat Completions (OpenAI or OpenRouter) supports application-defined compaction with
  `MessageCount` or `CompactionCutoff`.
- OpenRouter supports automatic compaction only on its Messages endpoint, and on-demand
  compaction only through an application-defined summarizer with `MessageCount` or
  `CompactionCutoff`.
- xAI supports native on-demand compaction through `OpenAIResponsesCompactor` (tool output is
  dropped from the summary) and `InstructionCompactor` on every client; it has no automatic
  compaction and no request token counting.
