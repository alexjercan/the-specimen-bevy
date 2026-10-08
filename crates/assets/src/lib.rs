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

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(path = "ui/input-prompts/T_F_Key_Alt.png")]
    pub interact_key: Handle<Image>,
    #[asset(path = "ui/fonts/SGr-IosevkaTerm-Medium.ttf")]
    pub font: Handle<Font>,
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
