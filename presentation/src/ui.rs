//! The arena's screen UI, in the Liquid Glass style of quiz-catch's light theme:
//! controls are drops of tinted glass floating over the scene, with a lit rim, soft
//! shadows and spring motion.
//!
//! On screen all the time: the score capsule at the top, an agent picker down each
//! side, in the order they would finish a tournament at the current settings, each
//! with its place (the chosen agent's numbers under it), and the control bar along the bottom:
//! Play, Reset, and a stepper each for the board, the time per move and the boards at
//! once. `?` or `H` springs the shortcut list in over the bar; starting a series or
//! `Esc` puts it away. Only an error shows as a toast under the score.
//!
//! The glass moves like liquid: a lens flows under the bar's hovered control, leading
//! edge first, so it stretches on the way; light gathers where the pointer is over
//! any pane; a changed value rolls up into place.
//!
//! The top bar, pickers and control bar's measured sizes go to `scene::UiMargins`, so
//! the camera frames the boards in the space between them.

use bevy::picking::Pickable;
use bevy::picking::hover::Hovered;
use bevy::picking::pointer::{PointerId, PointerLocation};
use bevy::prelude::*;
use bevy::text::LetterSpacing;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::{Activate, Button};

use crate::effects::Won;
use crate::game::{Action, Pending, Player, Series, Settings};
use crate::present::Mode;
use crate::ranking::{Ranking, update_ranking};
use crate::readout::{Readout, ReadoutSet};
use crate::roster::ROSTER;
use crate::scene::UiMargins;
use crate::sizes::LADDER;
use crate::theme::{
    ACCENT, ACCENT_INK, FAINT, FILL, FILL_2, GLASS, HI_A, HI_B, HI_C, MUTED, ON_ACCENT, PILL, RED, RED_INK, SEP, SHADOW, TEXT,
    THICK, YELLOW, YELLOW_INK, bold_font, ink, medium_font, title_font, ui_font, white,
};

/// Space between the window edge and the glass.
const PAD: f32 = 16.0;
const PICKER_WIDTH: f32 = 316.0;
/// One row of an agent picker, and the gap under it.
const ROW: f32 = 36.0;
const ROW_GAP: f32 = 2.0;
const CAPSULE: f32 = 999.0;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IntroClock>()
            .init_resource::<Ranking>()
            .init_resource::<Chrome>()
            .add_systems(Startup, (build_ui, open_from_env))
            .add_observer(on_activate)
            .add_systems(OnEnter(Mode::Arena), |mut clock: ResMut<IntroClock>| clock.0 = 0.0)
            .add_systems(
                Update,
                (
                    keys,
                    put_away_on_play,
                    style_buttons,
                    ripples,
                    update_values,
                    update_cards,
                    update_ranking,
                    update_rows,
                    reorder_rows,
                    pills,
                    lens,
                    glint,
                    update_score,
                    floaters,
                    pops,
                    reveal,
                    intro,
                    win_flash,
                    measure,
                )
                    .after(ReadoutSet)
                    .run_if(in_state(Mode::Arena)),
            )
            .add_systems(Update, (show_ui, fit_ui));
    }
}

// ================================================================== state

/// Which of the on-demand panes are open.
#[derive(Resource, Default)]
struct Chrome {
    help: bool,
}

impl Chrome {
    fn close_all(&mut self) {
        *self = Self::default();
    }

    fn toggle(&mut self, toggle: Toggle) {
        match toggle {
            Toggle::Help => self.help = !self.help,
        }
    }
}

/// A button that opens or closes a pane instead of acting on the game.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Toggle {
    Help,
}

/// A control in the bottom bar: the lens flows under it when hovered.
#[derive(Component)]
struct InBar;

/// The glass lens under the bar's hovered control. Each edge springs on its own, the
/// leading one stiffer, so the lens stretches toward where it is going.
#[derive(Component, Default)]
struct Lens {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    left_velocity: f32,
    right_velocity: f32,
    shown: f32,
}

/// Light that gathers under the pointer on a pane of glass.
#[derive(Component, Default)]
struct Glint {
    at: Vec2,
    strength: f32,
}

/// The whole overlay, hidden while presenting.
#[derive(Component)]
struct UiRoot;

/// The glass whose size keeps the boards clear of it.
#[derive(Component)]
struct TopBar;
#[derive(Component)]
struct BottomBar;
#[derive(Component)]
struct SidePanel;

/// A text that shows one readout field.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Value {
    Size,
    Time,
    Boards,
    Play,
    Note,
    Toast,
    Opening,
    Agent(Player),
}

/// What a button looks like.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Look {
    /// The palette's colour as tinted glass: Play.
    Primary,
    /// Words or an icon with no capsule until hovered.
    Ghost,
    /// A row in a picker; the flowing pill marks the chosen one.
    Row,
}

/// A pane that springs in when its condition holds and away when it does not.
#[derive(Component)]
struct Reveal {
    pane: Pane,
    shown: f32,
    velocity: f32,
    /// Where it comes from, in pixels.
    from: Vec2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    Help,
    Toast,
    Note,
}

#[derive(Component)]
struct ScoreText(Player);
/// Holds a score and the "+1"s that float off it.
#[derive(Component)]
struct ScoreBox(Player);

/// The pill behind a picker's chosen agent; it flows between rows.
#[derive(Component)]
struct Pill {
    player: Player,
    y: f32,
    velocity: f32,
}
/// A picker row: it stands in its agent's tournament place, springing there.
#[derive(Component)]
struct PickerRow {
    agent: usize,
    y: f32,
    velocity: f32,
}
/// The place number left of an agent's name.
#[derive(Component)]
struct PlaceLabel(usize);

/// A picker row's title, which is brighter when chosen.
#[derive(Component)]
struct RowLabel(Player, usize);

/// An agent card; slides in when the agent changes.
#[derive(Component)]
struct CardBox {
    player: Player,
    age: f32,
    agent: Option<usize>,
}
#[derive(Component, Clone, Copy)]
enum CardText {
    Tile(Player, usize),
}

/// A text that pops (grows and flashes orange) when its value changes.
#[derive(Component)]
struct Pop {
    age: f32,
    base: Color,
    strength: f32,
}

