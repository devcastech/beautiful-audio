use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use nice_plug::editor::dpi::LogicalSize;
use nice_plug_iced::{
    IcedEditor, IcedEditorState, IcedNiceContext, IcedNiceSettings, PersistentState, application,
    create_iced_editor,
    iced::{
        Border, Center, Color, Degrees, Element, Fill, Length, Padding, Point, PollSubNotifier,
        Rectangle, Renderer, Size, Subscription, Theme,
        alignment::Vertical,
        gradient, mouse,
        theme::Palette,
        widget::{button, canvas, column, container, row, stack, text},
    },
};

use iced_audio::{
    Gesture, Knob,
    param::{nice_to_iced, set_nice_param},
    style::knob as knob_style,
};

use nice_plug::prelude::FloatParam;

use crate::MetronomeParams;

// colores del equipo: gris carbón neutro y rojo solo para lo que está encendido
const SOCKET: Color = Color::from_rgb8(0x1c, 0x1c, 0x1c);
const HALO: Color = Color::from_rgb8(0x22, 0x22, 0x22);
const HALO_WIDTH: f32 = 2.0;
const KEY_BASE: Color = Color::from_rgb8(0x19, 0x19, 0x19);
const WELL: Color = Color::from_rgb8(0x1a, 0x1a, 0x1a);
const KEY_TOP: Color = Color::from_rgb8(0x2c, 0x2c, 0x2c);
const KEY: Color = Color::from_rgb8(0x26, 0x26, 0x26);
const KEY_HOVER_TOP: Color = Color::from_rgb8(0x31, 0x31, 0x31);
const KEY_HOVER: Color = Color::from_rgb8(0x2a, 0x2a, 0x2a);
const KEY_SUNK: Color = Color::from_rgb8(0x21, 0x21, 0x21);
// LED apagado: apenas más oscuro que el panel, para que el encendido destaque
const LED_OFF: Color = Color::from_rgb8(0x1c, 0x1c, 0x1c);

struct BeatIndicator {
    current_beat: Arc<AtomicU32>,
    beats_per_bar: u32,
    playing: bool,
}

impl canvas::Program<Message> for BeatIndicator {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let beat = self.current_beat.load(Ordering::Relaxed);
        let palette = theme.extended_palette();

        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let slot = bounds.width / self.beats_per_bar as f32;
        let radius = slot.min(bounds.height) * 0.2;

        for i in 0..self.beats_per_bar {
            let center = Point::new(slot * (i as f32 + 0.5), bounds.height / 2.0);
            let circle = canvas::Path::circle(center, radius);

            if self.playing && i == beat {
                frame.fill(&circle, palette.primary.base.color);
            } else {
                frame.fill(&circle, LED_OFF);
            }
        }

        vec![frame.into_geometry()]
    }
}

struct Screws;

impl canvas::Program<Message> for Screws {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let palette = theme.extended_palette();
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let margin = 10.0;
        let radius = 4.0;
        let corners = [
            Point::new(margin, margin),
            Point::new(bounds.width - margin, margin),
            Point::new(margin, bounds.height - margin),
            Point::new(bounds.width - margin, bounds.height - margin),
        ];

        for center in corners {
            // cabeza del tornillo
            frame.fill(
                &canvas::Path::circle(center, radius),
                palette.background.strong.color.scale_alpha(0.6),
            );
            // ranura en diagonal, de borde a borde de la cabeza
            let offset = radius * 0.7;
            let slot = canvas::Path::line(
                Point::new(center.x - offset, center.y - offset),
                Point::new(center.x + offset, center.y + offset),
            );
            frame.stroke(
                &slot,
                canvas::Stroke::default()
                    .with_width(1.5)
                    .with_color(palette.background.base.color),
            );
        }

        vec![frame.into_geometry()]
    }
}

struct PlayIcon {
    playing: bool,
}

impl canvas::Program<Message> for PlayIcon {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let palette = theme.extended_palette();
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let side = bounds.width.min(bounds.height);
        let origin = Point::new((bounds.width - side) / 2.0, (bounds.height - side) / 2.0);

        if self.playing {
            let inset = side * 0.1;
            let square = canvas::Path::rectangle(
                Point::new(origin.x + inset, origin.y + inset),
                Size::new(side - inset * 2.0, side - inset * 2.0),
            );
            frame.fill(&square, palette.background.base.text);
        } else {
            let triangle = canvas::Path::new(|b| {
                b.move_to(Point::new(origin.x + side * 0.15, origin.y));
                b.line_to(Point::new(origin.x + side * 0.95, origin.y + side / 2.0));
                b.line_to(Point::new(origin.x + side * 0.15, origin.y + side));
                b.close();
            });
            frame.fill(&triangle, palette.background.base.text);
        }

        vec![frame.into_geometry()]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Poll,
    BpmGesture(Gesture),
    VolumeGesture(Gesture),
    BeatsDown,
    BeatsUp,
    TogglePlayStop,
}

