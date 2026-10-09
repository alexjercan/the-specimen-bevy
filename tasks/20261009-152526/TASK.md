# Rename project to the-specimen-bevy

- STATUS: CLOSED
- PRIORITY: 0
- TAGS: backlog

## User facts

- Rename the project from `horror-game-bevy` to `the-specimen-bevy`.
- This task records the rename; do not apply it as part of task creation.

## Delivery

- Audit project identity in Cargo package/binary/library names, Nix and CI configuration, scripts, documentation, and repository-path references.
- Update relevant names consistently while preserving working commands and internal crate APIs unless a rename is required.
- Coordinate any repository-directory or remote-repository rename before changing external paths.

## Verification

- Check remaining old-name references for intentional history or attribution.
- Run focused builds/tests and validate documented launch and packaging commands after the rename.

## Done when

- Project metadata and active references consistently use `the-specimen-bevy`; old-name references are intentional or removed, and affected commands work.
