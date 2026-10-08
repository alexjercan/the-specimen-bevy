use bevy::{platform::collections::HashMap, prelude::*, world_serialization::WorldAsset};
use bevy_asset_loader::{mapped::AssetFileStem, prelude::*};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameAssetsState {
    #[default]
    Loading,
    Ready,
    Failed,
}

#[derive(AssetCollection, Resource)]
pub struct FacilityAssets {
    #[asset(
        paths(
            "facility/modules/floor_tile.glb#Scene0",
            "facility/modules/floor_tile_marked.glb#Scene0",
            "facility/modules/ceiling_tile.glb#Scene0",
            "facility/modules/wall.glb#Scene0",
            "facility/modules/wall_conduit.glb#Scene0",
            "facility/modules/wall_doorway.glb#Scene0",
            "facility/modules/door_panel.glb#Scene0",
            "facility/modules/wall_post.glb#Scene0",
            "facility/modules/ceiling_light_cool.glb#Scene0",
            "facility/modules/ceiling_light_dead.glb#Scene0",
            "facility/modules/ceiling_light_amber.glb#Scene0",
            "facility/modules/wall_lamp_red.glb#Scene0",
            "facility/modules/exit_sign.glb#Scene0",
            "facility/modules/wall_vent.glb#Scene0",
            "facility/modules/storage_crate.glb#Scene0",
            "facility/modules/steel_drum.glb#Scene0",
            "facility/modules/shelf_unit.glb#Scene0",
            "facility/modules/workbench.glb#Scene0",
            "facility/modules/route_line_orange.glb#Scene0",
            "facility/modules/route_line_blue.glb#Scene0",
            "facility/modules/route_line_green.glb#Scene0",
            "facility/modules/sign_label_boiler_room.glb#Scene0",
            "facility/modules/sign_label_storage.glb#Scene0",
            "facility/modules/sign_label_maintenance.glb#Scene0",
            "facility/modules/sign_label_utility.glb#Scene0",
            "facility/modules/sign_label_office.glb#Scene0",
            "facility/modules/sign_label_lab.glb#Scene0",
            "facility/modules/sign_label_security.glb#Scene0",
            "facility/modules/sign_label_prep.glb#Scene0",
            "facility/modules/sign_label_exit.glb#Scene0",
            "facility/modules/sign_arrow.glb#Scene0",
            "facility/modules/sign_hanger.glb#Scene0",
            "facility/modules/boiler_unit.glb#Scene0",
            "facility/modules/pipe_manifold.glb#Scene0",
            "facility/modules/tool_pegboard.glb#Scene0",
            "facility/modules/shelf_unit_low.glb#Scene0",
            "facility/modules/shelf_unit_bins.glb#Scene0",
            "facility/modules/concept_locker.glb#Scene0",
            "facility/modules/concept_table.glb#Scene0",
            "facility/modules/concept_crawl_vent.glb#Scene0",
            "facility/modules/vent_grille.glb#Scene0",
            "facility/modules/concept_containment_tank.glb#Scene0",
            "facility/modules/lab_console.glb#Scene0",
            "facility/modules/work_island.glb#Scene0",
            "facility/modules/clutter_papers.glb#Scene0",
            "facility/modules/clutter_tools.glb#Scene0",
            "facility/modules/chair_tipped.glb#Scene0",
            "facility/modules/drum_spilled.glb#Scene0",
            "facility/modules/trace_claw_marks.glb#Scene0",
            "facility/modules/trace_drag_marks.glb#Scene0",
            "facility/modules/fuse_panel.glb#Scene0",
            "facility/modules/fuse_pickup.glb#Scene0"
        ),
        collection(mapped, typed)
    )]
    pub modules: HashMap<AssetFileStem, Handle<WorldAsset>>,
}

impl FacilityAssets {
    pub fn module(&self, name: &str) -> Option<&Handle<WorldAsset>> {
        self.modules.get(name)
    }
}

