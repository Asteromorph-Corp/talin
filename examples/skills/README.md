# `skills`

This example shows how to give an agent skills.
Skills are instructions and reference material it loads only when it needs them.

```sh
ANTHROPIC_API_KEY=... cargo run             # review a poem
cargo run -- --list                         # list the skills, without calling a model
```

The agent reviews a haiku.
The rules for counting syllables live in a skill, and not in the agent's prompt.

## A skill

A skill is a directory with a `SKILL.md` file:

```text
skills/haiku-reviewer/
├── SKILL.md              instructions, with a name and description in the header
└── references/
    └── syllables.md      material the skill points to
```

```markdown
---
name: haiku-reviewer
description: Reviews haiku for the 5-7-5 syllable pattern. Use when the user shares a haiku.
---
# Reviewing a haiku
...
See `references/syllables.md` for the list of words and their specified syllable counts.
```

The model is shown only each skill's name and description.
It reads the rest of `SKILL.md`, and the files it refers to, when the task calls for them, so a skill costs little context until it is used.

## Skill Configuration

List the skills in `agents.toml`:

```toml
[reviewer]
# ...
skills = ["haiku-reviewer"]
```

Skills are looked up by name in `$TALIN_SKILLS_HOME`, otherwise in `skills/` in the package directory under `cargo run`, otherwise in `skills/` in the working directory.

## Limitations

As of right now, the crate only lets the model read the skill's reference files as text.
The script files or other files in the skill directory cannot be used by the model.

## Next

- A skill in a larger program: [`assistant-cli`](../assistant-cli).
