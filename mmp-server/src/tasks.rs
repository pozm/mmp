use std::sync::Arc;

use tokio::join;
use tracing::{debug, instrument};

use crate::tasks::ffmpeg::install_ffmpeg;

use self::files::check_files;

mod ffmpeg;
mod files;

#[instrument(skip(state))]
pub async fn register_all(state: Arc<crate::state::ServerState>) {
    let check_files_future = check_files(state);
    // we're currently not using ffmpeg
    // let download_ffmpeg = tokio::task::spawn_blocking(install_ffmpeg);
    let _ = join!(check_files_future);
}
