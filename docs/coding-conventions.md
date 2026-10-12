# Coding Conventions

## Comments

- Do not add ordinary code comments or Rust doc comments.
- Use only `NOTE:`, `XXX:`, `WTF:` and plain `TODO:`. No project marker, no ticket ID.
- Check: `python3 scripts/check-comments.py` in `tests.sh`.
- A `grep -rn 'TODO:' crates src` belongs in the release checklist
  (`docs/tooling.md`, "Release checklist"), not in the checker.

## Data

- `Res<T>`, never `Option<Res<T>>` with a fallback. The owning plugin inits
  the resource. Tests add that plugin. No mirror resources.
  Check: `gate option_res` in `scripts/check-architecture.sh`.
  Check: `gate test_mirror_resources` (tests add `FacilityPowerPlugin`, never
  a hand-made `FacilityPower`).
- Assert behavior, never plugin presence. No `is_plugin_added`.
  Check: `gate is_plugin_added`.
- Two exclusive bools are an enum. `Frontend::Headless | Windowed { menu }`,
  not `headless: bool, menu: bool`. Also `StaminaState`, `ThrowPhase` and
  `GamePreset::{Normal, Custom(PresetRules), Insane}`, where only Custom
  carries its own rules.
- Config and runtime are two components or resources, not one struct with
  both. `IntermittentSound` (kind, offset, range, interval, variation) and
  `IntermittentTimer` (remaining) are the example.
- A unitless number is a newtype. `LightScale` multiplies a light
  intensity; `AmbientNits` is an ambient brightness in cd/m^2.
- An index into a fixed list is an enum. `CustomLevel::{Low, Medium, High}`
  picks one of three monster values. It is saved as 0, 1 or 2, so older
  settings files still load.
- A string matched twice is an enum. `Module::Workbench`, not
  `Prop("workbench")`.
- Modules are files or directories found by name. No `#[path]` includes.
  Check: `gate path_includes`.
- Exactly one entity is `Single<D, F>`, not `Query` plus `iter().next()`.
  The player, the camera, one HUD node. The system skips when the entity
  is absent, which is the wanted behavior for a view.
- A view shows how things are by reading components in `Update`. A view
  reacts to what happened by reading a gameplay message. Neither lives in
  glue.

## Plugins

- Every plugin owns what it registers: its resources, its message types, its
  states.
- Logic plugins run headless. View plugins are added only in windowed mode.
- Gameplay never loads assets by path. See `docs/architecture.md`.

## Work habits

- State the intended result before changing code. List deletions first.
- Change the owning interface first, then its callers.
- Examples never assert. Tests under `tests/e2e` assert. See `docs/testing.md`.
