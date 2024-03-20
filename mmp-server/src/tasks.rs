use std::sync::Arc;

use tokio::join;
use tracing::{debug, instrument};

use self::files::check_files;

mod files;

#[instrument(skip(state))]
pub async fn register_all(state: Arc<crate::state::ServerState>) {
    let check_files_future = check_files(state);
    join!(check_files_future);
}
