use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use mmp_lib::routes::song::SearchSongQuery;
use mmp_lib::SongEntry;
use tantivy::query::QueryParser;
use tracing::debug;

use crate::state::ServerState;

async fn get_song_cover(
    Path(id): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<(HeaderMap, Vec<u8>), StatusCode> {
    let img_data = tokio::fs::read(
        state
            .args
            .data_dir
            .join("covers")
            .join(format!("{}.png", id)),
    )
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "image/png".parse().unwrap());
    Ok((headers, img_data))
}

async fn get_song_id(
    Path(id): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<SongEntry>, StatusCode> {
    let song = state
        .music_library
        .data
        .get(&id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let song = song.value().clone();
    Ok(Json(song))
}
async fn song_search(
    Query(query): Query<SearchSongQuery>,
    State(state): State<Arc<ServerState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut songs = Vec::new();
    let searcher = state.search.reader.searcher();
    let fields = state.search.fields.clone();
    let mut query_parser = QueryParser::for_index(
        &state.search.index,
        vec![fields.song_title, fields.song_artist, fields.song_album],
    );
    query_parser.set_field_fuzzy(fields.song_title, false, 2, false);
    debug!("parsing query: {:?}", query.query);
    let query = query_parser
        .parse_query(&query.query)
        .inspect_err(|e| tracing::error!("unable to parse query: {:?}", e))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // blocking - in actual route it needs to be spawned
    let top_10 = tokio::task::spawn_blocking(move || {
        searcher
            .search(&query, &tantivy::collector::TopDocs::with_limit(10))
            .inspect_err(|e| tracing::error!("unable to search query: {:?}", e))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;
    let searcher = state.search.reader.searcher();
    for (_score, doc_address) in top_10 {
        debug!("retrieving doc: {:?}", doc_address);
        let retrieved_doc = searcher
            .doc(doc_address)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let song_id = retrieved_doc
            .get_first(fields.song_id)
            .unwrap()
            .as_text()
            .unwrap();
        let song = state
            .music_library
            .data
            .get(song_id)
            .unwrap()
            .value()
            .clone();
        songs.push(song);
    }
    Ok(Json(songs))
}

pub fn make_song_router() -> Router<Arc<ServerState>> {
    Router::new()
        .route("/:id", get(get_song_id))
        .route("/cover/:id", get(get_song_cover))
        .route("/search", get(song_search))
}
