//! A cache system for storing preview MP3s from Deezer, and retrieving clips from them.
use std::ops::Range;

use crate::{deezer::CLIENT, Error};

/// Config options for the music cache system.
pub struct Config {
    /// The directory where music files are stored.
    pub music_dir: std::path::PathBuf,
}

impl From<&crate::Config> for Config {
    fn from(config: &crate::Config) -> Self {
        let music_dir = config.upload_dir.join("music");
        std::fs::create_dir_all(&music_dir).expect("failed to create music directory");
        Self { music_dir }
    }
}

impl From<hound::Error> for Error {
    fn from(err: hound::Error) -> Self {
        match err {
            hound::Error::IoError(err) => Self::Io(err),
            _ => Self::Internal("could not encode downloaded MP3 as WAV"),
        }
    }
}

/// Transcode a downloaded track from MP3 to WAV, and save it (blocking).
fn blocking_save_track(path: std::path::PathBuf, mp3_data: Vec<u8>) -> Result<(), Error> {
    let (mp3_header, samples) = puremp3::read_mp3(std::io::Cursor::new(mp3_data))
        .map_err(|_| Error::Internal("could not decode MP3 from Deezer"))?;
    let wav_spec = hound::WavSpec {
        channels: 2,
        sample_rate: mp3_header.sample_rate.hz(),
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let file = std::fs::File::create(path)?;
    let mut writer = hound::WavWriter::new(std::io::BufWriter::new(file), wav_spec)?;
    for (left, right) in samples {
        writer.write_sample(left)?;
        writer.write_sample(right)?;
    }
    writer.finalize()?;
    Ok(())
}

/// Download a track from Deezer and save it to the music cache.
async fn download_track(config: &Config, track_id: i32, preview: &str) -> Result<(), Error> {
    let response = CLIENT.get(preview).send().await?;
    let data = response.bytes().await?.to_vec();
    let path = config.music_dir.join(format!("{}.mp3", track_id));
    tokio::task::spawn_blocking(move || blocking_save_track(path, data))
        .await
        .map_err(|_| Error::Internal("transcoding and caching MP3 failed"))?
}

/// Ensure that a given track is cached, and return the path.
async fn ensure_cached(
    config: &Config,
    track_id: i32,
    preview: &str,
) -> Result<std::path::PathBuf, Error> {
    let path = config.music_dir.join(format!("{}.mp3", track_id));
    if tokio::fs::try_exists(&path).await? {
        download_track(config, track_id, preview).await?;
    }
    Ok(path)
}

/// Get a clip from a track (blocking).
fn blocking_clip_track(path: std::path::PathBuf, seconds: Range<u32>) -> Result<Vec<u8>, Error> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    reader.seek(spec.sample_rate * seconds.start)?;
    let mut buf = Vec::new();
    let cursor = std::io::Cursor::new(&mut buf);
    let mut writer = hound::WavWriter::new(cursor, spec)?;
    for _ in 0..(spec.sample_rate * (seconds.end - seconds.start)) {
        let left = reader
            .samples::<f32>()
            .next()
            .ok_or(Error::Internal("could not read sample from track"))??;
        let right = reader
            .samples::<f32>()
            .next()
            .ok_or(Error::Internal("could not read sample from track"))??;
        writer.write_sample(left)?;
        writer.write_sample(right)?;
    }
    writer.finalize()?;
    Ok(buf)
}

/// Get a clip from a track.
/// The clip is returned as a vector of bytes in WAV format.
pub async fn clip(
    config: &Config,
    track_id: i32,
    preview: &str,
    seconds: Range<u32>,
) -> Result<Vec<u8>, Error> {
    let path = ensure_cached(config, track_id, preview).await?;
    tokio::task::spawn_blocking(move || blocking_clip_track(path, seconds))
        .await
        .map_err(|_| Error::Internal("reading and clipping track failed"))?
}
