use std::{ops::Bound, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::{headers::Range, TypedHeader};
use axum_range::{KnownSize, Ranged};
use ffmpeg_sidecar::command::FfmpegCommand;
use tokio::{fs::File, io::AsyncSeekExt};
use tracing::{debug, trace};

use crate::state::ServerState;

#[tracing::instrument(skip(state))]
async fn stream_song(
    Path(id): Path<String>,
    range: Option<TypedHeader<Range>>,
    State(state): State<Arc<ServerState>>,
) -> Result<Ranged<KnownSize<File>>, StatusCode> {
    let song = state
        .music_library
        .data
        .get(&id)
        .ok_or(StatusCode::NOT_FOUND)?;
    debug!("found song to stream: {:?}", song.value().title);
    let path = song.value().path.clone();
    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let body = KnownSize::file(file).await.unwrap();
    let range = range.map(|TypedHeader(range)| range);

    // let mut ffcmd = if let Some(range) = range {
    //     let size_of_file = tokio::fs::metadata(&path)
    //         .await
    //         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    //         .len();
    //     let ranges = range
    //         .0
    //         .satisfiable_ranges(size_of_file)
    //         .map(|x| {
    //             let start = if let Bound::Included(x) = x.0 { x } else { 0 };
    //             let end = if let Bound::Included(x) = x.1 { x } else { 0 };
    //             format!("between(t,{},{})", start, end)
    //         })
    //         .collect::<Vec<String>>();
    //
    //     FfmpegCommand::new()
    //         .input(path.to_string_lossy())
    //         .format("flac")
    //         // .codec_audio("pcm_s16le")
    //         // .format("s16le")
    //         .arg("-af")
    //         .arg(format!("aselect='{}',asetpts=N/SR/TB", ranges.join("+")))
    //         .no_video()
    //         .duration("20s")
    //         .output("-")
    //         .spawn()
    //         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    // } else {
    //     FfmpegCommand::new()
    //         .input(path.to_string_lossy())
    //         .codec_audio("copy")
    //         .format("flac")
    //         // .codec_audio("pcm_s16le")
    //         // .format("s16le")
    //         .no_video()
    //         .duration("20s")
    //         .output("-")
    //         .spawn()
    //         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    // };
    // debug!("spawned ffmpeg command");
    // let mut chunks = vec![];
    // for evnt in ffcmd
    //     .iter()
    //     .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    // {
    //     match evnt {
    //         ffmpeg_sidecar::event::FfmpegEvent::Log(lvl, cnt) => {
    //             trace!(level = ?lvl, content = cnt)
    //         }
    //         ffmpeg_sidecar::event::FfmpegEvent::LogEOF => trace!("end of file ffmpeg"),
    //         ffmpeg_sidecar::event::FfmpegEvent::Error(e) => tracing::error!("ffmpeg error : {e}"),
    //         ffmpeg_sidecar::event::FfmpegEvent::Progress(p) => {
    //             trace!("ffmpeg speed: {}", p.speed)
    //         }
    //         ffmpeg_sidecar::event::FfmpegEvent::OutputChunk(mut chunk) => {
    //             trace!("got ffmpeg chunk!");
    //             chunks.append(&mut chunk)
    //         }
    //         ffmpeg_sidecar::event::FfmpegEvent::Done => trace!("ffmpeg done"),
    //         _ => {}
    //     };
    // }
    debug!("done streaming song");
    Ok(Ranged::new(range, body))
}

pub fn make_stream_router() -> Router<Arc<ServerState>> {
    Router::new().route("/song/:id", get(stream_song))
}
