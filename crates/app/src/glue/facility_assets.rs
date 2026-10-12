use bevy::prelude::*;
use game_assets::{FacilityAssets as FacilityCollection, GameAssetsState};
use gameplay::facility::{FacilityAssets, Module};

pub(crate) struct FacilityAssetsPlugin;

impl Plugin for FacilityAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameAssetsState::Ready), fill_facility_assets);
    }
}

fn fill_facility_assets(collection: Res<FacilityCollection>, mut commands: Commands) {
    let modules = Module::ALL
        .iter()
        .map(|&module| {
            let handle = collection
                .module(stem(module))
                .unwrap_or_else(|| panic!("missing facility module: {}", stem(module)))
                .clone();
            (module, handle)
        })
        .collect();
    commands.insert_resource(FacilityAssets { modules });
}

fn stem(module: Module) -> &'static str {
    match module {
        Module::FloorTile => "floor_tile",
        Module::FloorTileMarked => "floor_tile_marked",
        Module::CeilingTile => "ceiling_tile",
        Module::Wall => "wall",
        Module::WallConduit => "wall_conduit",
        Module::WallDoorway => "wall_doorway",
        Module::DoorPanel => "door_panel",
        Module::CeilingLightCool => "ceiling_light_cool",
        Module::CeilingLightDead => "ceiling_light_dead",
        Module::CeilingLightAmber => "ceiling_light_amber",
        Module::WallLampRed => "wall_lamp_red",
        Module::ExitSign => "exit_sign",
        Module::WallVent => "wall_vent",
        Module::StorageCrate => "storage_crate",
        Module::StorageCrateOpen => "storage_crate_open",
        Module::SteelDrum => "steel_drum",
        Module::DrumSpilled => "drum_spilled",
        Module::ShelfUnit => "shelf_unit",
        Module::ShelfUnitAlt => "shelf_unit_alt",
        Module::ShelfUnitBins => "shelf_unit_bins",
        Module::ShelfUnitLow => "shelf_unit_low",
        Module::ShelfBinContents => "shelf_bin_contents",
        Module::Workbench => "workbench",
        Module::WorkIsland => "work_island",
        Module::FileCabinet => "file_cabinet",
        Module::FileCabinetOpen => "file_cabinet_open",
        Module::SignLabelBoilerRoom => "sign_label_boiler_room",
        Module::SignLabelStorage => "sign_label_storage",
        Module::SignLabelMaintenance => "sign_label_maintenance",
        Module::SignLabelUtility => "sign_label_utility",
        Module::SignLabelOffice => "sign_label_office",
        Module::SignLabelLab => "sign_label_lab",
        Module::SignLabelSecurity => "sign_label_security",
        Module::SignLabelPrep => "sign_label_prep",
        Module::SignLabelExit => "sign_label_exit",
        Module::SignArrow => "sign_arrow",
        Module::SignHanger => "sign_hanger",
        Module::BoilerUnit => "boiler_unit",
        Module::PipeManifold => "pipe_manifold",
        Module::ToolPegboard => "tool_pegboard",
        Module::Toolbox => "toolbox",
        Module::ConceptLocker => "concept_locker",
        Module::ConceptTable => "concept_table",
        Module::ConceptCrawlVent => "concept_crawl_vent",
        Module::VentGrille => "vent_grille",
        Module::ConceptContainmentTank => "concept_containment_tank",
        Module::LabConsole => "lab_console",
        Module::ClutterPapers => "clutter_papers",
        Module::ClutterTools => "clutter_tools",
        Module::ChairTipped => "chair_tipped",
        Module::PottedPlant => "potted_plant",
        Module::TraceClawMarks => "trace_claw_marks",
        Module::TraceDragMarks => "trace_drag_marks",
        Module::FusePanel => "fuse_panel",
        Module::FusePickup => "fuse_pickup",
    }
}
