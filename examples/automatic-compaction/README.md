This application exercises automatic compaction using the library's generated agents and tools.
`configs/agents.toml` defines the remember and recall turns; `configs/models.toml` defines the
models and output limits; `configs/tools.toml` declares `LoadArchive`; `configs/workload.toml`
defines the threshold and generated tool text.

Run from this directory, so that `examples/.env` is found:

```sh
cargo run -- openai
cargo run -- claude
cargo run -- claude --stream
```

These commands use real, paid APIs. The default models are GPT-5-nano and Claude Sonnet 5.5.
Change configuration in TOML. `--report` selects the JSON evidence file. Each live run allows
eight requests across both turns, with one consecutive failure allowed and no continuations.

The archive is generated inventory data used to grow the context. It is not a repository archive.
Its revision is a random identifier for this generated dataset. The first user turn supplies four
random project facts, including a numeric `u16` port. The tool supplies the revision. The later
turn returns a typed `Recall`; the application compares it with the supplied facts.

`scenario.rs` calls `run_turn` and `continue_session`. It never inserts transcript messages or
markers itself. The normal runner records model responses, invokes tools, and applies
`AutoCompactStrategy` for OpenAI Responses or `ClaudeMessagesAutoCompactStrategy` for Claude
Messages while constructing subsequent requests. These strategies deserialize directly from `configs/workload.toml`.
`Trace` observes typed provider requests and responses.

Validation asks distinct questions:

- Did the tool result actually reach the API?
- Did the API return a successful native compaction?
- Did a later request reuse that native compaction while omitting the old inventory rows?
- Did the model recall the supplied facts?

These checks do not prove that the chosen compaction architecture is correct for all providers.
They expose concrete behavior in this workload.

The original live captures show:

| Run | Observed behavior |
| --- | --- |
| Claude, nonstreaming | Paused for compaction, resumed, recalled all five facts |
| Claude, streaming | Same behavior; exposed an empty-input tool stream parsing bug |
| GPT-5-nano | Recalled the four user facts but lost the tool's dataset revision |
| GPT-5.4-nano and GPT-5-mini | Requested the archive a second time after compaction |

The rewritten application also completed a fresh Claude streaming run. Its GPT-5-nano run
first exposed an OpenAI rejection of the generated empty-tool schema. The generator now emits
explicit `properties = {}` and `required = []` for empty inputs. The subsequent live request
was accepted (HTTP 200), returned a compaction, and requested the tool again; the saved
`typed-openai` capture and replay preserve that failure. The replay matches the API-accepted
tool definitions as well as conversation history, so removing the schema fix breaks it.

The raw HTTP captures are in `../../crates/talin/tests/fixtures/auto-compaction`.
The original reports are in `results`. They are retained as evidence, including failed runs.

The [direct HTTP controls](results/openai-controls.json) compare the original history with and
without a compaction item. **Sending the original history alongside a compaction item does not
restore it to the effective model context.** OpenAI performs context replacement on the server;
client-side removal of the preceding items reduces request size. Consequently, these controls
show different answers with and without compaction, but do not distinguish summary information
loss from a provider defect. See the [official OpenAI compaction documentation](https://developers.openai.com/api/docs/guides/compaction).
The second tool call is observable in the raw response; its cause has not been isolated.

Replay the captured conversations without making paid calls:

```sh
cargo test --test replay
```

These tests call the same application functions as the executable. Outgoing history must match
independently captured, API-accepted requests. Additional turns reuse the captured final response
and verify that fresh strategy clones preserve existing UUIDs and native records, with exactly one
advisory compaction marker before each compacting response. The LLM implementations select
outgoing context from native compaction items.
No test constructs or edits the transcript itself.
Replay reproduces saved behavior; it does not establish current model quality.

The automatic Claude strategy now sets `pause_after_compaction = false`. The live
`typed-claude-no-pause` and `typed-claude-stream-no-pause` captures verify that compaction and the
answer arrive in the same response, followed by a successful recall turn. Each run used three
requests: the archive tool call, compaction with the answer, and recall. The replay also matches
the captured compaction settings. The earlier paused captures remain as historical evidence.

The replays also check that OpenAI compaction stays in the stored native response alongside
the accompanying answer or tool call, with its encrypted payload preserved. Failed initial turns
retain a transcript snapshot in the report.

To capture a new run before SDK decoding:

```sh
cargo build
python3 examples/automatic-compaction/record_live.py --case claude-stream
```

The local proxy forwards credentials in memory and saves only request and response bodies.
New recordings use the `typed-` prefix so the original evidence remains intact.

The transcript stores one final canonical message alongside the whole native response. Automatic
compaction is handled by the LLM when building requests; all original native blocks remain stored. Use `session.transcript.entries()` and each record's `native()` accessor, or
`{:#?}` formatting, to inspect all reasoning, compaction, and server-tool blocks. Claude signed
on-demand compaction requires explicit removal of replaced context; this application exercises
threshold compaction only.


On-demand and custom compaction use the same generated agents and archive tool:

```sh
cargo run --bin on-demand -- claude
cargo run --bin on-demand -- claude --stream
cargo run --bin on-demand -- openai
cargo run --bin on-demand -- custom
```

`configs/on-demand.toml` configures the trigger, retained token budget, fixed-message selector,
and summarization instructions. Models remain in `configs/models.toml`: Haiku 5.5 for signed Claude
compaction, GPT-5 nano for Responses compaction, and Sonnet 5.5 for the custom summarizer
(`InstructionCompactor`). These commands use paid APIs. The recorded Haiku custom-summary
attempt preserved the facts in its summary but rejected them as fabricated afterward; its raw
evidence is retained in `custom-haiku-failure.json`. The Sonnet custom workflow passed.

The initial turn uses no compaction policy. Session continuation then configures the selected policy
with `session.with_compaction(...)` before recall generation and asks two recall questions. `on_demand.rs` checks the independently
supplied random facts, the tool-supplied archive revision, and every original stored native record
and UUID. It never inserts or edits transcript entries. `--report` saves native history and usage.

`tests/on_demand.rs` replays actual HTTP captures from
`crates/talin/tests/fixtures/on-demand-compaction/`. It matches complete count, compaction,
and generation requests, including signed payloads and retained suffixes. No model answer is
written by hand. The `openai-sdk-inputs` capture uses the SDK request and input types: retained
input-message IDs are omitted when the compacted window is sent back. The original `openai`
capture remains as historical evidence. Signed compaction content and the retained message
contents remain part of the outgoing request.

To record a new paid run after building `--bin on-demand`:

```sh
python3 examples/automatic-compaction/record_on_demand.py claude
python3 examples/automatic-compaction/record_on_demand.py openai
python3 examples/automatic-compaction/record_on_demand.py custom
```

The recorder forwards authentication headers in memory and records only HTTP bodies and request IDs.