pub const KEY_GLYPHS: &[(&str, &str)] = &[
    ("KeyA", "T_A_Key_Alt"),
    ("KeyB", "T_B_Key_Alt"),
    ("KeyC", "T_C_Key_Alt"),
    ("KeyD", "T_D_Key_Alt"),
    ("KeyE", "T_E_Key_Alt"),
    ("KeyF", "T_F_Key_Alt"),
    ("KeyG", "T_G_Key_Alt"),
    ("KeyH", "T_H_Key_Alt"),
    ("KeyI", "T_I_Key_Alt"),
    ("KeyJ", "T_J_Key_Alt"),
    ("KeyK", "T_K_Key_Alt"),
    ("KeyL", "T_L_Key_Alt"),
    ("KeyM", "T_M_Key_Alt"),
    ("KeyN", "T_N_Key_Alt"),
    ("KeyO", "T_O_Key_Alt"),
    ("KeyP", "T_P_Key_Alt"),
    ("KeyQ", "T_Q_Key_Alt"),
    ("KeyR", "T_R_Key_Alt"),
    ("KeyS", "T_S_Key_Alt"),
    ("KeyT", "T_T_Key_Alt"),
    ("KeyU", "T_U_Key_Alt"),
    ("KeyV", "T_V_Key_Alt"),
    ("KeyW", "T_W_Key_Alt"),
    ("KeyX", "T_X_Key_Alt"),
    ("KeyY", "T_Y_Key_Alt"),
    ("KeyZ", "T_Z_Key_Alt"),
    ("Digit0", "T_0_Key_Alt"),
    ("Digit1", "T_1_Key_Alt"),
    ("Digit2", "T_2_Key_Alt"),
    ("Digit3", "T_3_Key_Alt"),
    ("Digit4", "T_3_Key_Alt-1"),
    ("Digit5", "T_5_Key_Alt"),
    ("Digit6", "T_6_Key_Alt"),
    ("Digit7", "T_7_Key_Alt"),
    ("Digit8", "T_8_Key_Alt"),
    ("Digit9", "T_9_Key_Alt"),
    ("F1", "T_F1_Key_Alt"),
    ("F2", "T_F2_Key_Alt"),
    ("F3", "T_F3_Key_Alt"),
    ("F4", "T_F4_Key_Alt"),
    ("F5", "T_F5_Key_Alt"),
    ("F6", "T_F6_Key_Alt"),
    ("F7", "T_F7_Key_Alt"),
    ("F8", "T_F8_Key_Alt"),
    ("F9", "T_F9_Key_Alt"),
    ("F10", "T_F10_Key_Alt"),
    ("F11", "T_F11_Key_Alt"),
    ("F12", "T_F12_Key_Alt"),
    ("ArrowUp", "T_Up_Key_Alt"),
    ("ArrowDown", "T_Down_Key_Alt"),
    ("ArrowLeft", "T_Left_Key_Alt"),
    ("ArrowRight", "T_Right_Key_Alt"),
    ("Space", "T_Space_Key_Alt"),
    ("Tab", "T_Tab_Key_Alt"),
    ("Enter", "T_Enter_Key_Alt"),
    ("Escape", "T_Esc_Key_Alt"),
    ("Backspace", "T_BackSpace_Key_Alt"),
    ("CapsLock", "T_CapsLock_Key_Alt"),
    ("NumLock", "T_NumLock_Key_Alt"),
    ("PrintScreen", "T_PrtScrn_Key_Alt"),
    ("Insert", "T_Ins_Key_Alt"),
    ("Delete", "T_Del_Key_Alt"),
    ("Home", "T_Home_Key_Alt"),
    ("End", "T_End_Key_Alt"),
    ("PageUp", "T_PageUp_Key_Alt"),
    ("PageDown", "T_PageDown_Key_Alt"),
    ("ControlLeft", "T_Crtl_Key_Alt"),
    ("ControlRight", "T_Crtl_Key_Alt"),
    ("ShiftLeft", "T_Shift_Key_Alt"),
    ("ShiftRight", "T_Shift_Key_Alt"),
    ("AltLeft", "T_Alt_Key_Alt"),
    ("AltRight", "T_Alt_Key_Alt"),
    ("BracketLeft", "T_Brackets_L_Key_Alt"),
    ("BracketRight", "T_Brackets_R_Key_Alt"),
    ("Backquote", "T_Tilde_Key_Alt"),
    ("Minus", "T_Minus_Key_Alt"),
    ("Slash", "T_Slash_Key_Alt"),
    ("Semicolon", "T_Semicolon_Key_Alt"),
    ("Quote", "T_Quotation_Key_Alt"),
];

