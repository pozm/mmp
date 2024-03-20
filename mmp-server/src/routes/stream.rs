use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use ffmpeg_sidecar::command::FfmpegCommand;
use tracing::{debug, trace};

use crate::state::ServerState;
#[tracing::instrument(skip(state))]
async fn start_stream(
    Path(id): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<Vec<u8>, StatusCode> {
    let song = state
        .music_library
        .data
        .get(&id)
        .ok_or(StatusCode::NOT_FOUND)?;
    debug!("found song to stream: {:?}", song.value().title);
    let path = song.value().path.clone();
    let mut ffcmd = FfmpegCommand::new()
        .input(path.to_string_lossy())
        .codec_audio("copy")
        .format("flac")
        // .codec_audio("pcm_s16le")
        // .format("s16le")
        .no_video()
        .duration("20s")
        .output("-")
        .spawn()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    debug!("spawned ffmpeg command");
    let mut chunks = vec![];
    for evnt in ffcmd
        .iter()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        match evnt {
            ffmpeg_sidecar::event::FfmpegEvent::Log(lvl, cnt) => {
                trace!(level = ?lvl, content = cnt)
            }
            ffmpeg_sidecar::event::FfmpegEvent::LogEOF => trace!("end of file ffmpeg"),
            ffmpeg_sidecar::event::FfmpegEvent::Error(e) => tracing::error!("ffmpeg error : {e}"),
            ffmpeg_sidecar::event::FfmpegEvent::Progress(p) => {
                trace!("ffmpeg speed: {}", p.speed)
            }
            ffmpeg_sidecar::event::FfmpegEvent::OutputChunk(mut chunk) => {
                trace!("got ffmpeg chunk!");
                chunks.append(&mut chunk)
            }
            ffmpeg_sidecar::event::FfmpegEvent::Done => trace!("ffmpeg done"),
            _ => {}
        };
    }
    debug!("done streaming song");
    Ok(chunks)
}

pub fn make_stream_router() -> Router<Arc<ServerState>> {
    Router::new().route("/song/:id", get(start_stream))
}
