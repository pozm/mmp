use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use mmp_lib::SongEntry;

use crate::state::ServerState;

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
pub fn make_song_router() -> Router<Arc<ServerState>> {
    Router::new().route("/:id", get(get_song_id))
}