impl Pop {
    const fn new(base: Color, strength: f32) -> Self {
        Self { age: 10.0, base, strength }
    }
}

/// A button's spring: scale and its velocity, plus a flash after a click.
#[derive(Component, Default)]
struct Juice {
    scale: f32,
    velocity: f32,
    flash: f32,
}

/// A ring that grows out of a clicked button and fades.
#[derive(Component)]
struct Ripple(f32);

/// A "+1" rising off the scoreboard.
#[derive(Component)]
struct Floater {
    age: f32,
    colour: Color,
}

/// A glow round the window edge when a game is won.
#[derive(Component)]
struct EdgeFlash {
    age: f32,
    colour: Color,
}

/// Slides the fixed glass in from `from` (pixels) when the arena opens.
#[derive(Component)]
struct Enter {
    from: Vec2,
    delay: f32,
}

/// Seconds since the arena last opened.
#[derive(Resource, Default)]
struct IntroClock(f32);

const fn disc(player: Player) -> Color {
    match player {
        Player::Red => RED,
        Player::Yellow => YELLOW,
    }
}

/// A player's colour as text on cream.
const fn ink_of(player: Player) -> Color {
    match player {
        Player::Red => RED_INK,
        Player::Yellow => YELLOW_INK,
    }
}

// ================================================================== building blocks

/// Text in Inter, tracked tighter as it grows, the way the system type scale is.
fn text(value: impl Into<String>, face: bevy::text::FontSource, size: f32, colour: Color) -> impl Bundle {
    let tracking = if size >= 26.0 {
        -0.026
    } else if size >= 17.0 {
        -0.021
    } else if size >= 14.0 {
        -0.011
    } else {
        -0.003
    };
    (
        Text::new(value),
        TextFont {
            font: face,
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(colour),
        LetterSpacing::Px(size * tracking),
    )
}

/// The rim: one pixel of light, brightest at the top left and bottom right.
fn rim() -> BorderGradient {
    BorderGradient::from(LinearGradient::new(
        LinearGradient::TO_BOTTOM_RIGHT,
        vec![
            ColorStop::new(HI_A, percent(0)),
            ColorStop::new(HI_B, percent(24)),
            ColorStop::new(HI_C, percent(50)),
            ColorStop::new(HI_B, percent(76)),
            ColorStop::new(HI_A, percent(100)),
        ],
    ))
}

/// Light pooling under the top of a pane, standing in for the inner glow.
fn sheen(depth: f32, strength: f32) -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient::new(
        LinearGradient::TO_BOTTOM,
        vec![ColorStop::new(white(strength), px(0)), ColorStop::new(white(0.0), px(depth))],
    ))
}

fn shadow(colour: Color, y: f32, spread: f32, blur: f32) -> ShadowStyle {
    ShadowStyle {
        color: colour,
        x_offset: px(0),
        y_offset: px(y),
        spread_radius: px(spread),
        blur_radius: px(blur),
    }
}

/// A floating pane of glass: tinted surface, lit rim, a hairline and a soft shadow.
fn glass(node: Node, fill: Color, radius: f32) -> impl Bundle {
    (
        Node {
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(radius)),
            ..node
        },
        BackgroundColor(fill),
        sheen(28.0, 0.55),
        Glint::default(),
        rim(),
        BoxShadow(vec![
            shadow(ink(0.07), 0.0, 0.5, 0.0),
            shadow(SHADOW, 2.0, -2.0, 6.0),
            shadow(SHADOW, 18.0, -12.0, 44.0),
        ]),
    )
}

fn button_node(look: Look, height: f32) -> Node {
    let (padding, radius) = match look {
        Look::Row => (UiRect::horizontal(px(12)), 14.0),
        Look::Primary => (UiRect::horizontal(px(24)), CAPSULE),
        Look::Ghost => (UiRect::horizontal(px(16)), CAPSULE),
    };
    Node {
        min_height: px(height),
        min_width: px(height),
        padding,
        column_gap: px(8),
        justify_content: if look == Look::Row { JustifyContent::SpaceBetween } else { JustifyContent::Center },
        align_items: AlignItems::Center,
        border: UiRect::all(px(if look == Look::Ghost || look == Look::Row { 0 } else { 1 })),
        border_radius: BorderRadius::all(px(radius)),
        ..default()
    }
}

/// A pressable thing; `what` is an `Action` or a `Toggle`.
fn pressable(look: Look, height: f32, what: impl Bundle) -> impl Bundle {
    let fill = match look {
        Look::Primary => ACCENT,
        Look::Ghost | Look::Row => ink(0.0),
    };
    (
        Button,
        Hovered::default(),
        Juice::default(),
        look,
        what,
        button_node(look, height),
        BackgroundColor(fill),
    )
}

fn icon(what: impl Bundle, label: &str, size: f32) -> impl Bundle {
    (
        pressable(Look::Ghost, 32.0, what),
        children![(text(label, medium_font(), size, MUTED), Pickable::IGNORE)],
    )
}

/// A coloured dot, the size of a disc seen from far away.
fn dot(colour: Color, size: f32) -> impl Bundle {
    (
        Node {
            width: px(size),
            height: px(size),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(colour),
        BoxShadow(vec![shadow(colour.with_alpha(0.45), 1.0, 0.0, 4.0)]),
        Pickable::IGNORE,
    )
}

// ================================================================== the layout

fn build_ui(mut commands: Commands) {
    commands
        .spawn((
            UiRoot,
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            // The edge glow sits under everything and never takes clicks.
            root.spawn((
                EdgeFlash { age: 10.0, colour: ACCENT },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    border: UiRect::all(px(6)),
                    ..default()
                },
                BorderColor::all(Color::NONE),
                Pickable::IGNORE,
            ));
            top_bar(root);
            for player in [Player::Red, Player::Yellow] {
                picker(root, player);
            }
            bottom(root);
        });
}

