use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use mmp_lib::routes::test::SearchSongQuery;
use tantivy::{error, query::QueryParser};
use tracing::debug;

use crate::state::ServerState;

// #[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn test_song_search(
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
    let top_10 = searcher
        .search(&query, &tantivy::collector::TopDocs::with_limit(10))
        .inspect_err(|e| tracing::error!("unable to search query: {:?}", e))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
pub async fn test_library(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let mut songs = Vec::new();
    for (_, song) in state.music_library.data.clone().into_iter() {
        songs.push(song.clone());
    }
    Json(songs)
}
#[cfg(debug_assertions)]
pub fn make_test_router() -> Router<Arc<ServerState>> {
    Router::new()
        .route("/library", get(test_library))
        .route("/search", get(test_song_search))
}
#[cfg(not(debug_assertions))]
pub fn make_test_router() -> Router<Arc<ServerState>> {
    Router::new()
}
