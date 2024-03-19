use std::sync::Arc;

use tokio::join;
use tracing::{debug, instrument};

use crate::ServerArgs;

use self::files::check_files;

mod files;

#[instrument(skip(args))]
pub async fn register_all(args: ServerArgs, state: Arc<crate::state::ServerState>) {
    let check_files_future = check_files(&args.music_folder, state);
    join!(check_files_future);
}
