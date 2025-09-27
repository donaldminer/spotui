use crate::event::{AppEvent, Event, EventHandler};
use open;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    widgets::ListState,
};
use reqwest::Url;
use rouille::{Response, Server};
use spotify_rs::{
    AuthCodePkceClient, AuthCodePkceFlow, RedirectUrl, Token,
    client::Client,
    model::{
        artist::Artist,
        player::CurrentlyPlayingItem,
        playlist::{PlaylistItem, SimplifiedPlaylist},
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

#[derive(Debug, Clone)]
pub struct Directory {
    pub title: String,
    pub list: Vec<ratatui::widgets::ListItem<'static>>,
    pub list_state: ListState,
}

#[derive(Debug, Clone)]
pub struct UserPlaylists {
    pub name: String,
    pub list: Vec<Option<SimplifiedPlaylist>>,
    pub list_state: ListState,
}

#[derive(Debug, Clone)]
pub struct UserTopTracks {
    pub name: String,
    pub list: Vec<Option<Track>>,
    pub list_state: ListState,
}

#[derive(Debug, Clone)]
pub struct UserTopArtists {
    pub name: String,
    pub list: Vec<Option<Artist>>,
    pub list_state: ListState,
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub name: String,
    pub list: Vec<Option<PlaylistItem>>,
    pub list_state: ListState,
}

#[derive(Debug, Clone)]
pub struct UserLibrary {
    pub selected_state: ListState,
    pub directory: Directory,
    pub user_playlists: UserPlaylists,
    pub user_top_tracks: UserTopTracks,
    pub user_top_artists: UserTopArtists,
    pub playlist: Playlist,
}

pub struct Player {
    pub currently_playing: Option<CurrentlyPlayingItem>,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ActiveBlock {
    Directory,
    UserPlaylists,
    UserTopTracks,
    UserTopArtists,
    Playlist,
}

#[derive(Debug)]
pub struct Route {
    pub active_block: ActiveBlock,
    pub hovered_block: ActiveBlock,
}

pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub spotify_client: Client<Token, AuthCodePkceFlow>,
    pub user_library: UserLibrary,
    pub user: Option<PrivateUser>,
    pub player: Player,
    pub route: Route,
}
impl App {
    pub async fn new() -> Self {
        log::info!("Initializing application...");
        let spotify_client = match Self::get_spotify_client().await {
            Ok(client) => {
                log::info!("Successfully obtained Spotify client.");
                client
            }
            Err(e) => panic!("Failed to get spotify client: {e}"),
        };
        Self {
            running: true,
            events: EventHandler::new(),
            spotify_client,
            user_library: UserLibrary {
                selected_state: ListState::default(),
                directory: Directory {
                    title: "Directory".to_string(),
                    list: vec![
                        ratatui::widgets::ListItem::new("Playlists"),
                        ratatui::widgets::ListItem::new("Top Tracks"),
                        ratatui::widgets::ListItem::new("Top Artists"),
                    ],
                    list_state: ListState::default(),
                },
                user_playlists: UserPlaylists {
                    name: "My Playlists".to_string(),
                    list: Vec::new(),
                    list_state: ListState::default(),
                },
                user_top_tracks: UserTopTracks {
                    name: "My Top Tracks".to_string(),
                    list: Vec::new(),
                    list_state: ListState::default(),
                },
                user_top_artists: UserTopArtists {
                    name: "My Top Artists".to_string(),
                    list: Vec::new(),
                    list_state: ListState::default(),
                },
                playlist: Playlist {
                    name: String::new(),
                    list: Vec::new(),
                    list_state: ListState::default(),
                },
            },
            user: None,
            player: Player {
                currently_playing: None,
            },
            route: Route {
                active_block: ActiveBlock::Directory,
                hovered_block: ActiveBlock::UserPlaylists,
            },
        }
    }

    fn start_server(redirect_url_host: String, tx: std::sync::mpsc::SyncSender<(String, String)>) {
        tokio::spawn(async move {
            log::info!("Starting local server at http://{}", redirect_url_host);
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

    pub async fn get_spotify_client() -> color_eyre::Result<AuthCodePkceClient<Token>> {
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

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.init().await?;
        terminal.clear()?;
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    ratatui::crossterm::event::Event::Key(key_event) => {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                    AppEvent::Select => self.select().await,
                    AppEvent::Next => self.next().await,
                },
            }
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Tab => self.events.send(AppEvent::Next),
            KeyCode::Up | KeyCode::Char('k') => self.up(),
            KeyCode::Down | KeyCode::Char('j') => self.down(),
            KeyCode::Left | KeyCode::Char('h') => {}
            KeyCode::Right | KeyCode::Char('l') => {}
            KeyCode::Enter => self.events.send(AppEvent::Select),
            _ => {}
        }
        Ok(())
    }

    pub async fn next(&mut self) {
        self.route.active_block = match self.route.active_block {
            ActiveBlock::Directory => self.route.hovered_block,
            _ => ActiveBlock::Directory,
        };
    }

    pub async fn select(&mut self) {
        match self.route.active_block {
            ActiveBlock::Directory => {
                match self.user_library.directory.list_state.selected() {
                    Some(0) => self.route.active_block = ActiveBlock::UserPlaylists,
                    Some(1) => self.route.active_block = ActiveBlock::UserTopTracks,
                    Some(2) => self.route.active_block = ActiveBlock::UserTopArtists,
                    _ => {}
                }
                self.route.hovered_block = self.route.active_block
            }
            ActiveBlock::UserPlaylists => {
                match self.user_library.user_playlists.list_state.selected() {
                    Some(i) => {
                        let playlist = spotify_rs::playlist(
                            self.user_library.user_playlists.list[i]
                                .as_ref()
                                .unwrap()
                                .clone()
                                .id,
                        )
                        .get(&self.spotify_client)
                        .await
                        .unwrap();

                        self.user_library.playlist.name = playlist.name.clone();
                        self.user_library.playlist.list = playlist.tracks.items;
                        self.user_library.playlist.list_state = ListState::default();
                    }
                    _ => {}
                }
                self.route.active_block = ActiveBlock::Playlist;
                self.route.hovered_block = ActiveBlock::Playlist;
            }
            ActiveBlock::UserTopTracks => match self.user_library.selected_state.selected() {
                _ => todo!(),
            },
            ActiveBlock::UserTopArtists => match self.user_library.selected_state.selected() {
                _ => todo!(),
            },
            ActiveBlock::Playlist => match self.user_library.selected_state.selected() {
                _ => todo!(),
            },
        };
    }

    pub fn up(&mut self) {
        match self.route.active_block {
            ActiveBlock::Directory => {
                self.user_library.directory.list_state.select_previous();
            }
            ActiveBlock::UserPlaylists => {
                self.user_library
                    .user_playlists
                    .list_state
                    .select_previous();
            }
            ActiveBlock::UserTopTracks => {
                self.user_library
                    .user_top_tracks
                    .list_state
                    .select_previous();
            }
            ActiveBlock::UserTopArtists => {
                self.user_library
                    .user_top_artists
                    .list_state
                    .select_previous();
            }
            ActiveBlock::Playlist => {
                self.user_library.playlist.list_state.select_previous();
            }
        }
    }

    pub fn down(&mut self) {
        match self.route.active_block {
            ActiveBlock::Directory => {
                self.user_library.directory.list_state.select_next();
            }
            ActiveBlock::UserPlaylists => {
                self.user_library.user_playlists.list_state.select_next();
            }
            ActiveBlock::UserTopTracks => {
                self.user_library.user_top_tracks.list_state.select_next();
            }
            ActiveBlock::UserTopArtists => {
                self.user_library.user_top_artists.list_state.select_next();
            }
            ActiveBlock::Playlist => {
                self.user_library.playlist.list_state.select_next();
            }
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub async fn init(&mut self) -> color_eyre::Result<()> {
        let spotify_client = &self.spotify_client;

        let get_user_playlists_future = async {
            spotify_rs::current_user_playlists()
                .get(spotify_client)
                .await
        };

        let get_user_profile_future =
            async { spotify_rs::get_current_user_profile(spotify_client).await };

        let get_user_top_tracks_future = async {
            spotify_rs::current_user_top_tracks()
                .get(spotify_client)
                .await
        };

        let get_user_top_artists_future = async {
            spotify_rs::current_user_top_artists()
                .get(spotify_client)
                .await
        };
        let (playlists_result, profile_result, top_tracks_result, top_artists_result) = tokio::join!(
            get_user_playlists_future,
            get_user_profile_future,
            get_user_top_tracks_future,
            get_user_top_artists_future,
        );

        self.user_library.user_playlists.list = playlists_result?.items;
        self.user = Some(profile_result?);
        self.user_library.user_top_tracks.list = top_tracks_result?.items;
        self.user_library.user_top_artists.list = top_artists_result?.items;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_get_spotify_client() {
        let client = App::get_spotify_client().await;
        assert!(client.is_ok());
    }
}
