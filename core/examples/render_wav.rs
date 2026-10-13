use audio_core::metronome::Metronome;

const SAMPLE_RATE: u32 = 48_000;
const BPM: f32 = 120.0;
const SECONDS: usize = 4;
const OUTPUT: &str = "metronome.wav";

fn main() -> Result<(), hound::Error> {
    let mut metronome = Metronome::new(SAMPLE_RATE as f32, BPM)
        .with_volume(0.6)
        .with_beats_per_bar(2);
    let mut buf = vec![0.0; SAMPLE_RATE as usize * SECONDS];
    metronome.process(&mut buf);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(OUTPUT, spec)?;
    for &sample in &buf {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;

    println!("{OUTPUT}: {SECONDS} s a {BPM} BPM");
    Ok(())
}
