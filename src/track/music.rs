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
        let music_dir = config.media_dir.join("music");
        std::fs::create_dir_all(&music_dir).expect("failed to create music directory");
        Self { music_dir }
    }
}

/// Transcode a downloaded track from MP3 to WAV, and save it (blocking).
fn blocking_save_track(path: std::path::PathBuf, mp3_data: &[u8]) -> Result<(), Error> {
    let mut decoder = minimp3::Decoder::new(std::io::Cursor::new(mp3_data));
    let first_frame = decoder.next_frame().wrap_err("error reading first frame of MP3")?;
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
    while let Ok(frame) = maybe_frame {
        for sample in frame.data {
            writer
                .write_sample(sample)
                .wrap_err("error writing a sample to a WAV file")?;
        }
        maybe_frame = decoder.next_frame();
    }
    match maybe_frame.expect_err("we looped until it was an error") {
        minimp3::Error::Eof => (),
        err => Err(err).wrap_err("error decoding an MP3")?,
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
        .wrap_err("error reading the response for a track preview download")?;
    let path = config.music_dir.join(format!("{}.mp3", track_id));
    tokio::task::spawn_blocking(move || blocking_save_track(path, &data)).await?
}

/// Ensure that a given track is cached, and return the path.
async fn ensure_cached(
    config: &Config,
    track_id: u32,
    preview: &str,
) -> Result<std::path::PathBuf, Error> {
    let path = config.music_dir.join(format!("{}.mp3", track_id));
    if !tokio::fs::try_exists(&path)
        .await
        .wrap_err("error checking if a track is cached")?
    {
        download_track(config, track_id, preview).await?;
    }
    Ok(path)
}

/// Get a clip from a track (blocking).
fn blocking_clip_track(path: std::path::PathBuf, time: Range<chrono::Duration>) -> Result<Vec<u8>, Error> {
    let start = u32::try_from(time.start.num_milliseconds()).expect("start time to be positive and not overflow");
    let length = u32::try_from((time.end - time.start).num_milliseconds()).expect("clip length to be positive and not overflow");
    let mut reader = hound::WavReader::open(path).wrap_err("error opening a cached track")?;
    let spec = reader.spec();
    reader
        .seek(spec.sample_rate * start / 1000)
        .wrap_err("error seeking within a cached track")?;
    let mut buf = Vec::new();
    let cursor = std::io::Cursor::new(&mut buf);
    let mut writer = hound::WavWriter::new(cursor, spec).wrap_err("error creating a WAV writer")?;
    for _ in 0..(spec.sample_rate * length / 1000) {
        let left = reader
            .samples::<i16>()
            .next()
            .ok_or_eyre("could not read sample from track")??;
        let right = reader
            .samples::<i16>()
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
    time: Range<chrono::Duration>,
) -> Result<Vec<u8>, Error> {
    let path = ensure_cached(config, track_id, preview).await?;
    tokio::task::spawn_blocking(move || blocking_clip_track(path, time)).await?
}
