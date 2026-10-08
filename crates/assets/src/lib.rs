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
    #[asset(path = "sounds/amb/boiler/rumble.wav")]
    pub boiler: Handle<AudioSource>,
    #[asset(path = "sounds/amb/roomtone.wav")]
    pub roomtone: Handle<AudioSource>,
    #[asset(path = "sounds/amb/roomtone/conduit.wav")]
    pub conduit_roomtone: Handle<AudioSource>,
    #[asset(path = "sounds/amb/boiler/tick/01.wav")]
    pub boiler_tick: Handle<AudioSource>,
    #[asset(path = "sounds/amb/vent/hvac/01.wav")]
    pub vent_hvac: Handle<AudioSource>,
    #[asset(path = "sounds/amb/tank/hum.wav")]
    pub tank_hum: Handle<AudioSource>,
    #[asset(path = "sounds/amb/light/buzz/cool-low.wav")]
    pub cool_buzz: Handle<AudioSource>,
    #[asset(path = "sounds/door/unlatch/01.wav")]
    pub door_unlatch: Handle<AudioSource>,
    #[asset(path = "sounds/door/swing/open/01.wav")]
    pub door_swing: Handle<AudioSource>,
    #[asset(path = "sounds/door/shut/01.wav")]
    pub door_shut: Handle<AudioSource>,
    #[asset(path = "sounds/hiding/locker/open.wav")]
    pub locker_open: Handle<AudioSource>,
    #[asset(path = "sounds/hiding/locker/close.wav")]
    pub locker_close: Handle<AudioSource>,
    #[asset(path = "sounds/hiding/table/enter.wav")]
    pub table_enter: Handle<AudioSource>,
    #[asset(path = "sounds/hiding/table/leave.wav")]
    pub table_leave: Handle<AudioSource>,
    #[asset(path = "sounds/fuse/pickup/01.wav")]
    pub fuse_pickup: Handle<AudioSource>,
    #[asset(path = "sounds/panel/install/01.wav")]
    pub panel_install: Handle<AudioSource>,
    #[asset(path = "sounds/step/subway/01.ogg")]
    pub step_01: Handle<AudioSource>,
    #[asset(path = "sounds/step/subway/02.ogg")]
    pub step_02: Handle<AudioSource>,
    #[asset(path = "sounds/step/subway/04.ogg")]
    pub step_04: Handle<AudioSource>,
    #[asset(path = "sounds/ui/back/01.wav")]
    pub ui_back: Handle<AudioSource>,
    #[asset(path = "sounds/ui/confirm/01.wav")]
    pub ui_confirm: Handle<AudioSource>,
    #[asset(path = "sounds/ui/denied/01.wav")]
    pub ui_denied: Handle<AudioSource>,
    #[asset(path = "sounds/ui/focus/01.wav")]
    pub ui_focus: Handle<AudioSource>,
    #[asset(path = "sounds/ui/hover/01.wav")]
    pub ui_hover: Handle<AudioSource>,
    #[asset(path = "sounds/ui/pause/01.wav")]
    pub ui_pause: Handle<AudioSource>,
    #[asset(path = "sounds/ui/press/01.wav")]
    pub ui_press: Handle<AudioSource>,
    #[asset(path = "sounds/ui/resume/01.wav")]
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
    error!("facility asset loading failed; check the asset-loader errors for the missing or invalid asset under assets/");
}
