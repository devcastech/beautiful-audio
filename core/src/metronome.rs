use std::f32::consts::PI;

pub struct Metronome {
    sample_rate: f32,
    pub bpm: f32,
    pub beats_per_bar: u32,
    counter: u64,
    beat: u32,
    click_pos: Option<usize>,
    pub volume: f32,
    playing: bool,
}

impl Metronome {
    pub fn new(sample_rate: f32, bpm: f32) -> Self {
        let beats_per_bar = 4;
        let volume = 0.5;
        Self {
            sample_rate, // 48000
            bpm,
            beats_per_bar,
            counter: u64::MAX,
            beat: beats_per_bar - 1,
            click_pos: None,
            volume,
            playing: true,
        }
    }
    pub fn process(&mut self, out: &mut [f32]) {
        if !self.playing {
            for s in out.iter_mut() {
                *s = 0.0
            }
            return;
        }
        let period = (self.sample_rate * 60.0 / self.bpm) as u64;
        for s in out.iter_mut() {
            if self.counter >= period {
                self.counter = 0;
                self.beat = (self.beat + 1) % self.beats_per_bar;
                self.click_pos = Some(0);
            }
            *s = self.next_click_sample();
            self.counter += 1;
        }
    }
    fn next_click_sample(&mut self) -> f32 {
        let duration = (self.sample_rate * 0.020) as usize;
        if let Some(pos) = self.click_pos {
            if pos < duration {
                self.click_pos = Some(pos + 1);
                let time = pos as f32 / self.sample_rate;
                let freq = if self.beat == 0 { 1500.0 } else { 1000.0 };
                let wave = (2.0 * PI * freq * time).sin();
                let envelope = 1.0 - pos as f32 / duration as f32;
                return wave * envelope * self.volume;
            } else {
                self.click_pos = None
            }
        }
        0.0
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn with_beats_per_bar(mut self, beats_per_bar: u32) -> Self {
        self.beats_per_bar = beats_per_bar.max(1);
        self.beat = self.beats_per_bar - 1;
        self
    }

    pub fn beat(&self) -> u32 {
        self.beat
    }

    pub fn stop(&mut self) {
        if !self.playing {
            return;
        }
        self.playing = false;
        self.counter = u64::MAX;
        self.click_pos = None;
    }

    pub fn play(&mut self) {
        if self.playing {
            return;
        }
        self.beat = self.beats_per_bar - 1;
        self.playing = true;
    }
    pub fn is_playing(&self) -> bool {
        self.playing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_clicks(buf: &[f32]) -> Vec<usize> {
        let mut starts = Vec::new();
        if buf[0] != 0.0 {
            starts.push(0);
        }

        for i in 1..buf.len() {
            if buf[i] != 0.0 && buf[i - 1] == 0.0 {
                starts.push(i);
            }
        }
        starts
    }

    #[test]
    fn clicks_each_24000_samples_to_120bpm() {
        let mut m = Metronome::new(48_000.0, 120.0);
        let mut buf = vec![0.0; 48_000 * 3];

        m.process(&mut buf);
        let starts = start_clicks(&buf);
        println!("starts: {:?}", starts);

        assert!(starts.len() >= 2, "expected at least 2 clicks");
        assert!(starts[0] < 10);
        for pairs in starts.windows(2) {
            assert_eq!(pairs[1] - pairs[0], 24_000);
        }
    }

    #[test]
    fn clicks_each_48000_samples_to_60bpm() {
        let mut m = Metronome::new(48_000.0, 60.0);
        let mut buf = vec![0.0; 48_000 * 3];

        m.process(&mut buf);
        let starts = start_clicks(&buf);
        println!("starts: {:?}", starts);

        assert!(starts.len() >= 2, "expected at least 2 clicks");
        assert!(starts[0] < 10);
        for pairs in starts.windows(2) {
            assert_eq!(pairs[1] - pairs[0], 48_000);
        }
    }

    #[test]
    fn play_stop_toggle() {
        let mut m = Metronome::new(48_000.0, 60.0);
        let mut buf = vec![0.0; 48_000 * 3];

        m.process(&mut buf);
        assert!(m.is_playing(), "expected is_playing");
        assert_ne!(m.beat(), 0, "expected beat > 0");

        m.stop();
        assert!(!m.is_playing(), "expected is_playing == false");
        m.play();
        assert!(m.is_playing(), "expected is_playing");
    }

    #[test]
    fn stopped_outputs_silence() {
        let mut m = Metronome::new(48_000.0, 60.0);
        let mut buf = vec![0.0; 48_000];

        m.process(&mut buf);
        m.stop();
        let mut buf2 = vec![1.0; 10000];
        m.process(&mut buf2);
        assert!(buf2.iter().all(|e| *e == 0.0), "expected to be 0.0");
    }

    #[test]
    fn can_restart_after_play() {
        let mut m = Metronome::new(48_000.0, 120.0);
        let mut buf = vec![0.0; 30_000];
        m.process(&mut buf);
        m.stop();
        m.play();
        let mut buf2 = vec![0.0; 48_000];
        m.process(&mut buf2);
        let starts = start_clicks(&buf2);
        assert!(starts.len() >= 2, "expected at least 2 clicks");
        assert!(starts[0] < 10);
        assert_eq!(m.beat(), 1, "expected beat == 1");
    }

    #[test]
    fn repeated_play_keeps_rhythm() {
        let mut m = Metronome::new(48_000.0, 120.0);
        let mut buf = vec![0.0; 48_000 * 3];

        for chunk in buf.chunks_mut(512) {
            m.play();
            m.process(chunk);
        }

        let starts = start_clicks(&buf);
        assert_eq!(starts.len(), 6, "expected 6 clicks");
        for pairs in starts.windows(2) {
            assert_eq!(pairs[1] - pairs[0], 24_000);
        }
        assert_eq!(m.beat(), 1);
    }
}
