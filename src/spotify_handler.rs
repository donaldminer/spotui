// Description: A handler for Spotify-related functionality.
// any API calls or interactions with Spotify would be managed here.
use reqwest::Url;
use rouille::{Response, Server};
use spotify_rs::{
    AuthCodePkceClient, AuthCodePkceFlow, RedirectUrl, Token,
    client::Client,
    model::{
        Page,
        artist::Artist,
        playlist::{Playlist, SimplifiedPlaylist},
        track::Track,
        user::PrivateUser,
    },
};
use std::sync::{Arc, Mutex};

const SCOPES: [&str; 9] = [
    "user-top-read",
    "user-follow-read",
    "user-read-private",
    "user-read-email",
    "user-read-playback-state",
    "user-read-currently-playing",
    "user-modify-playback-state",
    "playlist-read-private",
    "playlist-read-collaborative",
];

pub struct SpotifyHandler {
    client: Client<Token, AuthCodePkceFlow>,
}
impl SpotifyHandler {
    pub async fn new() -> Self {
        let spotify_client = match Self::get_spotify_client().await {
            Ok(client) => {
                log::info!("Successfully obtained Spotify client.");
                client
            }
            Err(e) => {
                log::error!("Failed to obtain Spotify client: {e}");
                panic!("Failed to get spotify client: {e}");
            }
        };
        Self {
            client: spotify_client,
        }
    }

    pub async fn get_top_tracks(&self) -> color_eyre::Result<Page<Track>> {
        let top_tracks = spotify_rs::current_user_top_tracks()
            .get(&self.client)
            .await?;
        Ok(top_tracks)
    }

    pub async fn get_top_artists(&self) -> color_eyre::Result<Page<Artist>> {
        let top_artists = spotify_rs::current_user_top_artists()
            .get(&self.client)
            .await?;
        Ok(top_artists)
    }

    pub async fn set_user(&self) -> color_eyre::Result<PrivateUser> {
        let user = spotify_rs::get_current_user_profile(&self.client).await?;
        Ok(user)
    }

    pub async fn get_user_playlists(&self) -> color_eyre::Result<Page<SimplifiedPlaylist>> {
        let playlists = spotify_rs::current_user_playlists()
            .get(&self.client)
            .await?;
        Ok(playlists)
    }

    pub async fn get_user_top_tracks(&self) -> color_eyre::Result<Page<Track>> {
        let top_tracks = spotify_rs::current_user_top_tracks()
            .get(&self.client)
            .await?;
        Ok(top_tracks)
    }

    pub async fn get_playlist(
        &self,
        selected_playlist: Option<&SimplifiedPlaylist>,
    ) -> color_eyre::Result<Playlist> {
        let playlist_id = selected_playlist.unwrap().id.clone();
        let playlist = spotify_rs::playlist(playlist_id).get(&self.client).await?;
        Ok(playlist)
    }

    async fn get_spotify_client() -> color_eyre::Result<AuthCodePkceClient<Token>> {
        dotenvy::dotenv().ok();
        let client_id = dotenvy::var("SPOTIFY_CLIENT_ID")?;
        let redirect_uri = dotenvy::var("SPOTIFY_REDIRECT_URI")?;
        let redirect_url = Url::parse(&redirect_uri).unwrap();
        let redirect_url_host = format!(
            "{}:{}",
            redirect_url.host_str().unwrap(),
            redirect_url.port().unwrap()
        );

        let (tx, rx) = std::sync::mpsc::sync_channel(1);

        Self::start_server(redirect_url_host, tx);

        let auto_refresh = false;

        let (client, url) = AuthCodePkceClient::new(
            client_id,
            SCOPES,
            RedirectUrl::new(redirect_uri)?,
            auto_refresh,
        );
        log::info!("Opening browser for Spotify authentication...");
        open::that(url.as_str())?;

        let (auth_code, csrf_state) = rx.recv().unwrap();

        let spotify_auth = client.authenticate(auth_code, csrf_state).await?;

        Ok(spotify_auth)
    }

    fn start_server(redirect_url_host: String, tx: std::sync::mpsc::SyncSender<(String, String)>) {
        tokio::spawn(async move {
            let sent = Arc::new(Mutex::new(false));
            let sent2 = sent.clone();

            let server = Server::new(redirect_url_host, move |request| {
                let url = Url::parse(&format!("http://{}", request.raw_url())).unwrap();
                let mut queries: Vec<_> = url.query_pairs().into_owned().collect();
                let auth_code = queries.remove(0).1;
                let csrf_state = queries.remove(0).1;

                tx.send((auth_code, csrf_state)).unwrap();

                *sent2.lock().unwrap() = true;

                Response::html("<h1>You may close this page</h1><script>window.close()</script>")
            })
            .unwrap();

            while !*sent.lock().unwrap() {
                server.poll();
            }
        });
    }
}