pub struct EditorData {
    pub params: Arc<MetronomeParams>,
    pub current_beat: Arc<AtomicU32>,
}

struct Gui {
    data: PersistentState<EditorData>,
    ctx: IcedNiceContext,
}

impl Gui {
    fn new(data: PersistentState<EditorData>, ctx: IcedNiceContext) -> Self {
        Self { data, ctx }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Poll => {}
            //Message::Poll => {
            //    eprintln!("poll {}", self.data.current_beat.load(Ordering::Relaxed));
            //}
            Message::BpmGesture(gesture) => {
                let setter = self.ctx.nice_context.param_setter();
                set_nice_param(&self.data.params.bpm, gesture, &setter);
            }
            Message::VolumeGesture(gesture) => {
                let setter = self.ctx.nice_context.param_setter();
                set_nice_param(&self.data.params.volume, gesture, &setter);
            }
            Message::BeatsDown => {
                let setter = self.ctx.nice_context.param_setter();
                let beats = &self.data.params.beats_per_bar;
                setter.begin_set_parameter(beats);
                setter.set_parameter(beats, beats.value() - 1);
                setter.end_set_parameter(beats);
            }
            Message::BeatsUp => {
                let setter = self.ctx.nice_context.param_setter();
                let beats = &self.data.params.beats_per_bar;
                setter.begin_set_parameter(beats);
                setter.set_parameter(beats, beats.value() + 1);
                setter.end_set_parameter(beats);
            }
            Message::TogglePlayStop => {
                let setter = self.ctx.nice_context.param_setter();
                let is_playing = &self.data.params.is_playing;
                let toggle = !is_playing.value();
                setter.begin_set_parameter(is_playing);
                setter.set_parameter(is_playing, toggle);
                setter.end_set_parameter(is_playing);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let bpm = self.data.params.bpm.value();
        let beats_per_bar = self.data.params.beats_per_bar.value();
        let playing = self.data.params.is_playing.value();

        let tempo = container(
            column![
                row![
                    text(format!("{bpm:.0}")).size(56).style(display),
                    text("BPM").size(14).style(label)
                ]
                .align_y(Vertical::Center)
                .spacing(10),
            ]
            .spacing(8)
            .align_x(Center),
        )
        .padding(12)
        .width(Fill)
        .center_x(Fill);

        let bpm_knob = container(
            row![
                labeled_knob(&self.data.params.bpm, "TEMPO", Message::BpmGesture),
                key(
                    canvas(PlayIcon { playing }).width(Fill).height(Fill),
                    Message::TogglePlayStop,
                    48.0,
                    playing,
                ),
                labeled_knob(&self.data.params.volume, "VOLUMEN", Message::VolumeGesture),
            ]
            .align_y(Vertical::Center)
            .spacing(20),
        )
        .padding(12)
        .width(Fill)
        .center_x(Fill);

        let bar = container(
            column![
                canvas(BeatIndicator {
                    current_beat: self.data.current_beat.clone(),
                    beats_per_bar: beats_per_bar as u32,
                    playing,
                })
                .width(Fill)
                .height(28),
                row![
                    key(text("−").size(18).center(), Message::BeatsDown, 32.0, false),
                    text(format!("{beats_per_bar} / 4")).size(22),
                    key(text("+").size(18).center(), Message::BeatsUp, 32.0, false),
                ]
                .align_y(Vertical::Center)
                .spacing(16),
            ]
            .spacing(8)
            .align_x(Center),
        )
        .padding(12)
        .width(Fill);
        //.style(panel);

        let content = container(column![tempo, bpm_knob, bar].spacing(12))
            .padding(12)
            .center_x(Fill)
            .center_y(Fill);
        //.into();
        stack![canvas(Screws).width(Fill).height(Fill), content,].into()
    }

    fn theme(&self) -> Option<Theme> {
        Some(Theme::custom(
            "Metronome",
            Palette {
                background: Color::from_rgb8(0x26, 0x26, 0x26), // gris carbón
                text: Color::from_rgb8(0xa8, 0xa8, 0xa8),       // gris claro, sin brillo
                primary: Color::from_rgb8(0xe8, 0x10, 0x10),    // rojo LED
                success: Color::from_rgb8(0x7f, 0xd1, 0xae),
                warning: Color::from_rgb8(0xf2, 0xc4, 0x6d),
                danger: Color::from_rgb8(0xf0, 0x7b, 0x8c),
            },
        ))
    }

    fn subscription(&self) -> Subscription<Message> {
        nice_plug_iced::iced::poll_events().map(|_| Message::Poll)
    }
}

// perilla moderna: un arco que se llena con el valor
struct KnobStyle;

impl knob_style::StyleSheet for KnobStyle {
    type Style = Theme;

