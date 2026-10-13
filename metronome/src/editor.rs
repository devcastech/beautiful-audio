use std::sync::Arc;

use nice_plug::editor::dpi::LogicalSize;
use nice_plug_iced::{
    IcedEditor, IcedEditorState, IcedNiceContext, IcedNiceSettings, PersistentState, application,
    create_iced_editor,
    iced::{
        Center, Element, Fill, PollSubNotifier, Subscription, Theme,
        widget::{column, container, text},
    },
};

use iced_audio::{
    Gesture, Knob,
    param::{nice_to_iced, set_nice_param},
};

use crate::MetronomeParams;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Poll,
    BpmGesture(Gesture),
}

pub struct EditorData {
    pub params: Arc<MetronomeParams>,
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
            Message::BpmGesture(gesture) => {
                let setter = self.ctx.nice_context.param_setter();
                set_nice_param(&self.data.params.bpm, gesture, &setter);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let bpm = self.data.params.bpm.value();

        container(
            column![
                text(format!("{bpm:.0}")).size(96),
                text("BPM").size(24),
                Knob::new(nice_to_iced(&self.data.params.bpm)).on_gesture(Message::BpmGesture),
            ]
            .spacing(10)
            .align_x(Center),
        )
        .padding(10)
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }

    fn theme(&self) -> Option<Theme> {
        Some(Theme::Dark)
    }

    fn subscription(&self) -> Subscription<Message> {
        nice_plug_iced::iced::poll_events().map(|_| Message::Poll)
    }
}

pub fn default_state() -> Arc<IcedEditorState> {
    IcedEditorState::from_size(LogicalSize::new(300.0, 300.0), 1.0)
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
