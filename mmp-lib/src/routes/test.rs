#[derive(serde::Deserialize, Debug)]
pub struct SearchSongQuery {
    pub query: String,
}
