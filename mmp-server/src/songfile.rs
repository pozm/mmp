use std::{hash::Hasher, path::Path};

use eyre::{ContextCompat, Ok, Result};
use lofty::{Accessor, AudioFile, Probe, TaggedFileExt};
use mmp_lib::SongEntry;

#[tracing::instrument]
pub fn song_from_path(path: &Path) -> Result<SongEntry> {
    let tagged = Probe::open(path)?.read()?;
    let tag = tagged
        .primary_tag()
        .wrap_err("unable to read primary tag")?;
    let props = tagged.properties();
    let id = tag
        .get_string(&lofty::ItemKey::MusicBrainzTrackId)
        .map(|x| x.to_string())
        .unwrap_or_else(|| {
            let mut hasher = rustc_hash::FxHasher::default();
            hasher.write(path.as_os_str().as_encoded_bytes());
            format!("mmp.{}", hasher.finish())
        });
    Ok(SongEntry {
        title: tag.title().wrap_err("unable to read title")?.to_string(),
        artist: tag.artist().wrap_err("unable to read artist")?.to_string(),
        album: tag.album().wrap_err("unable to read album")?.to_string(),
        duration: props.duration().as_secs(),
        sample_rate: props.sample_rate().unwrap_or_default(),
        bit_rate: props.overall_bitrate().unwrap_or_default(),
        bit_depth: props.bit_depth().unwrap_or_default() as u16,
        channels: props.channels().unwrap_or_default() as u16,
        year: tag.year().unwrap_or_default() as u16,
        id,
        path: path.to_path_buf(),
        metadata: tag
            .items()
            .map(|ti| {
                (
                    ti.key()
                        .map_key(tag.tag_type(), true)
                        .map(|x| x.to_string()),
                    ti.value().text().map(|x| x.to_string()),
                )
            })
            .filter_map(|xe| match xe {
                (Some(key), Some(value)) => Some((key, value)),
                _ => None,
            })
            .collect(),
    })
}
