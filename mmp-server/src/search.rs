use std::path::Path;

use tantivy::{
    directory::MmapDirectory,
    schema::{Schema, STORED, TEXT},
    Index, IndexWriter,
};
use tracing::debug;

#[tracing::instrument]
pub fn init_tantivy(path: &Path) -> eyre::Result<(Schema, Index, IndexWriter)> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("song_id", TEXT | STORED);
    schema_builder.add_text_field("song_title", TEXT | STORED);
    schema_builder.add_text_field("song_artist", TEXT | STORED);
    schema_builder.add_text_field("song_album", TEXT | STORED);
    debug!("building schema");
    let schema = schema_builder.build();
    debug!("creating index in dir {:?}", path);
    std::fs::create_dir_all(path)?;
    let dir = MmapDirectory::open(path)?;

    let indxr = Index::open_or_create(dir, schema.clone())?;
    let idx_writer = indxr.writer(50_000_000)?;
    Ok((schema, indxr, idx_writer))
}
