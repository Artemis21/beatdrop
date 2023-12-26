//! A cache system for storing preview MP3s from Deezer, and retrieving clips from them.
use std::ops::Range;

use eyre::OptionExt;

use crate::{deezer::CLIENT, Error, ResultExt};

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

/// Transcode a downloaded track from MP3 to WAV, and save it (blocking).
fn blocking_save_track(path: std::path::PathBuf, mp3_data: Vec<u8>) -> Result<(), Error> {
    let (mp3_header, samples) =
        puremp3::read_mp3(std::io::Cursor::new(mp3_data)).wrap_err("error parsing a downloaded MP3")?;
    let wav_spec = hound::WavSpec {
        channels: 2,
        sample_rate: mp3_header.sample_rate.hz(),
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let file = std::fs::File::create(path).wrap_err("error creating a file for decoded music")?;
    let mut writer = hound::WavWriter::new(std::io::BufWriter::new(file), wav_spec)
        .wrap_err("error creating a WAV writer")?;
    for (left, right) in samples {
        writer
            .write_sample(left)
            .wrap_err("error writing a (left) sample to a WAV file")?;
        writer
            .write_sample(right)
            .wrap_err("error writing a (right) sample to a WAV file")?;
    }
    writer.finalize()?;
    Ok(())
}

/// Download a track from Deezer and save it to the music cache.
async fn download_track(config: &Config, track_id: u32, preview: &str) -> Result<(), Error> {
    let response = CLIENT
        .get(preview)
        .send()
        .await
        .wrap_err("error downloading a track preview")?;
    let data = response
        .bytes()
        .await
        .wrap_err("error reading the response for a track preview download")?
        .to_vec();
    let path = config.music_dir.join(format!("{}.mp3", track_id));
    tokio::task::spawn_blocking(move || blocking_save_track(path, data)).await?
}

/// Ensure that a given track is cached, and return the path.
async fn ensure_cached(
    config: &Config,
    track_id: u32,
    preview: &str,
) -> Result<std::path::PathBuf, Error> {
    let path = config.music_dir.join(format!("{}.mp3", track_id));
    if tokio::fs::try_exists(&path)
        .await
        .wrap_err("error checking if a track is cached")?
    {
        download_track(config, track_id, preview).await?;
    }
    Ok(path)
}

/// Get a clip from a track (blocking).
fn blocking_clip_track(path: std::path::PathBuf, seconds: Range<u32>) -> Result<Vec<u8>, Error> {
    let mut reader = hound::WavReader::open(path).wrap_err("error opening a cached track")?;
    let spec = reader.spec();
    reader
        .seek(spec.sample_rate * seconds.start)
        .wrap_err("error seeking within a cached track")?;
    let mut buf = Vec::new();
    let cursor = std::io::Cursor::new(&mut buf);
    let mut writer = hound::WavWriter::new(cursor, spec).wrap_err("error creating a WAV writer")?;
    for _ in 0..(spec.sample_rate * (seconds.end - seconds.start)) {
        let left = reader
            .samples::<f32>()
            .next()
            .ok_or_eyre("could not read sample from track")??;
        let right = reader
            .samples::<f32>()
            .next()
            .ok_or_eyre("could not read sample from track")??;
        writer
            .write_sample(left)
            .wrap_err("error writing a (left) sample to a WAV file")?;
        writer
            .write_sample(right)
            .wrap_err("error writing a (right) sample to a WAV file")?;
    }
    writer.finalize()?;
    Ok(buf)
}

/// Get a clip from a track.
/// The clip is returned as a vector of bytes in WAV format.
pub async fn clip(
    config: &Config,
    track_id: u32,
    preview: &str,
    seconds: Range<u32>,
) -> Result<Vec<u8>, Error> {
    let path = ensure_cached(config, track_id, preview).await?;
    tokio::task::spawn_blocking(move || blocking_clip_track(path, seconds)).await?
}
