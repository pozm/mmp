use dashmap::DashMap;
use mmp_lib::SongEntry;

use crate::cacher::ReadyCache;

#[derive(Debug, Default)]
pub struct ServerState {
    pub music_library: MusicLibrary,
}
#[derive(Debug, Default)]
pub struct MusicLibrary {
    // cacher: ReadyCache<SongEntry>,
    pub data: DashMap<String, SongEntry>,
}