    fn idle(&self, theme: &Theme) -> knob_style::Appearance {
        let palette = theme.extended_palette();
        knob_style::Appearance::Arc(knob_style::ArcAppearance {
            width: knob_style::StyleLength::Fixed(5.0),
            empty_color: SOCKET,
            filled_color: Color::from_rgb8(0xc8, 0x86, 0x2a),
            notch: knob_style::NotchShape::Line(knob_style::LineNotch {
                color: palette.background.base.text,
                width: knob_style::StyleLength::Fixed(3.0),
                length: knob_style::StyleLength::Scaled(0.2),
                cap: canvas::LineCap::Round,
                offset: knob_style::StyleLength::Scaled(0.2),
            }),
            cap: canvas::LineCap::Round,
        })
    }
}

fn display(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().text),
    }
}

// etiqueta pequeña y apagada, como serigrafía sobre el panel
fn label(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(
            theme
                .extended_palette()
                .background
                .base
                .text
                .scale_alpha(0.6),
        ),
    }
}

fn labeled_knob<'a>(
    param: &FloatParam,
    label: &'a str,
    on_gesture: fn(Gesture) -> Message,
) -> Element<'a, Message> {
    column![
        Knob::new(nice_to_iced(param))
            .on_gesture(on_gesture)
            .style(KnobStyle)
            .size(Length::Fixed(56.0)),
        text(label).size(11).style(self::label),
    ]
    .spacing(8)
    .align_x(Center)
    .into()
}

pub fn default_state() -> Arc<IcedEditorState> {
    IcedEditorState::from_size(LogicalSize::new(300.0, 380.0), 1.0)
}

pub fn create(
    editor_state: Arc<IcedEditorState>,
    notifier: PollSubNotifier,
    data: EditorData,
) -> Option<IcedEditor> {
    create_iced_editor(
        editor_state,
        data,
        notifier,
        IcedNiceSettings::new().with_always_redraw(true),
        |data, ctx| {
            Ok(application(data, ctx, Gui::new, Gui::update, Gui::view)
                .theme(Gui::theme)
                .subscription(Gui::subscription)
                .run())
        },
    )
}

fn key<'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: Message,
    size: f32,
    latched: bool,
) -> Element<'a, Message> {
    let ring = well_ring(size);
    // sombra dura de la tecla: una base oscura que asoma por debajo. enganchada,
    // la tecla baja y la base asoma por arriba, como el canto de un hueco
    let drop = (size / 16.0).max(2.0).round();
    let (above, below) = if latched { (drop, 0.0) } else { (0.0, drop) };

    container(
        container(
            container(
                button(container(content).center(Fill))
                    .on_press(on_press)
                    .width(Length::Fixed(size))
                    .height(Length::Fixed(size))
                    .padding(size * 0.3)
                    .style(move |theme, status| key_style(theme, status, size, latched)),
            )
            .padding(Padding {
                top: above,
                right: 0.0,
                bottom: below,
                left: 0.0,
            })
            .style(move |_theme| key_base(size)),
        )
        .padding(Padding {
            top: ring,
            right: ring,
            bottom: ring - drop / 2.0,
            left: ring,
        })
        .style(move |theme| fading(theme, WELL, size * 0.25 + ring)),
    )
    .padding(HALO_WIDTH)
    .style(move |theme| fading(theme, HALO, size * 0.25 + ring + HALO_WIDTH))
    .into()
}

fn key_base(size: f32) -> container::Style {
    container::Style {
        background: Some(KEY_BASE.into()),
        border: Border {
            radius: (size * 0.25).into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

fn well_ring(size: f32) -> f32 {
    (size / 12.0).max(3.0)
}

// capa redondeada que arriba tiene `color` y hacia abajo se funde con el panel,
// como la sombra de una luz que viene de arriba
fn fading(theme: &Theme, color: Color, radius: f32) -> container::Style {
    let panel = theme.extended_palette().background.base.color;
    let background = gradient::Linear::new(Degrees(180.0))
        .add_stop(0.0, color)
        .add_stop(1.0, panel);
    container::Style {
        background: Some(background.into()),
        border: Border {
            radius: radius.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

// tecla con un degradado leve (más clara arriba) que la hace ver abombada;
// enganchada o pulsada queda plana y más oscura, como si se hundiera
fn key_style(theme: &Theme, status: button::Status, size: f32, latched: bool) -> button::Style {
    let palette = theme.extended_palette();
    let sunk = latched || status == button::Status::Pressed;

    let (top, bottom) = if sunk {
        (KEY_SUNK, KEY_SUNK)
    } else if status == button::Status::Hovered {
        (KEY_HOVER_TOP, KEY_HOVER)
    } else {
        (KEY_TOP, KEY)
    };
    let background = gradient::Linear::new(Degrees(180.0))
        .add_stop(0.0, top)
        .add_stop(1.0, bottom);
    button::Style {
        background: Some(background.into()),
        text_color: palette.background.base.text,
        border: Border {
            radius: (size * 0.25).into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}