pub fn key_glyph_stem(key: &str) -> Option<&'static str> {
    KEY_GLYPHS
        .iter()
        .find_map(|(name, stem)| (*name == key).then_some(*stem))
}

#[derive(AssetCollection, Resource, Default)]
pub struct UiAssets {
    #[asset(
        paths(
            "ui/input-prompts/T_0_Key_Alt.png",
            "ui/input-prompts/T_1_Key_Alt.png",
            "ui/input-prompts/T_2_Key_Alt.png",
            "ui/input-prompts/T_3_Key_Alt.png",
            "ui/input-prompts/T_3_Key_Alt-1.png",
            "ui/input-prompts/T_5_Key_Alt.png",
            "ui/input-prompts/T_6_Key_Alt.png",
            "ui/input-prompts/T_7_Key_Alt.png",
            "ui/input-prompts/T_8_Key_Alt.png",
            "ui/input-prompts/T_9_Key_Alt.png",
            "ui/input-prompts/T_A_Key_Alt.png",
            "ui/input-prompts/T_Alt_Key_Alt.png",
            "ui/input-prompts/T_B_Key_Alt.png",
            "ui/input-prompts/T_BackSpace_Key_Alt.png",
            "ui/input-prompts/T_Brackets_L_Key_Alt.png",
            "ui/input-prompts/T_Brackets_R_Key_Alt.png",
            "ui/input-prompts/T_C_Key_Alt.png",
            "ui/input-prompts/T_CapsLock_Key_Alt.png",
            "ui/input-prompts/T_Crtl_Key_Alt.png",
            "ui/input-prompts/T_D_Key_Alt.png",
            "ui/input-prompts/T_Del_Key_Alt.png",
            "ui/input-prompts/T_Down_Key_Alt.png",
            "ui/input-prompts/T_E_Key_Alt.png",
            "ui/input-prompts/T_End_Key_Alt.png",
            "ui/input-prompts/T_Enter_Key_Alt.png",
            "ui/input-prompts/T_Esc_Key_Alt.png",
            "ui/input-prompts/T_F10_Key_Alt.png",
            "ui/input-prompts/T_F11_Key_Alt.png",
            "ui/input-prompts/T_F12_Key_Alt.png",
            "ui/input-prompts/T_F1_Key_Alt.png",
            "ui/input-prompts/T_F2_Key_Alt.png",
            "ui/input-prompts/T_F3_Key_Alt.png",
            "ui/input-prompts/T_F4_Key_Alt.png",
            "ui/input-prompts/T_F5_Key_Alt.png",
            "ui/input-prompts/T_F6_Key_Alt.png",
            "ui/input-prompts/T_F7_Key_Alt.png",
            "ui/input-prompts/T_F8_Key_Alt.png",
            "ui/input-prompts/T_F9_Key_Alt.png",
            "ui/input-prompts/T_F_Key_Alt.png",
            "ui/input-prompts/T_G_Key_Alt.png",
            "ui/input-prompts/T_H_Key_Alt.png",
            "ui/input-prompts/T_Home_Key_Alt.png",
            "ui/input-prompts/T_I_Key_Alt.png",
            "ui/input-prompts/T_Ins_Key_Alt.png",
            "ui/input-prompts/T_J_Key_Alt.png",
            "ui/input-prompts/T_K_Key_Alt.png",
            "ui/input-prompts/T_L_Key_Alt.png",
            "ui/input-prompts/T_Left_Key_Alt.png",
            "ui/input-prompts/T_M_Key_Alt.png",
            "ui/input-prompts/T_Minus_Key_Alt.png",
            "ui/input-prompts/T_N_Key_Alt.png",
            "ui/input-prompts/T_NumLock_Key_Alt.png",
            "ui/input-prompts/T_O_Key_Alt.png",
            "ui/input-prompts/T_P_Key_Alt.png",
            "ui/input-prompts/T_PageDown_Key_Alt.png",
            "ui/input-prompts/T_PageUp_Key_Alt.png",
            "ui/input-prompts/T_PrtScrn_Key_Alt.png",
            "ui/input-prompts/T_Q_Key_Alt.png",
            "ui/input-prompts/T_Quotation_Key_Alt.png",
            "ui/input-prompts/T_R_Key_Alt.png",
            "ui/input-prompts/T_Right_Key_Alt.png",
            "ui/input-prompts/T_S_Key_Alt.png",
            "ui/input-prompts/T_Semicolon_Key_Alt.png",
            "ui/input-prompts/T_Shift_Key_Alt.png",
            "ui/input-prompts/T_Slash_Key_Alt.png",
            "ui/input-prompts/T_Space_Key_Alt.png",
            "ui/input-prompts/T_T_Key_Alt.png",
            "ui/input-prompts/T_Tab_Key_Alt.png",
            "ui/input-prompts/T_Tilde_Key_Alt.png",
            "ui/input-prompts/T_U_Key_Alt.png",
            "ui/input-prompts/T_Up_Key_Alt.png",
            "ui/input-prompts/T_V_Key_Alt.png",
            "ui/input-prompts/T_W_Key_Alt.png",
            "ui/input-prompts/T_X_Key_Alt.png",
            "ui/input-prompts/T_Y_Key_Alt.png",
            "ui/input-prompts/T_Z_Key_Alt.png"
        ),
        collection(mapped, typed)
    )]
    pub key_glyphs: HashMap<AssetFileStem, Handle<Image>>,
    #[asset(path = "ui/fonts/SGr-IosevkaTerm-Medium.ttf")]
    pub font: Handle<Font>,
}

