#[cfg(debug_assertions)]
pub fn install_ffmpeg() -> eyre::Result<()> {
    ffmpeg_sidecar::download::auto_download()
        .inspect_err(|err| tracing::error!("error downloading ffmpeg : {err}"))
        .map_err(|err| eyre::eyre!(Box::new(err)))
}
#[cfg(not(debug_assertions))]
pub fn install_ffmpeg() -> eyre::Result<()> {}
