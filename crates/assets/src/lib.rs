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
            "facility/modules/fuse_panel.glb#Scene0"
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

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameAssetsState>()
            .add_loading_state(
                LoadingState::new(GameAssetsState::Loading)
                    .continue_to_state(GameAssetsState::Ready)
                    .on_failure_continue_to_state(GameAssetsState::Failed)
                    .load_collection::<FacilityAssets>(),
            )
            .add_systems(OnEnter(GameAssetsState::Ready), report_loaded_assets)
            .add_systems(OnEnter(GameAssetsState::Failed), report_failed_assets);
    }
}

fn report_loaded_assets(assets: Res<FacilityAssets>) {
    info!("facility assets ready: {} modules", assets.modules.len());
}

fn report_failed_assets() {
    error!("facility asset loading failed; check the asset-loader errors for the missing or invalid GLB under assets/facility/modules/");
}
