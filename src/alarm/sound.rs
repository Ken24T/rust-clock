//! Audio playback for alarm sounds using rodio.
//!
//! Plays an embedded default chime sound, or a user-specified file.

use std::path::Path;

use rodio::{Decoder, OutputStream, Sink};

/// Embedded default alarm sound (a short sine-wave chime generated at build time
/// is not practical, so we synthesise a simple beep pattern at runtime).
///
/// In future this could load from `assets/alarm.wav`.
pub fn play_alarm_sound(custom_path: Option<&Path>) {
    // Spawn a thread so we don't block the UI.
    let path = custom_path.map(|p| p.to_path_buf());
    std::thread::spawn(move || {
        if let Err(e) = play_sound_blocking(path.as_deref()) {
            eprintln!("Failed to play alarm sound: {e}");
        }
    });
}

/// Play the sound synchronously (called from a background thread).
fn play_sound_blocking(custom_path: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    if let Some(path) = custom_path {
        // User-specified sound file.
        let file = std::fs::File::open(path)?;
        let source = Decoder::new(std::io::BufReader::new(file))?;
        sink.append(source);
    } else {
        // Generate a simple beep pattern: three short sine-wave bursts.
        play_generated_beeps(&sink);
    }

    sink.sleep_until_end();
    Ok(())
}

/// Generate three short beep tones as a fallback alarm sound.
fn play_generated_beeps(sink: &Sink) {
    use rodio::source::{SineWave, Source};

    for _ in 0..3 {
        let beep = SineWave::new(880.0)
            .take_duration(std::time::Duration::from_millis(200))
            .amplify(0.4);
        sink.append(beep);

        // Short gap between beeps.
        let silence = SineWave::new(0.0)
            .take_duration(std::time::Duration::from_millis(150))
            .amplify(0.0);
        sink.append(silence);
    }
}
