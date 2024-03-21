use futures::{future::join_all, stream::StreamExt};
use std::{path::Path, sync::Arc};
use tantivy::{collector::Count, query::QueryParser, Document};
use tracing::{debug, error};

use async_walkdir::{Filtering, WalkDir};

use crate::{search::register_song_index, songfile::song_from_path};
const VALID_MUSIC_EXTS: [&str; 7] = ["mp3", "flac", "wav", "ogg", "m4a", "wma", "aiff"];

#[tracing::instrument]
pub async fn check_files(state: Arc<crate::state::ServerState>) {
    let folder = &state.args.music_folder;
    let mut entries = WalkDir::new(folder).filter(|e| async move {
        if let Some(true) = e.path().file_name().map(|f| {
            let path_name = f.to_string_lossy();
            VALID_MUSIC_EXTS.iter().any(|ext| path_name.ends_with(ext))
        }) {
            return Filtering::Continue;
        }
        Filtering::Ignore
    });
    let mut tasks = vec![];
    loop {
        match entries.next().await {
            Some(Ok(entry)) => {
                let path = entry.path();
                let state = Arc::clone(&state);
                tasks.push(tokio::task::spawn_blocking(move || {
                    // spawn blocking because the function will read files without tokio
                    //         VVVVVVVVVVVVVV     <------^
                    let song = song_from_path(Arc::clone(&state), &path, Some(true));
                    // 🙏 no inlay hints
                    match song {
                        Ok(song) => {
                            let searcher = state.search.reader.searcher();
                            register_song_index(
                                &song,
                                &state.search.fields,
                                &state.search.index,
                                &searcher,
                                &state.search.writer.read(),
                            );
                            state.music_library.data.insert(song.id.clone(), song);
                        }
                        Err(e) => {
                            tracing::error!("error: {}", e);
                        }
                    }
                }));
            }
            Some(Err(e)) => {
                tracing::error!("error: {}", e);
                break;
            }
            None => break,
        }
    }
    tokio::task::spawn(async move {
        debug!("waiting for song indexing tasks to complete");
        join_all(tasks).await;
        debug!("committing song indexer");

        let _ = state
            .search
            .writer
            .write()
            .commit()
            .inspect_err(|e| error!("unable to commit song indexer: {e}"));
    });
}
