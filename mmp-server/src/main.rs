pub mod cacher;
mod routes;
pub mod search;
pub mod songfile;
mod state;
mod tasks;
use std::sync::Arc;

use clap::Parser;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

#[tokio::main]
async fn main() {
    init_logging();
    let server_args = state::ServerArgs::parse();

    let server_state: Arc<state::ServerState> = Arc::new(state::ServerState::new(
        &server_args.data_dir.clone(),
        server_args,
    ));

    debug!("registering tasks");
    let awtasks = tokio::task::spawn(tasks::register_all(Arc::clone(&server_state)));
    let server_router = routes::make_router(Arc::clone(&server_state));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on port 3000");
    axum::serve(listener, server_router).await.unwrap();
    // send stop notification
    let _ = awtasks.await;
}
fn init_logging() {
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
}
