use std::path::Path;

use mmp_lib::SongEntry;
use tantivy::{
    collector::Count,
    directory::MmapDirectory,
    query::QueryParser,
    schema::{Schema, STORED, TEXT},
    Document, Index, IndexWriter, Searcher,
};
use tracing::{debug, error};

use crate::state::SearchFields;

#[tracing::instrument]
pub fn init_tantivy(path: &Path) -> eyre::Result<(Schema, Index, IndexWriter)> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("song_id", TEXT | STORED);
    schema_builder.add_text_field("song_title", TEXT);
    schema_builder.add_text_field("song_artist", TEXT);
    schema_builder.add_text_field("song_album", TEXT);
    schema_builder.add_json_field("song_metadata", STORED);
    debug!("building schema");
    let schema = schema_builder.build();
    debug!("creating index in dir {:?}", path);
    std::fs::create_dir_all(path)?;
    let dir = MmapDirectory::open(path)?;

    let indxr = Index::open_or_create(dir, schema.clone())?;
    let idx_writer = indxr.writer(50_000_000)?;
    Ok((schema, indxr, idx_writer))
}
#[tracing::instrument(skip_all, fields(song_title = song.title))]
pub fn register_song_index(
    song: &SongEntry,
    fields: &SearchFields,
    indexer: &tantivy::Index,
    searcher: &Searcher,
    writer: &IndexWriter,
) {
    let query_parser = QueryParser::for_index(indexer, vec![fields.song_id]);
    let query = query_parser.parse_query(&song.id).unwrap();
    let count = searcher.search(&query, &Count);
    if let Ok(0) = count {
        debug!("adding {} to song index", &song.id);
        let mut doc = Document::default();
        doc.add_text(fields.song_id, &song.id);
        doc.add_text(fields.song_title, &song.title);
        doc.add_text(fields.song_artist, &song.artist);
        doc.add_text(fields.song_album, &song.album);
        let jsn = serde_json::to_value(&song.metadata);
        if let Ok(serde_json::Value::Object(jsn_map)) = jsn {
            doc.add_json_object(fields.song_metadata, jsn_map);
        };
        let _ = writer
            .add_document(doc)
            .inspect(|_id| debug!("added {} to song indexer", &song.id))
            .inspect_err(|e| error!("unable to add {} to song indexer: {e}", &song.id));
    } else {
        debug!("{} already indexed", &song.id);
    }
}
