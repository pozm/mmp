use futures_lite::stream::StreamExt;
use std::{path::Path, sync::Arc};
use tracing::debug;

use async_walkdir::{Filtering, WalkDir};

use crate::songfile::song_from_path;
const VALID_MUSIC_EXTS: [&str; 7] = ["mp3", "flac", "wav", "ogg", "m4a", "wma", "aiff"];

#[tracing::instrument]
pub async fn check_files(folder: &Path, state: Arc<crate::state::ServerState>) {
    let mut entries = WalkDir::new(folder).filter(|e| async move {
        if let Some(true) = e.path().file_name().map(|f| {
            let path_name = f.to_string_lossy();
            VALID_MUSIC_EXTS.iter().any(|ext| path_name.ends_with(ext))
        }) {
            return Filtering::Continue;
        }
        Filtering::Ignore
    });
    loop {
        match entries.next().await {
            Some(Ok(entry)) => {
                let path = entry.path();
                // tokio::task::spawn(async {
                let state = Arc::clone(&state);
                tokio::task::spawn_blocking(move || {
                    let song = song_from_path(&path);
                    match song {
                        Ok(song) => {
                            state.music_library.data.insert(song.id.clone(), song);
                        }
                        Err(e) => {
                            tracing::error!("error: {}", e);
                        }
                    }
                });
                // });
            }
            Some(Err(e)) => {
                tracing::error!("error: {}", e);
                break;
            }
            None => break,
        }
    }
}