fn top_bar(root: &mut ChildSpawnerCommands<'_>) {
    root.spawn((
        TopBar,
        Enter {
            from: Vec2::new(0.0, -120.0),
            delay: 0.0,
        },
        Node {
            position_type: PositionType::Absolute,
            left: px(PAD),
            right: px(PAD),
            top: px(PAD),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|top| {
        // Left: empty, balancing the right so the score stays centred.
        top.spawn(Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            ..default()
        });
        // Middle: the score, naming both agents; a note and a toast hang under it.
        top.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(8),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|middle| {
            middle
                .spawn(glass(
                    Node {
                        height: px(56),
                        padding: UiRect::all(px(6)),
                        column_gap: px(4),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    GLASS,
                    CAPSULE,
                ))
                .with_children(|score| {
                    seat(score, Player::Red);
                    score_box(score, Player::Red);
                    score.spawn((text(":", title_font(), 24.0, FAINT), Pickable::IGNORE));
                    score_box(score, Player::Yellow);
                    seat(score, Player::Yellow);
                });
            middle.spawn((
                Reveal {
                    pane: Pane::Note,
                    shown: 0.0,
                    velocity: 0.0,
                    from: Vec2::new(0.0, -8.0),
                },
                Node { display: Display::None, ..default() },
                Pickable::IGNORE,
                children![(text("", medium_font(), 13.0, FAINT), Value::Note, Pickable::IGNORE)],
            ));
            middle.spawn((
                Reveal {
                    pane: Pane::Toast,
                    shown: 0.0,
                    velocity: 0.0,
                    from: Vec2::new(0.0, -26.0),
                },
                glass(
                    Node {
                        display: Display::None,
                        padding: UiRect::axes(px(18), px(10)),
                        ..default()
                    },
                    THICK,
                    CAPSULE,
                ),
                Pickable::IGNORE,
                children![(text("", bold_font(), 16.0, TEXT), Value::Toast, Pickable::IGNORE)],
            ));
        });
        // Right: the slides, and the shortcut list.
        top.spawn((
            Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                justify_content: JustifyContent::FlexEnd,
                column_gap: px(8),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|right| {
            right
                .spawn(glass(
                    Node {
                        height: px(48),
                        padding: UiRect::all(px(4)),
                        column_gap: px(2),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    GLASS,
                    CAPSULE,
                ))
                .with_children(|group| {
                    group.spawn(icon(Toggle::Help, "?", 17.0));
                    group.spawn((
                        pressable(Look::Ghost, 38.0, Action::Present),
                        children![(text("Slides", medium_font(), 15.0, TEXT), Pickable::IGNORE)],
                    ));
                });
        });
    });
}

/// One side's agent in the score capsule: a dot and a name.
fn seat(parent: &mut ChildSpawnerCommands<'_>, player: Player) {
    let name = (text("", medium_font(), 15.0, TEXT), Value::Agent(player), Pickable::IGNORE);
    let dot = dot(disc(player), 12.0);
    let mut holder = parent.spawn((
        Node {
            min_height: px(44),
            padding: UiRect::horizontal(px(16)),
            column_gap: px(8),
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
    ));
    holder.with_children(|inside| match player {
        Player::Red => {
            inside.spawn(dot);
            inside.spawn(name);
        }
        Player::Yellow => {
            inside.spawn(name);
            inside.spawn(dot);
        }
    });
}

fn score_box(parent: &mut ChildSpawnerCommands<'_>, player: Player) {
    parent
        .spawn((
            ScoreBox(player),
            Node {
                min_width: px(34),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|slot| {
            slot.spawn((
                text("0", title_font(), 28.0, ink_of(player)),
                ScoreText(player),
                Pop::new(ink_of(player), 0.6),
                Pickable::IGNORE,
            ));
        });
}

/// A side's picker, always shown: the eight agents, then the chosen one's numbers.
fn picker(root: &mut ChildSpawnerCommands<'_>, player: Player) {
    let side = if player == Player::Red { -1.0 } else { 1.0 };
    let mut node = Node {
        position_type: PositionType::Absolute,
        top: px(PAD + 56.0 + 14.0),
        width: px(PICKER_WIDTH),
        padding: UiRect::all(px(8)),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    if player == Player::Red {
        node.left = px(PAD);
    } else {
        node.right = px(PAD);
    }
    root.spawn((
        SidePanel,
        Enter {
            from: Vec2::new(side * (PICKER_WIDTH + 40.0), 0.0),
            delay: 0.05,
        },
        glass(node, THICK, 28.0),
    ))
    .with_children(|pane| {
        // Rows stand in tournament order; each springs to its place when it changes.
        pane.spawn(Node {
            height: px(8.0 * (ROW + ROW_GAP) - ROW_GAP),
            ..default()
        })
        .with_children(|list| {
            list.spawn((
                Pill { player, y: -1.0, velocity: 0.0 },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    height: px(ROW),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(14)),
                    ..default()
                },
                BackgroundColor(PILL),
                rim(),
                BoxShadow(vec![shadow(ink(0.06), 0.0, 0.5, 0.0), shadow(SHADOW, 2.0, -3.0, 8.0)]),
                Pickable::IGNORE,
            ));
            for (agent, info) in ROSTER.iter().enumerate() {
                list.spawn((
                    pressable(Look::Row, ROW, Action::Pick(player, agent)),
                    PickerRow { agent, y: -1.0, velocity: 0.0 },
                    ZIndex(1),
                ))
                .insert(Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    ..button_node(Look::Row, ROW)
                })
                .with_children(|row| {
                    row.spawn((
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: px(10),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|words| {
                        words.spawn((
                            text("", bold_font(), 12.0, FAINT),
                            PlaceLabel(agent),
                            Node {
                                width: px(14),
                                ..default()
                            },
                            Pop::new(FAINT, 0.3),
                            Pickable::IGNORE,
                        ));
                        words.spawn((text(info.title, medium_font(), 15.0, MUTED), RowLabel(player, agent), Pickable::IGNORE));
                    });
                });
            }
        });
        pane.spawn((
            Node {
                height: px(1),
                margin: UiRect::new(px(10), px(10), px(10), px(4)),
                ..default()
            },
            BackgroundColor(SEP),
        ));
        pane.spawn((
            CardBox { player, age: 10.0, agent: None },
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(px(10), px(10), px(6), px(4)),
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|card| {
            card.spawn(Node {
                column_gap: px(6),
                margin: UiRect::top(px(2)),
                ..default()
            })
            .with_children(|tiles| {
                for (index, caption) in ["Rank", "8\u{d7}7 acc.", "9\u{d7}8 acc."].into_iter().enumerate() {
                    tiles
                        .spawn((
                            Node {
                                flex_grow: 1.0,
                                flex_basis: px(0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::axes(px(10), px(8)),
                                row_gap: px(1),
                                border_radius: BorderRadius::all(px(16)),
                                ..default()
                            },
                            BackgroundColor(FILL),
                        ))
                        .with_children(|tile| {
                            tile.spawn((text("", title_font(), 18.0, TEXT), CardText::Tile(player, index)));
                            tile.spawn(text(caption, medium_font(), 11.0, FAINT));
                        });
                }
            });
        });
    });
}

/// The shortcut list, over the control bar.
fn help(column: &mut ChildSpawnerCommands<'_>) {
    const KEYS: [(&str, &str); 11] = [
        ("Space", "Play or pause"),
        ("R", "Reset"),
        ("\u{2212}  +", "Board size"),
        ("[  ]", "Time per move"),
        (",  .", "Boards at once"),
        ("O", "Empty or random start"),
        ("P", "Slides"),
        ("F11", "Full screen"),
        ("Esc", "Close this list"),
        ("Left-drag", "Orbit"),
        ("Right-drag", "Pan"),
    ];
    column.spawn((
        Reveal {
            pane: Pane::Help,
            shown: 0.0,
            velocity: 0.0,
            from: Vec2::new(0.0, 70.0),
        },
        glass(
            Node {
                display: Display::None,
                width: px(290),
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                row_gap: px(7),
                ..default()
            },
            THICK,
            24.0,
        ),
    ))
    .with_children(|card| {
        for (key, what) in KEYS {
            card.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Node {
                        min_width: px(92),
                        justify_content: JustifyContent::Center,
                        padding: UiRect::axes(px(8), px(3)),
                        border_radius: BorderRadius::all(px(8)),
                        ..default()
                    },
                    BackgroundColor(FILL_2),
                    children![text(key, bold_font(), 12.0, MUTED)],
                ));
                row.spawn(text(what, ui_font(), 14.0, TEXT));
            });
        }
    });
}

/// The bottom: the shortcut list (when open) over the control bar.
fn bottom(root: &mut ChildSpawnerCommands<'_>) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(PAD),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(12),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|column| {
        help(column);
        column
            .spawn((
                BottomBar,
                Enter {
                    from: Vec2::new(0.0, 140.0),
                    delay: 0.1,
                },
                glass(
                    Node {
                        height: px(64),
                        padding: UiRect::axes(px(8), px(8)),
                        column_gap: px(6),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    GLASS,
                    CAPSULE,
                ),
            ))
            .with_children(|bar| {
                bar.spawn((
                    Lens::default(),
                    Node {
                        position_type: PositionType::Absolute,
                        display: Display::None,
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(PILL),
                    BorderColor::all(white(0.0)),
                    BoxShadow(vec![shadow(ink(0.06), 0.0, 0.5, 0.0), shadow(SHADOW, 3.0, -3.0, 10.0)]),
                    Pickable::IGNORE,
                ));
                bar.spawn((
                    pressable(Look::Primary, 48.0, Action::PlayPause),
                    rim(),
                    sheen(26.0, 0.32),
                    BoxShadow(vec![shadow(ACCENT.with_alpha(0.5), 8.0, -8.0, 22.0)]),
                    children![(text("", bold_font(), 16.0, ON_ACCENT), Value::Play, Pickable::IGNORE)],
                ))
                .insert(Node {
                    min_width: px(132),
                    ..button_node(Look::Primary, 48.0)
                });
                bar.spawn((
                    pressable(Look::Ghost, 48.0, Action::Reset),
                    InBar,
                    children![(text("Reset", medium_font(), 15.0, TEXT), Pickable::IGNORE)],
                ));
                choice(bar, "START", Action::Opening, Value::Opening, 70.0);
                stepper(bar, "BOARD", (Action::Shrink, Value::Size, Action::Grow), 76.0);
                stepper(bar, "PER MOVE", (Action::Faster, Value::Time, Action::Slower), 70.0);
                stepper(bar, "BOARDS", (Action::FewerBoards, Value::Boards, Action::MoreBoards), 58.0);
            });
    });
}

/// A small caption over a value, the whole capsule one button that switches it.
fn choice(bar: &mut ChildSpawnerCommands<'_>, caption: &str, action: Action, value: Value, width: f32) {
    bar.spawn((pressable(Look::Ghost, 48.0, action), InBar))
        .insert(Node {
            min_width: px(width + 28.0),
            flex_direction: FlexDirection::Column,
            ..button_node(Look::Ghost, 48.0)
        })
        .with_children(|slot| {
            slot.spawn((
                Text::new(caption),
                TextFont {
                    font: bold_font(),
                    font_size: FontSize::Px(9.5),
                    ..default()
                },
                TextColor(FAINT),
                LetterSpacing::Px(0.9),
                Pickable::IGNORE,
            ));
            slot.spawn((text("", bold_font(), 16.0, TEXT), value, Pop::new(TEXT, 0.18), Pickable::IGNORE));
        });
}

/// Minus, a small caption over the value, plus.
fn stepper(bar: &mut ChildSpawnerCommands<'_>, caption: &str, (less, value, more): (Action, Value, Action), width: f32) {
    bar.spawn((
        Node {
            height: px(48),
            padding: UiRect::horizontal(px(2)),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(ink(0.035)),
        Pickable::IGNORE,
    ))
    .with_children(|group| {
        group.spawn((icon(less, "\u{2212}", 18.0), InBar)).insert(button_node(Look::Ghost, 44.0));
        group
            .spawn((
                Node {
                    width: px(width),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|slot| {
                slot.spawn((
                    Text::new(caption),
                    TextFont {
                        font: bold_font(),
                        font_size: FontSize::Px(9.5),
                        ..default()
                    },
                    TextColor(FAINT),
                    LetterSpacing::Px(0.9),
                    Pickable::IGNORE,
                ));
                slot.spawn((text("", bold_font(), 16.0, TEXT), value, Pop::new(TEXT, 0.18), Pickable::IGNORE));
            });
        group.spawn((icon(more, "+", 18.0), InBar)).insert(button_node(Look::Ghost, 44.0));
    });
}

// ================================================================== behaviour

fn show_ui(mode: Res<State<Mode>>, mut roots: Query<&mut Node, With<UiRoot>>) {
    let wanted = if *mode.get() == Mode::Arena { Display::Flex } else { Display::None };
    for mut node in &mut roots {
        if node.display != wanted {
            node.display = wanted;
        }
    }
}

fn on_activate(
    event: On<Activate>,
    mut commands: Commands,
    mut buttons: Query<(Option<&Action>, Option<&Toggle>, &Look, &mut Juice)>,
    mut pending: ResMut<Pending>,
    mut chrome: ResMut<Chrome>,
) {
    let Ok((action, toggle, &look, mut juice)) = buttons.get_mut(event.entity) else {
        return;
    };
    if let Some(&action) = action {
        pending.0.push(action);
    }
    if let Some(&toggle) = toggle {
        chrome.toggle(toggle);
    }
    juice.flash = 1.0;
    juice.velocity -= 2.4;
    let ring = match action {
        Some(Action::Pick(player, _)) => disc(*player),
        _ => ACCENT,
    };
    commands.spawn((
        Ripple(0.0),
        Node {
            position_type: PositionType::Absolute,
            left: px(-1),
            right: px(-1),
            top: px(-1),
            bottom: px(-1),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(if look == Look::Row { 14.0 } else { CAPSULE })),
            ..default()
        },
        BorderColor::all(ring.with_alpha(0.7)),
        Pickable::IGNORE,
        ChildOf(event.entity),
    ));
}

/// Developer aid: `C4_OPEN=help` opens the shortcut list at startup, for screenshots.
fn open_from_env(mut chrome: ResMut<Chrome>) {
    let Ok(list) = std::env::var("C4_OPEN") else {
        return;
    };
    for name in list.split(',') {
        match name.trim() {
            "help" => chrome.toggle(Toggle::Help),
            _ => {}
        }
    }
}

/// `H` (or `?`) for the shortcut list, `Esc` to put every pane away.
fn keys(keys: Res<ButtonInput<KeyCode>>, mut chrome: ResMut<Chrome>) {
    if keys.just_pressed(KeyCode::KeyH) || keys.just_pressed(KeyCode::Slash) {
        chrome.toggle(Toggle::Help);
    }
    if keys.just_pressed(KeyCode::Escape) {
        chrome.close_all();
    }
}

/// Starting a series means watching: the panes get out of the way.
fn put_away_on_play(series: Res<Series>, mut was: Local<bool>, mut chrome: ResMut<Chrome>) {
    if series.playing && !*was {
        chrome.close_all();
    }
    *was = series.playing;
}

/// Colours ease toward their state, and each button springs: a little up on hover,
/// in while pressed, wobbling back after a click.
#[allow(clippy::type_complexity)]
fn style_buttons(
    time: Res<Time>,
    chrome: Res<Chrome>,
    mut buttons: Query<(
        &Look,
        Has<InBar>,
        Option<&Toggle>,
        &Hovered,
        Has<Pressed>,
        Has<InteractionDisabled>,
        &mut Juice,
        &mut BackgroundColor,
        &mut UiTransform,
    )>,
) {
    let dt = time.delta_secs().min(1.0 / 20.0);
    let blend = 1.0 - (-dt * 18.0).exp();
    for (look, in_bar, toggle, hovered, pressed, disabled, mut juice, mut background, mut transform) in &mut buttons {
        let hover = hovered.get() && !disabled;
        let open = toggle.is_some_and(|_| chrome.help);
        let fill = match (look, pressed, hover || open) {
            // In the bar, the lens shows which control is under the pointer.
            (Look::Ghost, false, _) if in_bar => ink(0.0),
            (Look::Ghost, true, _) if in_bar => ink(0.04),
            (Look::Primary, true, _) => ACCENT.mix(&TEXT, 0.12),
            (Look::Primary, false, true) => ACCENT.mix(&white(1.0), 0.14),
            (Look::Primary, false, false) => ACCENT,
            (Look::Ghost | Look::Row, true, _) => FILL_2,
            (Look::Ghost | Look::Row, false, true) => FILL,
            (Look::Ghost | Look::Row, false, false) => ink(0.0),
        };
        juice.flash = (juice.flash - dt * 3.5).max(0.0);
        let fill = fill.mix(&white(0.9), juice.flash * 0.3);
        background.0 = background.0.mix(&fill, blend);

        if juice.scale == 0.0 {
            juice.scale = 1.0;
        }
        let target = if pressed {
            0.95
        } else if hover && *look != Look::Row {
            1.03
        } else {
            1.0
        };
        // A bouncy spring: overshoots about 8% and settles.
        let force = (target - juice.scale) * 420.0 - juice.velocity * 17.0;
        juice.velocity += force * dt;
        juice.scale += juice.velocity * dt;
        let scale = Vec2::splat(juice.scale);
        if transform.scale != scale {
            transform.scale = scale;
        }
    }
}

fn ripples(mut commands: Commands, time: Res<Time>, mut rings: Query<(Entity, &mut Ripple, &mut UiTransform, &mut BorderColor)>) {
    for (entity, mut ring, mut transform, mut border) in &mut rings {
        ring.0 += time.delta_secs();
        let t = (ring.0 / 0.5).min(1.0);
        let grow = 1.0 - (1.0 - t).powi(3);
        transform.scale = Vec2::new(1.0 + grow * 0.12, 1.0 + grow * 0.35);
        let colour = border.top.with_alpha((1.0 - t) * 0.7);
        *border = BorderColor::all(colour);
        if t >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn set(text: &mut Text, value: &str) -> bool {
    let changed = text.0 != value;
    if changed {
        value.clone_into(&mut text.0);
    }
    changed
}

/// Every readout text; values that change pop.
fn update_values(readout: Res<Readout>, settings: Res<Settings>, mut texts: Query<(&Value, &mut Text, Option<&mut Pop>)>) {
    if !readout.is_changed() && !settings.is_changed() {
        return;
    }
    let size = LADDER[settings.size];
    for (value, mut text, pop) in &mut texts {
        let wanted = match value {
            Value::Size => format!("{} \u{d7} {}", size.columns, size.rows),
            Value::Time => readout.time.clone(),
            Value::Boards => readout.boards.clone(),
            Value::Play => readout.play.clone(),
            Value::Note => readout.note.clone(),
            Value::Toast => readout.toast.clone(),
            Value::Opening => if settings.random_opening { "Random" } else { "Empty" }.to_owned(),
            Value::Agent(player) => ROSTER[settings.agent[player.seat()]].title.to_owned(),
        };
        if set(&mut text, &wanted)
            && let Some(mut pop) = pop
        {
            pop.age = 0.0;
        }
    }
}

/// The card under each picker, sliding in when the agent changes.
fn update_cards(
    time: Res<Time>,
    settings: Res<Settings>,
    ranking: Res<Ranking>,
    mut cards: Query<(&mut CardBox, &mut UiTransform)>,
    mut texts: Query<(&CardText, &mut Text)>,
) {
    for (mut card, mut transform) in &mut cards {
        let agent = settings.agent[card.player.seat()];
        if card.agent != Some(agent) {
            // The first fill at startup is not a change.
            card.age = if card.agent.is_none() { 10.0 } else { 0.0 };
            card.agent = Some(agent);
        }
        card.age += time.delta_secs();
        let t = (card.age / 0.45).min(1.0);
        let rest = (1.0 - t).powi(3);
        let translation = Val2::px(0.0, 18.0 * rest);
        if transform.translation != translation {
            transform.translation = translation;
        }
    }
    if !settings.is_changed() && !ranking.is_changed() {
        return;
    }
    for (part, mut text) in &mut texts {
        let CardText::Tile(player, _) = *part;
        let agent = settings.agent[player.seat()];
        let info = &ROSTER[agent];
        let wanted = match *part {
            CardText::Tile(_, 0) => ranking.place[agent].map_or_else(|| "\u{2013}".to_owned(), |place| format!("#{place}")),
            CardText::Tile(_, 1) => format!("{:.1}%", info.acc_8x7 * 100.0),
            CardText::Tile(..) => format!("{:.1}%", info.acc_9x8 * 100.0),
        };
        set(&mut text, &wanted);
    }
}

/// The chosen agent's name is in its side's colour; the others are muted.
fn update_rows(settings: Res<Settings>, mut labels: Query<(&RowLabel, &mut TextColor)>) {
    if !settings.is_changed() {
        return;
    }
    for (label, mut ink) in &mut labels {
        ink.0 = if settings.agent[label.0.seat()] == label.1 { ink_of(label.0) } else { MUTED };
    }
}

/// Rows spring to their agent's place; the place numbers update (and pop) with them.
fn reorder_rows(
    time: Res<Time>,
    ranking: Res<Ranking>,
    mut rows: Query<(&mut PickerRow, &mut Node)>,
    mut places: Query<(&PlaceLabel, &mut Text, &mut Pop)>,
) {
    let dt = time.delta_secs().min(1.0 / 20.0);
    for (mut row, mut node) in &mut rows {
        #[expect(clippy::cast_precision_loss, reason = "eight rows")]
        let target = ranking.slot[row.agent] as f32 * (ROW + ROW_GAP);
        if row.y < 0.0 {
            row.y = target;
        }
        let force = (target - row.y) * 260.0 - row.velocity * 25.0;
        row.velocity += force * dt;
        row.y += row.velocity * dt;
        if (target - row.y).abs() < 0.05 && row.velocity.abs() < 0.5 {
            row.y = target;
            row.velocity = 0.0;
        }
        if node.top != px(row.y) {
            node.top = px(row.y);
        }
    }
    if !ranking.is_changed() {
        return;
    }
    for (label, mut text, mut pop) in &mut places {
        let wanted = ranking.place[label.0].map_or_else(String::new, |place| place.to_string());
        if set(&mut text, &wanted) {
            pop.age = 0.0;
        }
    }
}

/// Each picker's pill flows to the chosen row, stretching a little as it moves.
fn pills(
    time: Res<Time>,
    settings: Res<Settings>,
    ranking: Res<Ranking>,
    mut pills: Query<(&mut Pill, &mut Node, &mut UiTransform)>,
) {
    let dt = time.delta_secs().min(1.0 / 20.0);
    for (mut pill, mut node, mut transform) in &mut pills {
        #[expect(clippy::cast_precision_loss, reason = "eight rows")]
        let target = ranking.slot[settings.agent[pill.player.seat()]] as f32 * (ROW + ROW_GAP);
        if pill.y < 0.0 {
            pill.y = target;
        }
        let force = (target - pill.y) * 300.0 - pill.velocity * 26.0;
        pill.velocity += force * dt;
        pill.y += pill.velocity * dt;
        node.top = px(pill.y);
        let stretch = (pill.velocity.abs() / 900.0).min(1.0);
        transform.scale = Vec2::new(1.0 + 0.03 * stretch, 1.0 + 0.12 * stretch);
    }
}

/// Flows the lens under whichever bar control the pointer is on, leading edge first,
/// and fades it away when the pointer leaves the bar.
#[allow(clippy::type_complexity)]
fn lens(
    time: Res<Time>,
    bars: Query<(&ComputedNode, &UiGlobalTransform), With<BottomBar>>,
    controls: Query<(&Hovered, Has<InteractionDisabled>, &ComputedNode, &UiGlobalTransform), With<InBar>>,
    mut lenses: Query<(&mut Lens, &mut Node, &mut BackgroundColor, &mut BorderColor, &mut UiTransform)>,
) {
    let dt = time.delta_secs().min(1.0 / 20.0);
    let Ok((bar, bar_at)) = bars.single() else {
        return;
    };
    // Bar-local rectangles in UI pixels, inside the bar's one-pixel border.
    let corner = bar_at.translation - bar.size / 2.0;
    let target = controls
        .iter()
        .find(|(hovered, disabled, ..)| hovered.get() && !disabled)
        .map(|(_, _, node, at)| {
            let min = (at.translation - node.size / 2.0 - corner) * bar.inverse_scale_factor - Vec2::ONE;
            let max = min + node.size * bar.inverse_scale_factor;
            (min, max)
        });
    for (mut lens, mut node, mut fill, mut border, mut transform) in &mut lenses {
        if let Some((min, max)) = target {
            if lens.shown < 0.05 {
                (lens.left, lens.right) = (min.x, max.x);
                (lens.left_velocity, lens.right_velocity) = (0.0, 0.0);
            }
            (lens.top, lens.bottom) = (min.y, max.y);
            let going_right = (min.x + max.x) > (lens.left + lens.right);
            let (left_k, right_k): (f32, f32) = if going_right { (170.0, 520.0) } else { (520.0, 170.0) };
            let left_force = (min.x - lens.left) * left_k - lens.left_velocity * 2.0 * left_k.sqrt() * 0.85;
            let right_force = (max.x - lens.right) * right_k - lens.right_velocity * 2.0 * right_k.sqrt() * 0.85;
            lens.left_velocity += left_force * dt;
            lens.right_velocity += right_force * dt;
            let (left_velocity, right_velocity) = (lens.left_velocity, lens.right_velocity);
            lens.left += left_velocity * dt;
            lens.right += right_velocity * dt;
        }
        let wanted = if target.is_some() { 1.0 } else { 0.0 };
        let rate = if target.is_some() { 16.0 } else { 9.0 };
        lens.shown += (wanted - lens.shown) * (1.0 - (-dt * rate).exp());
        let display = if lens.shown > 0.01 { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
        if display == Display::None {
            continue;
        }
        node.left = px(lens.left);
        node.width = px((lens.right - lens.left).max(8.0));
        node.top = px(lens.top);
        node.height = px((lens.bottom - lens.top).max(8.0));
        fill.0 = PILL.with_alpha(PILL.alpha() * lens.shown);
        *border = BorderColor::all(white(0.85 * lens.shown));
        // Squashes a little as it travels, like a drop.
        let speed = (lens.left_velocity.abs() + lens.right_velocity.abs()) / 2.0;
        let squash = (speed / 1600.0).min(1.0);
        transform.scale = Vec2::new(1.0, (1.0 - 0.1 * squash) * (0.8 + 0.2 * lens.shown));
    }
}

/// Light gathers under the pointer on any pane of glass, and follows it softly.
fn glint(
    time: Res<Time>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    pointers: Query<(&PointerId, &PointerLocation)>,
    mut panes: Query<(&mut Glint, &ComputedNode, &UiGlobalTransform, &mut BackgroundGradient)>,
) {
    let dt = time.delta_secs().min(1.0 / 20.0);
    let scale = windows.single().map_or(1.0, Window::scale_factor);
    // The mouse as picking sees it, in physical pixels.
    let cursor = pointers
        .iter()
        .find(|(id, _)| id.is_mouse())
        .and_then(|(_, at)| at.location.as_ref())
        .map(|location| location.position * scale);
    for (mut glint, node, at, mut gradient) in &mut panes {
        let corner = at.translation - node.size / 2.0;
        let extent = node.size * node.inverse_scale_factor;
        let inside = cursor
            .map(|point| (point - corner) * node.inverse_scale_factor)
            .filter(|local| node.size.x > 0.0 && local.cmpge(Vec2::ZERO).all() && local.cmple(extent).all());
        let was = glint.strength;
        if let Some(local) = inside {
            if glint.strength < 0.02 {
                glint.at = local;
            }
            let follow = 1.0 - (-dt * 14.0).exp();
            glint.at = glint.at.lerp(local, follow);
        }
        let wanted = if inside.is_some() { 1.0 } else { 0.0 };
        glint.strength += (wanted - glint.strength) * (1.0 - (-dt * 7.0).exp());
        if glint.strength < 0.005 && was < 0.005 {
            continue;
        }
        let mut layers = sheen(28.0, 0.55).0;
        layers.push(Gradient::Radial(RadialGradient::new(
            UiPosition::TOP_LEFT.at_px(glint.at.x, glint.at.y),
            RadialGradientShape::Circle(px(120)),
            vec![
                ColorStop::new(white(0.42 * glint.strength), px(0)),
                ColorStop::new(white(0.12 * glint.strength), px(60)),
                ColorStop::new(white(0.0), px(120)),
            ],
        )));
        gradient.0 = layers;
    }
}

fn update_score(
    mut commands: Commands,
    readout: Res<Readout>,
    mut scores: Query<(&ScoreText, &mut Text, &mut Pop)>,
    boxes: Query<(Entity, &ScoreBox)>,
) {
    for (score, mut shown, mut pop) in &mut scores {
        let old: u32 = shown.0.parse().unwrap_or(0);
        let new = readout.score[score.0.seat()];
        if !set(&mut shown, &new.to_string()) {
            continue;
        }
        pop.age = 0.0;
        if new <= old {
            continue;
        }
        let Some((holder, _)) = boxes.iter().find(|(_, slot)| slot.0 == score.0) else {
            continue;
        };
        commands.spawn((
            Floater {
                age: 0.0,
                colour: ink_of(score.0),
            },
            text("+1", title_font(), 20.0, ink_of(score.0)),
            Node {
                position_type: PositionType::Absolute,
                top: px(4),
                ..default()
            },
            UiTransform::IDENTITY,
            Pickable::IGNORE,
            ChildOf(holder),
        ));
    }
}

fn floaters(mut commands: Commands, time: Res<Time>, mut items: Query<(Entity, &mut Floater, &mut UiTransform, &mut TextColor)>) {
    for (entity, mut item, mut transform, mut ink) in &mut items {
        item.age += time.delta_secs();
        let t = (item.age / 1.1).min(1.0);
        let rise = 1.0 - (1.0 - t).powi(2);
        transform.translation = Val2::px(0.0, 30.0 + rise * 36.0);
        transform.scale = Vec2::splat(1.0 + 0.3 * (1.0 - t));
        ink.0 = item.colour.with_alpha(1.0 - t * t);
        if t >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// easeOutBack: overshoots a little, then settles at 1.
fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    let u = t - 1.0;
    1.0 + c3 * u.powi(3) + c1 * u.powi(2)
}

/// A changed value rolls up into place, swells a little and flashes orange.
fn pops(time: Res<Time>, mut texts: Query<(&mut Pop, &mut UiTransform, &mut TextColor)>) {
    for (mut pop, mut transform, mut ink) in &mut texts {
        if pop.age > 2.0 {
            continue;
        }
        pop.age += time.delta_secs();
        let kick = (-pop.age * 7.0).exp();
        let scale = 1.0 + pop.strength * kick * (pop.age * 16.0).cos().abs();
        let rise = 1.0 - ease_out_back((pop.age / 0.35).min(1.0));
        transform.scale = Vec2::splat(scale);
        transform.translation = Val2::px(0.0, 9.0 * rise);
        ink.0 = pop.base.mix(&ACCENT_INK, kick.min(1.0));
    }
}

/// Springs each on-demand pane toward open or closed; a closed pane leaves the layout.
fn reveal(
    time: Res<Time>,
    chrome: Res<Chrome>,
    readout: Res<Readout>,
    mut panes: Query<(&mut Reveal, &mut Node, &mut UiTransform)>,
) {
    let dt = time.delta_secs().min(1.0 / 20.0);
    for (mut pane, mut node, mut transform) in &mut panes {
        let open = match pane.pane {
            Pane::Help => chrome.help,
            Pane::Toast => !readout.toast.is_empty(),
            Pane::Note => !readout.note.is_empty() && readout.toast.is_empty(),
        };
        let target = if open { 1.0 } else { 0.0 };
        if !open && pane.shown <= 0.001 && node.display == Display::None {
            continue;
        }
        // Opening overshoots like a drop of glass; closing is quick and does not.
        let (stiffness, damping) = if open { (260.0, 19.0) } else { (420.0, 42.0) };
        let force = (target - pane.shown) * stiffness - pane.velocity * damping;
        pane.velocity += force * dt;
        pane.shown += pane.velocity * dt;
        if !open && pane.shown < 0.02 {
            pane.shown = 0.0;
            pane.velocity = 0.0;
        }
        let rest = 1.0 - pane.shown;
        transform.translation = Val2::px(pane.from.x * rest, pane.from.y * rest);
        transform.scale = Vec2::splat(0.86 + 0.14 * pane.shown);
        let display = if open || pane.shown > 0.0 { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
}

/// The fixed glass slides in each time the arena opens.
fn intro(time: Res<Time>, mut clock: ResMut<IntroClock>, mut panels: Query<(&Enter, &mut UiTransform)>) {
    if clock.0 > 3.0 {
        return;
    }
    clock.0 += time.delta_secs();
    for (enter, mut transform) in &mut panels {
        let t = ((clock.0 - enter.delay) / 0.7).clamp(0.0, 1.0);
        let rest = 1.0 - ease_out_back(t);
        transform.translation = Val2::px(enter.from.x * rest, enter.from.y * rest);
    }
}

fn win_flash(
    time: Res<Time>,
    settings: Res<Settings>,
    mut won: MessageReader<Won>,
    mut flashes: Query<(&mut EdgeFlash, &mut BorderColor)>,
) {
    let latest = won.read().last().map(|win| win.player);
    for (mut flash, mut border) in &mut flashes {
        // Fast games win many times a second: only flash when there is time to see it.
        if let Some(player) = latest
            && settings.animated()
            && flash.age > 0.8
        {
            flash.age = 0.0;
            flash.colour = disc(player);
        }
        flash.age += time.delta_secs();
        let alpha = 0.7 * (-flash.age * 2.5).exp();
        let wanted = BorderColor::all(flash.colour.with_alpha(if alpha < 0.01 { 0.0 } else { alpha }));
        if *border != wanted {
            *border = wanted;
        }
    }
}

/// The window size the arena layout is drawn for; other windows scale it to fit.
const DESIGN_WIDTH: f32 = 1600.0;
const DESIGN_HEIGHT: f32 = 920.0;

/// Scales the whole UI with the window (maximised, fullscreen, a projector), so the
/// glass always fits: smaller on a short screen, larger on a big one.
fn fit_ui(windows: Query<&Window, With<bevy::window::PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    let Ok(window) = windows.single() else {
        return;
    };
    if window.width() <= 0.0 || window.height() <= 0.0 {
        return;
    }
    let wanted = (window.width() / DESIGN_WIDTH).min(window.height() / DESIGN_HEIGHT).clamp(0.5, 1.6);
    if (scale.0 - wanted).abs() > 0.005 {
        scale.0 = wanted;
    }
}

/// Tells the camera how much of the window the fixed glass covers. The panes that
/// come and go float over the scene and do not move the camera.
fn measure(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    scale: Res<UiScale>,
    top: Query<&ComputedNode, With<TopBar>>,
    sides: Query<&ComputedNode, With<SidePanel>>,
    bottom: Query<&ComputedNode, With<BottomBar>>,
    mut margins: ResMut<UiMargins>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    // Sizes in window (logical) pixels, whatever the UI scale.
    let logical = |node: &ComputedNode| node.size / window.scale_factor().max(0.01);
    let pad = PAD * scale.0;
    let top = top.iter().map(|node| logical(node).y).fold(0.0, f32::max);
    let side = sides.iter().map(|node| logical(node).x).fold(0.0, f32::max);
    let bottom = bottom.iter().map(|node| logical(node).y).fold(0.0, f32::max);
    if bottom <= 0.0 || window.width() <= 0.0 {
        return;
    }
    let wanted = UiMargins {
        left: pad + side + 14.0,
        right: pad + side + 14.0,
        top: pad + top + 14.0,
        bottom: pad + bottom + 14.0,
    };
    let differs = (wanted.left - margins.left).abs() > 0.5
        || (wanted.right - margins.right).abs() > 0.5
        || (wanted.top - margins.top).abs() > 0.5
        || (wanted.bottom - margins.bottom).abs() > 0.5;
    if differs {
        *margins = wanted;
    }
}
