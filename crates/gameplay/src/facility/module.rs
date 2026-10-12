use bevy::prelude::*;

use super::builder::LightEffect;

const COOL: Color = Color::linear_rgb(0.78, 0.88, 1.0);
const AMBER: Color = Color::linear_rgb(1.0, 0.58, 0.2);
const EXIT_GREEN: Color = Color::linear_rgb(0.12, 1.0, 0.35);
const FAULT_RED: Color = Color::linear_rgb(1.0, 0.06, 0.03);
const FIRE: Color = Color::linear_rgb(1.0, 0.35, 0.08);
const SPECIMEN: Color = Color::linear_rgb(0.3, 0.95, 0.85);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Module {
    FloorTile,
    FloorTileMarked,
    CeilingTile,
    Wall,
    WallConduit,
    WallDoorway,
    DoorPanel,
    CeilingLightCool,
    CeilingLightDead,
    CeilingLightAmber,
    WallLampRed,
    ExitSign,
    WallVent,
    StorageCrate,
    StorageCrateOpen,
    SteelDrum,
    DrumSpilled,
    ShelfUnit,
    ShelfUnitAlt,
    ShelfUnitBins,
    ShelfUnitLow,
    ShelfBinContents,
    Workbench,
    WorkIsland,
    FileCabinet,
    FileCabinetOpen,
    SignLabelBoilerRoom,
    SignLabelStorage,
    SignLabelMaintenance,
    SignLabelUtility,
    SignLabelOffice,
    SignLabelLab,
    SignLabelSecurity,
    SignLabelPrep,
    SignLabelExit,
    SignArrow,
    SignHanger,
    BoilerUnit,
    PipeManifold,
    ToolPegboard,
    Toolbox,
    ConceptLocker,
    ConceptTable,
    ConceptCrawlVent,
    VentGrille,
    ConceptContainmentTank,
    LabConsole,
    ClutterPapers,
    ClutterTools,
    ChairTipped,
    PottedPlant,
    TraceClawMarks,
    TraceDragMarks,
    FusePanel,
    FusePickup,
}

pub struct LightSpec {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub effect: Option<LightEffect>,
}

impl Module {
    pub const ALL: &[Module] = &[
        Self::FloorTile,
        Self::FloorTileMarked,
        Self::CeilingTile,
        Self::Wall,
        Self::WallConduit,
        Self::WallDoorway,
        Self::DoorPanel,
        Self::CeilingLightCool,
        Self::CeilingLightDead,
        Self::CeilingLightAmber,
        Self::WallLampRed,
        Self::ExitSign,
        Self::WallVent,
        Self::StorageCrate,
        Self::StorageCrateOpen,
        Self::SteelDrum,
        Self::DrumSpilled,
        Self::ShelfUnit,
        Self::ShelfUnitAlt,
        Self::ShelfUnitBins,
        Self::ShelfUnitLow,
        Self::ShelfBinContents,
        Self::Workbench,
        Self::WorkIsland,
        Self::FileCabinet,
        Self::FileCabinetOpen,
        Self::SignLabelBoilerRoom,
        Self::SignLabelStorage,
        Self::SignLabelMaintenance,
        Self::SignLabelUtility,
        Self::SignLabelOffice,
        Self::SignLabelLab,
        Self::SignLabelSecurity,
        Self::SignLabelPrep,
        Self::SignLabelExit,
        Self::SignArrow,
        Self::SignHanger,
        Self::BoilerUnit,
        Self::PipeManifold,
        Self::ToolPegboard,
        Self::Toolbox,
        Self::ConceptLocker,
        Self::ConceptTable,
        Self::ConceptCrawlVent,
        Self::VentGrille,
        Self::ConceptContainmentTank,
        Self::LabConsole,
        Self::ClutterPapers,
        Self::ClutterTools,
        Self::ChairTipped,
        Self::PottedPlant,
        Self::TraceClawMarks,
        Self::TraceDragMarks,
        Self::FusePanel,
        Self::FusePickup,
    ];

    pub fn footprint(self) -> Option<(Vec2, Vec2)> {
        Some(match self {
            Self::StorageCrate => (Vec2::new(-0.51, -0.415), Vec2::new(0.51, 0.41)),
            Self::StorageCrateOpen => (Vec2::new(-0.51, -0.435), Vec2::new(0.51, 0.8)),
            Self::SteelDrum => (Vec2::splat(-0.3), Vec2::splat(0.3)),
            Self::DrumSpilled => (Vec2::new(-0.41, -0.35), Vec2::new(0.75, 0.35)),
            Self::ShelfUnit | Self::ShelfUnitAlt => (Vec2::new(-0.9, -0.25), Vec2::new(0.9, 0.258)),
            Self::ShelfUnitBins => (Vec2::new(-0.9, -0.25), Vec2::new(0.9, 0.25)),
            Self::ShelfUnitLow => (Vec2::new(-0.6, -0.225), Vec2::new(0.6, 0.225)),
            Self::Workbench => (Vec2::new(-0.8, -0.35), Vec2::new(0.8, 0.35)),
            Self::FileCabinet => (Vec2::new(-0.38, -0.27), Vec2::new(0.38, 0.245)),
            Self::FileCabinetOpen => (Vec2::new(-0.38, -0.59), Vec2::new(0.38, 0.245)),
            Self::ConceptTable => (Vec2::new(-0.9, -0.45), Vec2::new(0.9, 0.46)),
            Self::ConceptLocker => (Vec2::new(-0.3, -0.4873), Vec2::new(0.3144, 0.3)),
            Self::WorkIsland => (Vec2::new(-0.95, -0.45), Vec2::new(0.95, 0.4529)),
            Self::LabConsole => (Vec2::new(-0.75, -0.35), Vec2::new(0.75, 0.35)),
            Self::BoilerUnit => (Vec2::new(-0.635, -0.7), Vec2::new(0.635, 0.635)),
            Self::ConceptContainmentTank => (Vec2::new(-0.7327, -0.9189), Vec2::new(0.702, 0.6687)),
            _ => return None,
        })
    }

    pub fn light_spec(self) -> Option<LightSpec> {
        let (position, color, intensity, range, effect) = match self {
            Self::CeilingLightCool => (Vec3::Y * 2.7, COOL, 90_000.0, 9.0, None),
            Self::CeilingLightAmber => (Vec3::Y * 2.7, AMBER, 140_000.0, 9.0, None),
            Self::ExitSign => (Vec3::NEG_Z * 0.2, EXIT_GREEN, 6_000.0, 5.0, None),
            Self::WallLampRed => (
                Vec3::NEG_Z * 0.25,
                FAULT_RED,
                60_000.0,
                5.0,
                Some(LightEffect::Pulse),
            ),
            Self::BoilerUnit => (
                Vec3::new(0.0, 0.5, -0.85),
                FIRE,
                45_000.0,
                7.0,
                Some(LightEffect::Flicker(-7.0)),
            ),
            Self::ConceptContainmentTank => (
                Vec3::new(0.0, 2.1, 0.0),
                SPECIMEN,
                12_000.0,
                4.5,
                Some(LightEffect::Flicker(0.0)),
            ),
            _ => return None,
        };
        Some(LightSpec {
            position,
            color,
            intensity,
            range,
            effect,
        })
    }

    pub fn mains_powered(self) -> bool {
        !matches!(self, Self::WallLampRed | Self::ExitSign | Self::BoilerUnit)
    }
}
