use std::sync::Arc;

use nice_plug::editor::dpi::LogicalSize;
use nice_plug_iced::{
    IcedEditor, IcedEditorState, IcedNiceContext, IcedNiceSettings, PersistentState, application,
    create_iced_editor,
    iced::{
        Center, PollSubNotifier, Subscription, Theme,
        widget::{Column, column, slider, text},
    },
};

use crate::MetronomeParams;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Poll,
    BpmChanged(f32),
    BpmReleased,
}

pub struct EditorData {
    pub params: Arc<MetronomeParams>,
}

struct Gui {
    data: PersistentState<EditorData>,
    ctx: IcedNiceContext,
    dragging: bool,
}

impl Gui {
    fn new(data: PersistentState<EditorData>, ctx: IcedNiceContext) -> Self {
        Self {
            data,
            ctx,
            dragging: false,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Poll => {}
            Message::BpmChanged(value) => {
                let setter = self.ctx.nice_context.param_setter();
                let bpm = &self.data.params.bpm;
                if !self.dragging {
                    setter.begin_set_parameter(bpm);
                    self.dragging = true;
                }
                setter.set_parameter(bpm, value);
            }
            Message::BpmReleased => {
                if self.dragging {
                    let setter = self.ctx.nice_context.param_setter();
                    setter.end_set_parameter(&self.data.params.bpm);
                    self.dragging = false;
                }
            }
        }
    }

    fn view(&self) -> Column<'_, Message> {
        let bpm = self.data.params.bpm.value();
        column![
            text(format!("{bpm:.0}")).size(96),
            text("BPM").size(24),
            slider(30.0..=300.0, bpm, Message::BpmChanged)
                .step(1.0_f32)
                .on_release(Message::BpmReleased),
        ]
        .padding(20)
        .align_x(Center)
    }

    fn theme(&self) -> Option<Theme> {
        Some(Theme::Dark)
    }

    fn subscription(&self) -> Subscription<Message> {
        nice_plug_iced::iced::poll_events().map(|_| Message::Poll)
    }
}

pub fn default_state() -> Arc<IcedEditorState> {
    IcedEditorState::from_size(LogicalSize::new(300.0, 220.0), 1.0)
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
        IcedNiceSettings::new(),
        |data, ctx| {
            Ok(application(data, ctx, Gui::new, Gui::update, Gui::view)
                .theme(Gui::theme)
                .subscription(Gui::subscription)
                .run())
        },
    )
}
