//! A cache system for storing preview MP3s from Deezer, and retrieving clips from them.
use std::{ops::Range, sync::OnceLock};

use eyre::{Context, Result};
use futures::Stream;
use futures::TryStreamExt;
use minimp3_fixed as minimp3;
use rocket::{
    http::hyper::body::Bytes,
    tokio::{fs, task},
};

use crate::deezer;

/// The config for the music cache system, set on startup.
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Config options for the music cache system.
#[derive(Debug)]
struct Config {
    /// The directory where music files are stored.
    pub music_dir: std::path::PathBuf,
}

/// Initialise the music cache system using the given config.
///
/// Must only be called once.
pub fn init(config: &crate::Config) {
    CONFIG
        .set(config.into())
        .map_err(|_| ())
        .expect("track::music::init must only be called once");
}

impl From<&crate::Config> for Config {
    fn from(config: &crate::Config) -> Self {
        let music_dir = config.media_dir.join("music");
        // fine to use blocking API here, only called on startup
        std::fs::create_dir_all(&music_dir).expect("failed to create music directory");
        Self { music_dir }
    }
}

/// Transcode a downloaded track from MP3 to WAV, and save it.
async fn save_track(
    path: std::path::PathBuf,
    mp3_stream: impl Stream<Item = Result<Bytes>> + Send,
) -> Result<()> {
    let mp3_read = Box::pin(tokio_util::io::StreamReader::new(
        mp3_stream.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
    ));
    let mut decoder = minimp3::Decoder::new(mp3_read);
    let first_frame = decoder
        .next_frame_future()
        .await
        .wrap_err("error reading first frame of MP3")?;
    let wav_spec = hound::WavSpec {
        channels: u16::try_from(first_frame.channels).wrap_err("invalid channel count")?,
        sample_rate: u32::try_from(first_frame.sample_rate).wrap_err("negative sample rate")?,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let file = std::fs::File::create(path).wrap_err("error creating a file for decoded music")?;
    let mut writer = hound::WavWriter::new(std::io::BufWriter::new(file), wav_spec)
        .wrap_err("error creating a WAV writer")?;
    let mut maybe_frame = Ok(first_frame);
    loop {
        let frame = match maybe_frame {
            Ok(frame) => frame,
            Err(minimp3::Error::Eof) => return writer.finalize().map_err(Into::into),
            Err(err) => return Err(err.into()),
        };
        for sample in frame.data {
            writer
                .write_sample(sample)
                .wrap_err("error writing a sample to a WAV file")?;
        }
        maybe_frame = decoder.next_frame_future().await;
    }
}

/// Download a track from Deezer and save it to the music cache.
async fn download_track(config: &Config, track_id: u32, preview: &str) -> Result<()> {
    let data = deezer::track_preview(preview).await?;
    let path = config.music_dir.join(format!("{track_id}.wav"));
    save_track(path, data).await
}

/// Ensure that a given track is cached, and return the path.
async fn ensure_cached(track_id: u32, preview: &str) -> Result<std::path::PathBuf> {
    let config = CONFIG
        .get()
        .expect("music system used before initialisation");
    let path = config.music_dir.join(format!("{track_id}.wav"));
    if !fs::try_exists(&path)
        .await
        .wrap_err("error checking if a track is cached")?
    {
        download_track(config, track_id, preview).await?;
    }
    Ok(path)
}

/// Get a clip from a track (blocking).
fn blocking_clip_track(path: std::path::PathBuf, time: Range<chrono::Duration>) -> Result<Vec<u8>> {
    let start = u32::try_from(time.start.num_milliseconds())
        .expect("start time to be positive and not overflow");
    let length = u32::try_from((time.end - time.start).num_milliseconds())
        .expect("clip length to be positive and not overflow");
    let mut reader = hound::WavReader::open(path).wrap_err("error opening a cached track")?;
    let spec = reader.spec();
    reader
        .seek(spec.sample_rate * start / 1000)
        .wrap_err("error seeking within a cached track")?;
    let mut buf = Vec::new();
    let cursor = std::io::Cursor::new(&mut buf);
    let mut writer = hound::WavWriter::new(cursor, spec).wrap_err("error creating a WAV writer")?;
    assert_eq!(spec.channels, 2, "Deezer should return stereo music");
    let sample_rate = usize::try_from(spec.sample_rate).expect("sample rate should fit in usize");
    let samples_to_read = usize::from(spec.channels)
        * sample_rate
        * usize::try_from(length).expect("length in ms should fit in usize")
        / 1000;
    let mut samples_read = 0;
    for sample in reader.samples::<i16>().take(samples_to_read) {
        let sample = sample.wrap_err("could not read sample from track")?;
        writer
            .write_sample(sample)
            .wrap_err("error writing a sample to a WAV file")?;
        samples_read += 1;
    }
    // the length of the preview should be 30 seconds, but sometimes it's a little under
    let remaining_samples = samples_to_read - samples_read;
    if remaining_samples > sample_rate {
        // if it's more than half a second under, error
        // (`remaining_samples` is doubled because we're counting individual values from stereo music)
        Err(eyre::eyre!(
            "could not read enough samples from track ({} < {})",
            samples_read,
            samples_to_read
        ))?;
    } else {
        // otherwise just fill in silence
        for _ in 0..remaining_samples {
            writer
                .write_sample(0)
                .wrap_err("error writing padding to a WAV file")?;
        }
    }
    writer.finalize()?;
    Ok(buf)
}

/// Get a clip from a track.
/// The clip is returned as a vector of bytes in WAV format.
pub async fn clip(track_id: u32, preview: &str, time: Range<chrono::Duration>) -> Result<Vec<u8>> {
    let path = ensure_cached(track_id, preview).await?;
    task::spawn_blocking(move || blocking_clip_track(path, time)).await?
}
