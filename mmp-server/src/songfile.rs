use std::{
    hash::Hasher,
    path::{Path, PathBuf},
    sync::Arc,
};

use eyre::{ContextCompat, Result};
use lofty::{Accessor, AudioFile, FileProperties, Picture, Probe, Tag, TaggedFile, TaggedFileExt};
use mmp_lib::SongEntry;
use tracing::debug;

use crate::state::ServerState;
#[tracing::instrument(skip(tag, fallback))]
fn get_id_from_tag(tag: &Tag, fallback: &[u8]) -> Result<String> {
    Ok(tag
        .get_string(&lofty::ItemKey::MusicBrainzTrackId)
        .map(|x| x.to_string())
        .unwrap_or_else(|| {
            let mut hasher = rustc_hash::FxHasher::default();
            hasher.write(fallback);
            format!("mmp.{}", hasher.finish())
        }))
}
#[tracing::instrument(skip(first_pic))]
pub async fn register_cover(data_dir: PathBuf, id: String, first_pic: Picture) {
    let dir = data_dir.join("covers");
    tokio::fs::create_dir_all(&dir).await.unwrap_or_else(|e| {
        tracing::error!("unable to create cover dir: {}", e);
    });
    let cover_path = dir.join(format!("{}.png", &id));
    let ignore = tokio::fs::try_exists(&cover_path).await;
    if let Ok(true) = ignore {
        debug!("cover already exists for {}", id);
        return;
    }
    // let pictype = first_pic.mime_type();
    let data = first_pic.into_data();
    tokio::fs::write(cover_path, data)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("unable to write cover for {}: {}", id, e);
        });
}
#[tracing::instrument(skip(tag, props, path))]
pub fn create_song_entry(
    tag: &Tag,
    props: &FileProperties,
    path: PathBuf,
    id: String,
) -> Result<SongEntry> {
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

#[tracing::instrument(skip(state))]
pub fn song_from_path(
    state: Arc<ServerState>,
    path: &Path,
    should_register_cover: Option<bool>,
) -> Result<SongEntry> {
    let tagged = Probe::open(path)?.read()?;
    let tag = tagged
        .primary_tag()
        .wrap_err("unable to read primary tag")?;
    let props = tagged.properties();
    let id = get_id_from_tag(tag, path.as_os_str().as_encoded_bytes())?;
    let pictures = tag.pictures().first().cloned();
    if should_register_cover.unwrap_or(false) {
        if let Some(first_pic) = pictures {
            tokio::task::spawn(register_cover(
                state.args.data_dir.clone(),
                id.clone(),
                first_pic,
            ));
        }
    }
    create_song_entry(tag, props, path.to_path_buf(), id)
}
