use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    text::LineBreak,
};
use game_assets::UiAssets;
use game_ui::{menu_button, text, theme};
use gameplay::levels::build_closed_exit_cinematic;

use super::{cinematic, release_cursor, GameState, MenuAction};

pub(super) const HUMAN_DEER_CREDIT: &str = "This work is based on \"The Human Deer\" (https://sketchfab.com/3d-models/the-human-deer-7644694337404bb18eb68e6b637740a1) by ceeleste (https://sketchfab.com/ceeleste) licensed under CC-BY-4.0 (http://creativecommons.org/licenses/by/4.0/)";
pub(super) const ROLL_SPEED: f32 = 32.0;
const KEY_SCROLL_SPEED: f32 = 320.0;
const LINE_SCROLL: f32 = 40.0;
const START_LEAD: f32 = 0.85;
const COLUMN_WIDTH: f32 = 760.0;
const BUTTON_GAP: f32 = 12.0;

pub(super) struct CreditsSection {
    pub(super) heading: &'static str,
    pub(super) lines: &'static [&'static str],
}

pub(super) const SECTIONS: &[CreditsSection] = &[
    CreditsSection {
        heading: "Development",
        lines: &["Alex Jercan"],
    },
    CreditsSection {
        heading: "Monster",
        lines: &[
            HUMAN_DEER_CREDIT,
            "Changes: WALK, CHASE, and ATTACK animations were added and IDLE was resampled.",
            "License and provenance are source-provided claims, not independently verified.",
        ],
    },
    CreditsSection {
        heading: "Font",
        lines: &[
            "Iosevka Term by Renzhi Li (Belleve Invis)",
            "https://typeof.net/Iosevka/",
            "SIL Open Font License 1.1",
        ],
    },
    CreditsSection {
        heading: "Input prompts",
        lines: &[
            "FREE Input Prompts Pack v1.4 by JulioCacko",
            "https://juliocacko.itch.io/free-input-prompts",
            "Recorded as CC0 1.0 Universal; not independently verified.",
        ],
    },
    CreditsSection {
        heading: "Textures",
        lines: &[
            "Poly Haven concrete_floor_worn_001",
            "Dimitrios Savva (photography), Rico Cilliers (processing)",
            "https://polyhaven.com/a/concrete_floor_worn_001",
            "CC0 1.0 Universal",
        ],
    },
    CreditsSection {
        heading: "Sounds",
        lines: &[
            "Gristi - snd_footsteps_metal_floor_inside.wav",
            "https://freesound.org/people/gristi/sounds/562195/",
            "Ultra-Edward - Crawling Through a Vent",
            "https://freesound.org/people/Ultra-Edward/sounds/795872/",
            "Jofae - Growl and Roar",
            "https://freesound.org/people/Jofae/sounds/366837/",
            "under_the_hood - Real heartbeat sound fastest",
            "https://freesound.org/people/under_the_hood/sounds/455440/",
            "GboxMikeFozzy - Footsteps",
            "https://opengameart.org/content/footsteps-0",
            "rubberduck - 100 CC0 metal and wood SFX",
            "https://opengameart.org/content/100-cc0-metal-and-wood-sfx",
            "DrFahrts - doorknob rattle",
            "https://freesound.org/people/DrFahrts/sounds/727791/",
            "CleytonKauffman - SFX - Circuit breaker",
            "https://opengameart.org/content/sfx-circuit-breaker",
            "SFX by Cleyton Kauffman - https://soundcloud.com/cleytonkauffman",
            "mikeask - Breathing Tired",
            "https://opengameart.org/content/breathing-tired",
            "Ralph0o7 - Flashlight switch",
            "https://freesound.org/people/Ralph0o7/sounds/690300/",
            "Avreliy - ThrowGrenadeCloth",
            "https://freesound.org/people/Avreliy/sounds/611613/",
            "iankath - Furnace.WAV",
            "https://freesound.org/people/iankath/sounds/173991/",
            "willstepp - Water Drop",
            "https://freesound.org/people/willstepp/sounds/188293/",
            "DBlover - Howling Wind Ambience",
            "https://freesound.org/people/DBlover/sounds/405601/",
            "These source pages display CC0 1.0. Authorship and rights are not independently verified.",
        ],
    },
    CreditsSection {
        heading: "Licenses",
        lines: &[
            "Full credits, license texts, and dependency licenses are in the credits folder that ships with the game.",
        ],
    },
];

#[derive(Resource, Default, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CreditsRoute {
    #[default]
    MainMenu,
    Ending,
}

#[derive(Component)]
pub(super) struct CreditsScreen;

#[derive(Component)]
struct CreditsViewport;

#[derive(Component, Default)]
pub(super) struct CreditsRoll {
    pub(super) offset: f32,
    started: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CreditsRoute>()
        .add_systems(OnEnter(GameState::Credits), (spawn_credits, release_cursor))
        .add_systems(Update, roll.run_if(in_state(GameState::Credits)));
}

