use mmp_lib::SongEntry;

use crate::cacher::ReadyCache;

struct ServerState {
    music_library: MusicLibrary,
}
struct MusicLibrary {
    cacher: ReadyCache<SongEntry>,
}
