mod test;

use std::sync::Arc;

use axum::Router;

use crate::state::ServerState;

use self::test::make_test_router;

pub fn make_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .nest("/test", make_test_router())
        .with_state(state)
}
