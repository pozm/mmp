use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::Parser;
use dashmap::DashMap;
use mmp_lib::SongEntry;
use parking_lot::RwLock;
use sqlx::{sqlite::SqliteConnectOptions, Acquire, ConnectOptions};
use tantivy::schema::Schema;

use crate::{cacher::ReadyCache, data::dbinit, search};

#[derive(Debug)]
pub struct ServerState {
    pub music_library: MusicLibrary,
    pub search: SongSearch,
    pub args: ServerArgs,
    pub db_pool: sqlx::SqlitePool,
}
#[derive(Debug, Default)]
pub struct MusicLibrary {
    // cacher: ReadyCache<SongEntry>,
    pub data: DashMap<String, SongEntry>,
}
pub struct SearchFields {
    pub song_id: tantivy::schema::Field,
    pub song_title: tantivy::schema::Field,
    pub song_artist: tantivy::schema::Field,
    pub song_album: tantivy::schema::Field,
    pub song_metadata: tantivy::schema::Field,
}
pub struct SongSearch {
    pub fields: Arc<SearchFields>,
    pub schema: Schema,
    pub index: tantivy::Index,
    pub reader: tantivy::IndexReader,
    pub writer: RwLock<tantivy::IndexWriter>,
}
impl Debug for SongSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("song search")
    }
}
impl SongSearch {
    fn new(path: &Path) -> Self {
        let (schema, index, writer) = search::init_tantivy(path).unwrap();
        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommit)
            .try_into()
            .unwrap();

        SongSearch {
            fields: Arc::new(SearchFields::new(&schema)),
            schema,
            index,
            reader,
            writer: RwLock::new(writer),
        }
    }
}
impl ServerState {
    pub async fn new(data_dir: &Path, args: ServerArgs) -> Self {
        let sqlit = args.data_dir.join("mmp.db");
        let conopts = SqliteConnectOptions::new()
            .filename(sqlit)
            .create_if_missing(true);
        let db_pool = sqlx::SqlitePool::connect_with(conopts).await.unwrap();
        let pool_conn = db_pool.acquire().await.unwrap();
        dbinit::init_db(pool_conn).await.unwrap();
        Self {
            search: SongSearch::new(data_dir),
            music_library: MusicLibrary::default(),
            args,
            db_pool,
        }
    }
}
impl SearchFields {
    pub fn new(schema: &Schema) -> Self {
        let song_id = schema.get_field("song_id").unwrap();
        let song_title = schema.get_field("song_title").unwrap();
        let song_artist = schema.get_field("song_artist").unwrap();
        let song_album = schema.get_field("song_album").unwrap();
        let song_metadata = schema.get_field("song_metadata").unwrap();
        Self {
            song_id,
            song_title,
            song_artist,
            song_album,
            song_metadata,
        }
    }
}
#[derive(Parser, Debug)]
pub struct ServerArgs {
    pub music_folder: PathBuf,
    pub data_dir: PathBuf,
}
