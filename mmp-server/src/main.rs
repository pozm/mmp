pub mod cacher;
pub mod songfile;
mod state;
mod tasks;
use std::path::PathBuf;

use axum::Router;
use clap::Parser;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

#[derive(Parser)]
struct ServerArgs {
    music_folder: PathBuf,
}
#[tokio::main]
async fn main() {
    let layer = HierarchicalLayer::default()
        .with_writer(std::io::stdout)
        .with_indent_lines(true)
        .with_indent_amount(2)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_verbose_exit(true)
        .with_verbose_entry(true)
        .with_targets(true);

    let env_filter = EnvFilter::from_default_env();
    let subscriber = Registry::default().with(layer).with(env_filter);
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let server_args = ServerArgs::parse();

    debug!("registering tasks");
    let awtasks = tokio::task::spawn(tasks::register_all(server_args));
    let app = Router::new();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on port 3000");
    axum::serve(listener, app).await.unwrap();
    // send stop notification
    let _ = awtasks.await;
}
