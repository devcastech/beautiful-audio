use audio_core::metronome::Metronome;
use nice_plug::params::range::IntRange;
use nice_plug::prelude::{
    ActivateContext, AsyncExecutor, AudioIOLayout, AuxiliaryBuffers, Buffer, BufferConfig,
    FloatParam, FloatRange, IntParam, Params, Plugin, ProcessContext, ProcessStatus,
};
use nice_plug_iced::{IcedEditor, IcedEditorState, iced::PollSubNotifier};
use std::num::NonZeroU32;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
mod editor;

pub struct MetronomePlugin {
    params: Arc<MetronomeParams>,
    metronome: Metronome,
    editor_state: Arc<IcedEditorState>,
    notifier: PollSubNotifier,
    current_beat: Arc<AtomicU32>,
}

#[derive(Params)]
struct MetronomeParams {
    #[id = "bpm"]
    bpm: FloatParam,
    #[id = "beats_per_bar"]
    beats_per_bar: IntParam,
}

impl Default for MetronomeParams {
    fn default() -> Self {
        Self {
            bpm: FloatParam::new(
                "BPM",
                120.0,
                FloatRange::Linear {
                    min: 30.0,
                    max: 300.0,
                },
            )
            .with_step_size(1.0)
            .with_unit(" BPM"),
            beats_per_bar: IntParam::new("BEATS_PER_BAR", 4, IntRange::Linear { min: 1, max: 8 })
                .with_unit(" BEATS PER BAR"),
        }
    }
}

impl Default for MetronomePlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(MetronomeParams::default()),
            metronome: Metronome::new(48_000.0, 120.0),
            editor_state: editor::default_state(),
            notifier: PollSubNotifier::new(),
            current_beat: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl Plugin for MetronomePlugin {
    const NAME: &'static str = "Metronome";
    const VENDOR: &'static str = "Ed";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    type Editor = IcedEditor;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn activate(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.metronome = Metronome::new(buffer_config.sample_rate, 120.0);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let channels = buffer.as_slice();
        let (left, others) = channels.split_first_mut().unwrap();
        self.metronome.bpm = self.params.bpm.value();
        self.metronome.beats_per_bar = self.params.beats_per_bar.value() as u32;
        self.metronome.process(left);
        self.current_beat
            .store(self.metronome.beat(), Ordering::Relaxed);
        self.notifier.notify();
        for ch in others {
            ch.copy_from_slice(left);
        }
        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        editor::create(
            self.editor_state.clone(),
            self.notifier.clone(),
            editor::EditorData {
                params: self.params.clone(),
                current_beat: self.current_beat.clone(),
            },
        )
    }
}
