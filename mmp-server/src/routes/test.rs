use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};

use crate::state::ServerState;

pub async fn test_library(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let mut songs = Vec::new();
    for (_, song) in state.music_library.data.clone().into_iter() {
        songs.push(song.clone());
    }
    Json(songs)
}
#[cfg(debug_assertions)]
pub fn make_test_router() -> Router<Arc<ServerState>> {
    Router::new().route("/library", get(test_library))
}
#[cfg(not(debug_assertions))]
pub fn make_test_router() -> Router<Arc<ServerState>> {
    Router::new()
}
