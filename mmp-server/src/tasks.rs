use tokio::join;
use tracing::{debug, instrument};

use crate::ServerArgs;

use self::files::check_files;

mod files;

#[instrument(skip(args))]
pub async fn register_all(args: ServerArgs) {
    let check_files_future = check_files(&args.music_folder);
    join!(check_files_future);
}