pub(super) fn wrap_offset(offset: f32, span: f32) -> f32 {
    if span <= 0.0 {
        return 0.0;
    }
    offset.max(0.0) % span
}

fn spawn_credits(mut commands: Commands, assets: Res<UiAssets>, route: Res<CreditsRoute>) {
    let font = assets.font.clone();
    let scene = build_closed_exit_cinematic(&mut commands);
    commands
        .entity(scene.root)
        .insert(DespawnOnExit(GameState::Credits));
    let view = scene
        .view
        .with_translation(scene.view.translation + scene.view.back() * 0.4);
    cinematic::spawn_cameras(&mut commands, GameState::Credits, view);
    let roll = commands
        .spawn((
            CreditsRoll::default(),
            Node {
                width: percent(100),
                max_width: px(COLUMN_WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                row_gap: px(10),
                top: percent(100),
                ..default()
            },
        ))
        .id();
    for section in SECTIONS {
        commands.entity(roll).with_children(|parent| {
            parent.spawn(line(section.heading, 26.0, 36.0, font.clone()));
            for value in section.lines {
                parent.spawn(line(value, 18.0, 0.0, font.clone()));
            }
        });
    }
    let buttons = match *route {
        CreditsRoute::MainMenu => vec![("Back button", MenuAction::MainMenu, "Back")],
        CreditsRoute::Ending => vec![
            ("Retry button", MenuAction::Retry, "Retry"),
            ("Main menu button", MenuAction::MainMenu, "Main Menu"),
            #[cfg(not(target_arch = "wasm32"))]
            ("Quit button", MenuAction::Quit, "Quit"),
        ],
    };
    let controls = commands
        .spawn(Node {
            width: percent(30),
            max_width: px(300),
            min_width: px(160),
            flex_direction: FlexDirection::Column,
            row_gap: px(BUTTON_GAP),
            ..default()
        })
        .with_children(|parent| {
            if *route == CreditsRoute::Ending {
                parent.spawn((
                    text("ESCAPED", 64.0, theme::ACCENT, font.clone()),
                    Node {
                        margin: UiRect::bottom(px(28)),
                        ..default()
                    },
                ));
            }
            for (name, action, label) in buttons {
                parent.spawn((Name::new(name), action, menu_button(label, font.clone())));
            }
        })
        .id();
    commands
        .spawn((
            CreditsScreen,
            Name::new("Credits screen"),
            DespawnOnExit(GameState::Credits),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                padding: UiRect::axes(px(72), px(36)),
                column_gap: px(32),
                ..default()
            },
            BackgroundColor(theme::BACKGROUND.with_alpha(0.4)),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100),
                    height: percent(100),
                    min_width: px(0),
                    min_height: px(0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(16),
                    ..default()
                })
                .with_children(|column| {
                    column.spawn(text("CREDITS", 44.0, theme::ACCENT, font.clone()));
                    column.spawn(text(
                        "Mouse wheel or Up/Down to scroll",
                        15.0,
                        theme::TEXT,
                        font.clone(),
                    ));
                    column
                        .spawn((
                            CreditsViewport,
                            Node {
                                width: percent(100),
                                flex_grow: 1.0,
                                min_height: px(0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                        ))
                        .add_child(roll);
                });
        })
        .insert_children(0, &[controls]);
}

fn line(value: &'static str, size: f32, gap: f32, font: Handle<Font>) -> impl Bundle {
    (
        text(value, size, theme::TEXT, font),
        TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
        Node {
            max_width: percent(100),
            margin: UiRect::top(px(gap)),
            ..default()
        },
    )
}

fn roll(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    viewports: Query<&ComputedNode, With<CreditsViewport>>,
    mut rolls: Query<(&mut CreditsRoll, &mut Node, &ComputedNode)>,
) {
    let mut delta = ROLL_SPEED * time.delta_secs();
    if keys.pressed(KeyCode::ArrowDown) {
        delta += KEY_SCROLL_SPEED * time.delta_secs();
    }
    if keys.pressed(KeyCode::ArrowUp) {
        delta -= (KEY_SCROLL_SPEED + ROLL_SPEED) * time.delta_secs();
    }
    for event in wheel.read() {
        delta -= match event.unit {
            MouseScrollUnit::Line => event.y * LINE_SCROLL,
            MouseScrollUnit::Pixel => event.y,
        };
    }
    let view = viewports
        .iter()
        .map(|node| node.size().y * node.inverse_scale_factor())
        .next()
        .unwrap_or(0.0);
    if view <= 0.0 {
        return;
    }
    for (mut roll, mut node, computed) in &mut rolls {
        let content = computed.size().y * computed.inverse_scale_factor();
        if !roll.started {
            roll.started = true;
            roll.offset = view * START_LEAD;
        }
        roll.offset = wrap_offset(roll.offset + delta, view + content);
        node.top = px(view - roll.offset);
    }
}
