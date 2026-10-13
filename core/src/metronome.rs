use std::f32::consts::PI;

pub struct Metronome {
    sample_rate: f32,
    pub bpm: f32,
    pub beats_per_bar: u32,
    counter: u64, // muestras desde el último beat
    beat: u32,
    click_pos: Option<usize>,
    volume: f32,
}

impl Metronome {
    pub fn new(sample_rate: f32, bpm: f32) -> Self {
        Self {
            sample_rate, // 48000
            bpm,
            beats_per_bar: 4,
            counter: u64::MAX,
            beat: 0,
            click_pos: None,
            volume: 0.5,
        }
    }
    pub fn process(&mut self, out: &mut [f32]) {
        let period = (self.sample_rate * 60.0 / self.bpm) as u64;
        for s in out.iter_mut() {
            if self.counter >= period {
                self.counter = 0;
                self.beat = (self.beat + 1) % self.beats_per_bar;
                self.click_pos = Some(0);
            }
            *s = self.next_click_sample();
            // seno corto con envolvente; más agudo si beat == 0
            self.counter += 1;
        }
    }
    fn next_click_sample(&mut self) -> f32 {
        let duration = (self.sample_rate * 0.020) as usize;
        if let Some(pos) = self.click_pos {
            if pos < duration {
                self.click_pos = Some(pos + 1);
                let time = pos as f32 / self.sample_rate;
                let wave = (2.0 * PI * 1000.0 * time).sin();

                return wave * self.volume;
                //return 1.0 * self.volume;
            } else {
                self.click_pos = None
            }
        }
        0.0
    }

    //fn next_click_sample(&mut self) -> f32 {
    //    let duration = (self.sample_rate * 0.020) as usize;
    //    if let Some(pos) = self.click_pos {
    //        if pos < duration {
    //            self.click_pos = Some(pos + 1);
    //            return 1.0 * self.volume;
    //        } else {
    //            self.click_pos = None
    //        }
    //    }
    //    0.0
    //}

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
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
}
