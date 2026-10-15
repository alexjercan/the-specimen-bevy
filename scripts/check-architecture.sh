#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

gates=(
  option_res
  is_plugin_added
  path_includes
  gameplay_handles
  gameplay_asset_paths
  gameplay_manifest
  gameplay_input_types
  glue_config_only
  power_state_scope
  requires_power
  test_mirror_resources
  message_owners
  audio_manifest
  frame_rate_owner
  frontend_assets
  gameplay_render_types
  app_settings_sync
)

fail() {
  echo "gate $1: $2" >&2
  return 1
}

grep_rs() {
  grep -rn --include='*.rs' "$@" || true
}

gate_option_res() {
  local hits
  hits=$(grep_rs -F 'Option<Res<' crates src)
  [[ -z "$hits" ]] || fail option_res "use Res<T>; the owning plugin inits it"$'\n'"$hits"
}

gate_is_plugin_added() {
  local hits
  hits=$(grep_rs -F 'is_plugin_added' crates src tests examples)
  [[ -z "$hits" ]] || fail is_plugin_added "assert behavior, not plugin presence"$'\n'"$hits"
}

gate_path_includes() {
  local hits
  hits=$(grep_rs -F '#[path' crates src tests examples)
  [[ -z "$hits" ]] || fail path_includes "no #[path] module includes"$'\n'"$hits"
}

gate_gameplay_handles() {
  local hits
  hits=$(grep -rn --include='*.rs' -F 'Handle<' crates/gameplay/src \
    | grep -v -E '/(view|sounds|assets)\.rs:' || true)
  [[ -z "$hits" ]] || fail gameplay_handles "Handle<T> in gameplay only in view.rs, sounds.rs, assets.rs"$'\n'"$hits"
}

gate_gameplay_asset_paths() {
  local hits
  hits=$(grep -rn --include='*.rs' -E 'bevy_asset_loader|asset_server\.load|"[^"]+\.(glb|gltf|ogg|wav|png|ttf)' crates/gameplay/src || true)
  [[ -z "$hits" ]] || fail gameplay_asset_paths "gameplay never loads by path; app glue fills feature assets"$'\n'"$hits"
}

gate_gameplay_manifest() {
  local hits
  hits=$(grep -n -E 'game_assets' crates/gameplay/Cargo.toml || true)
  [[ -z "$hits" ]] || fail gameplay_manifest "gameplay must not depend on game_assets"$'\n'"$hits"
}

gate_gameplay_input_types() {
  local hits
  hits=$(grep -rn --include='*.rs' -E '^\s*pub .*(bevy_enhanced_input|Binding\b)' crates/gameplay/src || true)
  [[ -z "$hits" ]] || fail gameplay_input_types "gameplay pub items wrap input library types"$'\n'"$hits"
}

gate_glue_config_only() {
  local hits
  hits=$(grep -rn --include='*.rs' -E '\bconst |== "|\bRng\b|\brand\b|\bUpdate\b|\bFixedUpdate\b|Changed<' crates/app/src/glue || true)
  [[ -z "$hits" ]] || fail glue_config_only "glue is config only: Startup or OnEnter, no constants, compares, RNG or change detection"$'\n'"$hits"
}

gate_power_state_scope() {
  local hits
  hits=$(grep -rn --include='*.rs' -E 'PowerState|FacilityPower' crates src examples tests \
    | grep -v -E '^(crates/gameplay/|crates/app/src/playable/|crates/debug/|crates/[^/]+/tests/|tests/)' || true)
  [[ -z "$hits" ]] || fail power_state_scope "PowerState and FacilityPower are read only under crates/gameplay, the stdio snapshot, the debug decorator and tests"$'\n'"$hits"
}

gate_requires_power() {
  local hits
  hits=$(grep_rs -F 'requires_power' crates src examples tests)
  [[ -z "$hits" ]] || fail requires_power "power dependence is the MainsPowered tag, not a kind method"$'\n'"$hits"
}

gate_test_mirror_resources() {
  local hits
  hits=$(grep -rn --include='*.rs' -E 'init_resource::<FacilityPower|insert_resource\(FacilityPower' crates tests examples \
    | grep -v '^crates/gameplay/src/' || true)
  [[ -z "$hits" ]] || fail test_mirror_resources "tests add FacilityPowerPlugin, not a hand-made FacilityPower"$'\n'"$hits"
}

gate_message_owners() {
  local dupes
  dupes=$(grep -rhoE --include='*.rs' 'add_message::<[A-Za-z0-9_:]+>' crates src | sort | uniq -d || true)
  [[ -z "$dupes" ]] || fail message_owners "one add_message per message type"$'\n'"$dupes"
}

gate_audio_manifest() {
  local hits
  hits=$(grep -n -E 'game_assets|game_settings' crates/audio/Cargo.toml || true)
  [[ -z "$hits" ]] || fail audio_manifest "game_audio is a bevy-only bus"$'\n'"$hits"
}

gate_frame_rate_owner() {
  local crates_with_60
  crates_with_60=$(grep -rlE --include='*.rs' '\b60(\.0)?\b' crates/*/src src | cut -d/ -f1-2 | sort -u || true)
  [[ $(echo "$crates_with_60" | grep -c . ) -le 1 ]] || {
    fail frame_rate_owner "one owner for the frame step"$'\n'"$crates_with_60"
    return 1
  }
  local steps
  steps=$(grep -rnE --include='*.rs' '1_000_000_000 */|16_66[67]|1\.0 / (30|60)(\.0)?\b' crates/*/src src tests examples \
    | grep -v '^crates/app/src/lib.rs:' || true)
  [[ -z "$steps" ]] || fail frame_rate_owner "frame step literals live only in game_app::FrameStep"$'\n'"$steps"
}

gate_frontend_assets() {
  local hits
  hits=$(grep_rs -F 'game_assets' crates/frontend/src)
  hits+=$(sed -n '/^\[dependencies\]/,/^\[/p' crates/frontend/Cargo.toml | grep -n 'game_assets' || true)
  [[ -z "$hits" ]] || fail frontend_assets "frontend reads MenuAssets filled by app glue, not game_assets"$'\n'"$hits"
}

gate_gameplay_render_types() {
  local hits
  hits=$(grep -rn --include='*.rs' -E '\b(Mesh3d|MeshMaterial3d|PointLight|SpotLight|DirectionalLight|Camera3d|Window)\b' crates/gameplay/src \
    | grep -v -E '/(view|[a-z_]+_view|hud|[a-z_]+_hud)\.rs:' || true)
  [[ -z "$hits" ]] || fail gameplay_render_types "render types in gameplay only in view.rs, *_view.rs, hud.rs, *_hud.rs"$'\n'"$hits"
}

gate_app_settings_sync() {
  local hits
  hits=$(grep -n -E 'resource_changed::<GameSettings>|Changed<GameSettings>|is_changed\(' crates/app/src/lib.rs || true)
  [[ -z "$hits" ]] || fail app_settings_sync "the settings panel writes live changes; app copies settings once at build"$'\n'"$hits"
}

selected=("$@")
if [[ -z "${1:-}" ]]; then
  selected=("${gates[@]}")
fi
status=0
for gate in "${selected[@]}"; do
  if "gate_$gate"; then
    echo "gate $gate: ok"
  else
    status=1
  fi
done
exit "$status"
