# The Specimen

The Specimen is a horror game written in Rust using the Bevy game engine.
This is a `nix` based project: run every command inside `nix develop`.

## Agent instructions

- Use session todos for user requests. Use Tatr (`tasks/`) only for
  requested durable work.

## Repository layout

- `crates` contains first party libraries for the game
- `src` contains the entry point for the game
- `tests` contains CLI tests and asserting e2e scenarios
- `examples` contains screenshot and manual playtest tools
- `playtest` contains the stdio host scripts
- `art` contains research materials and art files that are proposed for the game
- `assets` contains the game assets
- `docs` contains the project rules
- `scripts` contains tools and scripts for helping with running commands

## Docs

- `docs/architecture.md`: layers, allowed edges, features, glue, messages.
- `docs/coding-conventions.md`: comment policy and data rules.
- `docs/testing.md`: what each kind of test proves and where it lives.
- `docs/tooling.md`: game flags, stdio, video, profiling, release builds.
- `docs/assets.md`: art to assets flow and provenance.
- `docs/achievements.md`: the achievement list.

## Tools

| Command | Use |
| --- | --- |
| `nix develop --command ./tests.sh --smoke` | Fast check before reporting work. |
| `nix develop --command ./tests.sh --full` | What CI runs. |
| `scripts/check-architecture.sh` | Architecture gates, also part of `tests.sh`. |
| `cargo run -- --headless --seed 1` | Bounded logic-only run. |
| `cargo run -- --stdio` | Drive the game over stdin and stdout. |
| `tatr new`, `tatr ls`, `tatr edit` | Durable tracked work under `tasks/`. |
