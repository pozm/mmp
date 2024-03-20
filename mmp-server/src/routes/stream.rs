use std::{ops::Bound, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::{headers::Range, TypedHeader};
use axum_range::{KnownSize, Ranged};
use tokio::fs::File;
use tracing::debug;

use crate::state::ServerState;

#[tracing::instrument(skip(state))]
async fn stream_song(
    Path(id): Path<String>,
    range: Option<TypedHeader<Range>>,
    State(state): State<Arc<ServerState>>,
) -> Result<Ranged<KnownSize<File>>, StatusCode> {
    let song = state
        .music_library
        .data
        .get(&id)
        .ok_or(StatusCode::NOT_FOUND)?;
    debug!("found song to stream: {:?}", song.value().title);
    let path = song.value().path.clone();
    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let body = KnownSize::file(file).await.unwrap();
    let range = range
        .map(|TypedHeader(range)| range)
        .or_else(|| Range::bytes(0..400000).ok());
    debug!("done streaming song");
    Ok(Ranged::new(range, body))
}

pub fn make_stream_router() -> Router<Arc<ServerState>> {
    Router::new().route("/song/:id", get(stream_song))
}