impl UiAssets {
    pub fn key_glyph(&self, key: &str) -> Option<Handle<Image>> {
        self.key_glyphs.get(key_glyph_stem(key)?).cloned()
    }
}

#[derive(AssetCollection, Resource)]
pub struct SoundAssets {
    #[asset(path = "sounds/amb/furnace/burning.wav")]
    pub furnace: Handle<AudioSource>,
    #[asset(path = "sounds/amb/water/faucet.wav")]
    pub faucet: Handle<AudioSource>,
    #[asset(path = "sounds/amb/vent/wind.wav")]
    pub vent_wind: Handle<AudioSource>,
    #[asset(path = "sounds/amb/roomtone/plain.wav")]
    pub roomtone: Handle<AudioSource>,
    #[asset(path = "sounds/amb/pressure/low.wav")]
    pub low_pressure: Handle<AudioSource>,
    #[asset(path = "sounds/amb/roomtone/conduit.wav")]
    pub conduit_roomtone: Handle<AudioSource>,
    #[asset(path = "sounds/amb/boiler/tick.wav")]
    pub boiler_tick: Handle<AudioSource>,
    #[asset(path = "sounds/amb/boiler/power-down.wav")]
    pub power_down: Handle<AudioSource>,
    #[asset(path = "sounds/amb/boiler/switch-off-cleytonkauffman.wav")]
    pub breaker_trip: Handle<AudioSource>,
    #[asset(path = "sounds/amb/boiler/reset.wav")]
    pub boiler_reset: Handle<AudioSource>,
    #[asset(path = "sounds/amb/boiler/restart.wav")]
    pub boiler_restart: Handle<AudioSource>,
    #[asset(path = "sounds/amb/vent/hvac.wav")]
    pub vent_hvac: Handle<AudioSource>,
    #[asset(path = "sounds/amb/tank/hum.wav")]
    pub tank_hum: Handle<AudioSource>,
    #[asset(path = "sounds/amb/light/cool-buzz.wav")]
    pub cool_buzz: Handle<AudioSource>,
    #[asset(path = "sounds/door/unlatch.wav")]
    pub door_unlatch: Handle<AudioSource>,
    #[asset(path = "sounds/door/swing-open.wav")]
    pub door_swing: Handle<AudioSource>,
    #[asset(path = "sounds/door/shut.wav")]
    pub door_shut: Handle<AudioSource>,
    #[asset(path = "sounds/door/locked-rattle.wav")]
    pub door_locked: Handle<AudioSource>,
    #[asset(path = "sounds/hiding/locker/open.wav")]
    pub locker_open: Handle<AudioSource>,
    #[asset(path = "sounds/hiding/locker/close.wav")]
    pub locker_close: Handle<AudioSource>,
    #[asset(path = "sounds/hiding/table/enter.wav")]
    pub table_enter: Handle<AudioSource>,
    #[asset(path = "sounds/hiding/table/leave.wav")]
    pub table_leave: Handle<AudioSource>,
    #[asset(path = "sounds/ui/fuse/slot-1.wav")]
    pub fuse_slot_1: Handle<AudioSource>,
    #[asset(path = "sounds/ui/fuse/slot-2.wav")]
    pub fuse_slot_2: Handle<AudioSource>,
    #[asset(path = "sounds/ui/fuse/slot-3.wav")]
    pub fuse_slot_3: Handle<AudioSource>,
    #[asset(path = "sounds/ui/fuse/complete.wav")]
    pub fuse_complete: Handle<AudioSource>,
    #[asset(path = "sounds/flashlight/click-ralph0o7.ogg")]
    pub flashlight_click: Handle<AudioSource>,
    #[asset(path = "sounds/self/breathing-tired-mikeask.wav")]
    pub sprint_exhausted: Handle<AudioSource>,
    #[asset(path = "sounds/step/subway/subway-step-a.ogg")]
    pub step_01: Handle<AudioSource>,
    #[asset(path = "sounds/step/subway/subway-step-b.ogg")]
    pub step_02: Handle<AudioSource>,
    #[asset(path = "sounds/step/subway/subway-step-c.ogg")]
    pub step_04: Handle<AudioSource>,
    #[asset(path = "sounds/ui/back.wav")]
    pub ui_back: Handle<AudioSource>,
    #[asset(path = "sounds/ui/confirm.wav")]
    pub ui_confirm: Handle<AudioSource>,
    #[asset(path = "sounds/ui/denied.wav")]
    pub ui_denied: Handle<AudioSource>,
    #[asset(path = "sounds/ui/focus.wav")]
    pub ui_focus: Handle<AudioSource>,
    #[asset(path = "sounds/ui/hover.wav")]
    pub ui_hover: Handle<AudioSource>,
    #[asset(path = "sounds/ui/pause.wav")]
    pub ui_pause: Handle<AudioSource>,
    #[asset(path = "sounds/ui/press.wav")]
    pub ui_press: Handle<AudioSource>,
    #[asset(path = "sounds/ui/resume.wav")]
    pub ui_resume: Handle<AudioSource>,
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameAssetsState>()
            .add_loading_state(
                LoadingState::new(GameAssetsState::Loading)
                    .continue_to_state(GameAssetsState::Ready)
                    .on_failure_continue_to_state(GameAssetsState::Failed)
                    .load_collection::<FacilityAssets>()
                    .load_collection::<UiAssets>()
                    .load_collection::<SoundAssets>(),
            )
            .add_systems(OnEnter(GameAssetsState::Ready), report_loaded_assets)
            .add_systems(OnEnter(GameAssetsState::Failed), report_failed_assets);
    }
}

fn report_loaded_assets(assets: Res<FacilityAssets>, _ui: Res<UiAssets>) {
    info!("facility assets ready: {} modules", assets.modules.len());
}

fn report_failed_assets() {
    error!(
        "facility asset loading failed; check the asset-loader errors for the missing or invalid asset under assets/"
    );
}
