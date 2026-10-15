use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use nice_plug::editor::dpi::LogicalSize;
use nice_plug_iced::{
    IcedEditor, IcedEditorState, IcedNiceContext, IcedNiceSettings, PersistentState, application,
    create_iced_editor,
    iced::{
        Border, Center, Color, Element, Fill, Length, Point, PollSubNotifier, Rectangle, Renderer,
        Subscription, Theme,
        alignment::Vertical,
        mouse,
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

struct BeatIndicator {
    current_beat: Arc<AtomicU32>,
    beats_per_bar: u32,
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

            if i == beat {
                frame.fill(&circle, palette.primary.base.color);
            } else {
                frame.fill(&circle, palette.background.strong.color);
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

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Poll,
    BpmGesture(Gesture),
    VolumeGesture(Gesture),
    BeatsDown,
    BeatsUp,
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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let bpm = self.data.params.bpm.value();
        let beats_per_bar = self.data.params.beats_per_bar.value();

        let tempo = container(
            column![
                row![
                    text(format!("{bpm:.0}")).size(48),
                    text("BPM").size(12).style(screen_label)
                ]
                .align_y(Vertical::Center)
                .spacing(10),
            ]
            .spacing(8)
            .align_x(Center),
        )
        .padding(12)
        .width(Fill)
        .center_x(Fill)
        .style(screen);
        //.style(panel);

        let bpm_knob = container(
            row![
                labeled_knob(&self.data.params.bpm, "TEMPO", Message::BpmGesture),
                labeled_knob(&self.data.params.volume, "VOLUMEN", Message::VolumeGesture),
            ]
            .spacing(32),
        )
        .padding(12)
        .width(Fill)
        .center_x(Fill);

        let bar = container(
            column![
                canvas(BeatIndicator {
                    current_beat: self.data.current_beat.clone(),
                    beats_per_bar: beats_per_bar as u32,
                })
                .width(Fill)
                .height(28),
                row![
                    button(text("-"))
                        .on_press(Message::BeatsDown)
                        .style(flat_button),
                    text(format!("{beats_per_bar}")).size(24),
                    text("/ 4").size(18),
                    button(text("+"))
                        .on_press(Message::BeatsUp)
                        .style(flat_button),
                ]
                .align_y(Vertical::Center)
                .spacing(10),
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
                background: Color::from_rgb8(0x24, 0x21, 0x2e), // gris oscuro con tinte morado
                text: Color::from_rgb8(0xe7, 0xe3, 0xf0),       // blanco lavanda
                primary: Color::from_rgb8(0xa7, 0x8b, 0xfa),    // morado de acento
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
            empty_color: palette.background.strong.color,
            filled_color: palette.primary.base.color,
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

// tarjeta con fondo un poco más claro, esquinas redondeadas y borde sutil
fn panel(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        ..container::Style::default()
    }
}

fn screen(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Color::from_rgb8(0x13, 0x11, 0x1a).into()),
        text_color: Some(palette.primary.base.color),
        border: Border {
            radius: 6.0.into(),
            width: 2.0,
            color: Color::from_rgb8(0x0c, 0x0a, 0x10),
        },
        ..container::Style::default()
    }
}

// etiqueta pequeña y apagada para el título de cada sección
fn label(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().background.strong.text),
    }
}

// texto secundario dentro de la pantalla: el color de acento, apagado
fn screen_label(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(
            theme
                .extended_palette()
                .primary
                .base
                .color
                .scale_alpha(0.75),
        ),
    }
}

// perilla con su etiqueta debajo, como la serigrafía de un equipo físico
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
        text(label).size(11).style(screen_label),
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

fn flat_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let background = match status {
        button::Status::Hovered => Some(palette.background.strong.color.into()),
        button::Status::Pressed => Some(palette.background.stronger.color.into()),
        _ => None,
    };
    button::Style {
        background,
        text_color: palette.background.base.text,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        ..button::Style::default()
    }
}
