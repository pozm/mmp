mod song;
mod stream;
mod test;

use std::sync::Arc;

use axum::Router;

use crate::state::ServerState;

use self::{song::make_song_router, stream::make_stream_router, test::make_test_router};

#[tracing::instrument]
pub fn make_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .nest("/songs", make_song_router())
        .nest("/stream", make_stream_router())
        .nest("/test", make_test_router())
        .with_state(state)
}
