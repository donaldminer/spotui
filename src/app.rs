use crate::event::{AppEvent, Event, EventHandler};
use open;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    widgets::{ListItem, ListState},
};
use reqwest::Url;
use rouille::{Response, Server};
use spotify_rs::{
    AuthCodePkceClient, AuthCodePkceFlow, RedirectUrl, Token,
    client::Client,
    model::{
        Page,
        artist::Artist,
        playlist::{Playlist, PlaylistItem, SimplifiedPlaylist},
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

pub const DIRECTORY: [&str; 3] = ["Playlists", "Top Tracks", "Top Artists"];
pub const TRACK_OPTIONS: [&str; 4] = ["Play", "Add to Queue", "Go to Artist", "Go to Album"];

#[derive(Debug, Clone)]
pub struct NavList {
    pub title: String,
    pub list: Vec<ratatui::widgets::ListItem<'static>>,
    pub list_state: ListState,
}

#[derive(Debug, Clone)]
pub struct PageEndpoint<T: Clone> {
    pub total: usize,
    pub page: Option<Page<T>>,
    pub list: Vec<Option<T>>,
    pub list_state: ListState,
}

impl<T> PageEndpoint<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self {
            total: 0,
            page: None,
            list: Vec::new(),
            list_state: ListState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrackList<T, U: Clone> {
    pub result: Option<T>,
    pub pages: PageEndpoint<U>,
    pub list_state: ListState,
}

impl<T, U> TrackList<T, U>
where
    U: Clone,
{
    pub fn new() -> Self {
        Self {
            result: None,
            pages: PageEndpoint::new(),
            list_state: ListState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserLibrary {
    pub user_playlists: PageEndpoint<SimplifiedPlaylist>,
    pub user_top_tracks: PageEndpoint<Track>,
    pub user_top_artists: PageEndpoint<Artist>,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ActiveBlock {
    Directory,
    UserPlaylists,
    UserTopTracks,
    UserTopArtists,
    Playlist,
    Artist,
    Popup,
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
    pub directory: NavList,
    pub selected_state: ListState,
    pub user: Option<PrivateUser>,
    pub route: Route,
    pub playlist: TrackList<Playlist, PlaylistItem>,
    pub track_popup: NavList,
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
            selected_state: ListState::default(),
            directory: NavList {
                title: "Directory".to_string(),
                list: DIRECTORY.iter().map(|&item| ListItem::new(item)).collect(),
                list_state: ListState::default(),
            },
            user_library: UserLibrary {
                user_playlists: PageEndpoint::new(),
                user_top_tracks: PageEndpoint::new(),
                user_top_artists: PageEndpoint::new(),
            },
            user: None,
            route: Route {
                active_block: ActiveBlock::Directory,
                hovered_block: ActiveBlock::UserPlaylists,
            },
            playlist: TrackList::new(),
            track_popup: NavList {
                title: "Options".to_string(),
                list: TRACK_OPTIONS
                    .iter()
                    .map(|&item| ListItem::new(item))
                    .collect(),
                list_state: ListState::default(),
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
        terminal.clear()?;
        self.events.send(AppEvent::Init);
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
                    AppEvent::Init => self.init().await?,
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
            ActiveBlock::Directory | ActiveBlock::Popup => self.route.hovered_block,
            _ => ActiveBlock::Directory,
        };
    }

    pub async fn select(&mut self) {
        match self.route.active_block {
            ActiveBlock::Directory => {
                match self.directory.list_state.selected() {
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

                        self.playlist.result = Some(playlist.clone());
                        self.playlist.pages.list = playlist.tracks.items;
                        self.playlist.pages.total = usize::try_from(playlist.tracks.total).unwrap();
                        self.playlist.list_state = ListState::default();
                    }
                    _ => {}
                }
                self.route.active_block = ActiveBlock::Playlist;
                self.route.hovered_block = ActiveBlock::Playlist;
            }
            //TODO: Implement Track Selection
            ActiveBlock::UserTopTracks => {
                match self.user_library.user_top_tracks.list_state.selected() {
                    Some(i) => {
                        self.track_popup.list_state.select(Some(0));
                        self.route.active_block = ActiveBlock::Popup;
                        log::info!(
                            "Track selected: {}",
                            self.user_library.user_top_tracks.list[i]
                                .as_ref()
                                .unwrap()
                                .name
                        );
                    }
                    _ => log::info!("Track selected"),
                }
            }
            //TODO: Implement Artist Selection
            ActiveBlock::UserTopArtists => match self.selected_state.selected() {
                _ => log::info!("Artist selected"),
            },
            //TODO: Implement Track Selection
            ActiveBlock::Playlist => match self.playlist.list_state.selected() {
                Some(i) => {
                    self.track_popup.list_state.select(Some(0));
                    self.route.active_block = ActiveBlock::Popup;
                    log::info!(
                        "Track selected: {:#?}",
                        self.playlist.pages.list[i].as_ref().unwrap().track
                    );
                }
                _ => log::info!("Track selected"),
            },
            //TODO: Implement Artist block
            ActiveBlock::Artist => { /* Not implemented yet */ }
            ActiveBlock::Popup => {
                self.route.active_block = self.route.hovered_block;
            }
        };
    }

    pub fn up(&mut self) {
        match self.route.active_block {
            ActiveBlock::Directory => {
                self.directory.list_state.select_previous();
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
                if self.playlist.pages.list.is_empty() {
                    return;
                }
                self.playlist.list_state.select_previous();
            }
            //TODO: Implement Artist block
            ActiveBlock::Artist => { /* Not implemented yet */ }
            ActiveBlock::Popup => {
                self.track_popup.list_state.select_previous();
            }
        }
    }

    pub fn down(&mut self) {
        match self.route.active_block {
            ActiveBlock::Directory => {
                if self.directory.list_state.selected() <= Some(DIRECTORY.len() - 2) {
                    self.directory.list_state.select_next();
                }
            }
            ActiveBlock::UserPlaylists => {
                if self.user_library.user_playlists.list_state.selected()
                    <= Some(self.user_library.user_playlists.total - 2)
                {
                    self.user_library.user_playlists.list_state.select_next();
                }
            }
            ActiveBlock::UserTopTracks => {
                if self.user_library.user_top_tracks.list_state.selected()
                    <= Some(self.user_library.user_top_tracks.total - 2)
                {
                    self.user_library.user_top_tracks.list_state.select_next();
                }
            }
            ActiveBlock::UserTopArtists => {
                if self.user_library.user_top_artists.list_state.selected()
                    <= Some(self.user_library.user_top_artists.total - 2)
                {
                    self.user_library.user_top_artists.list_state.select_next();
                }
            }
            ActiveBlock::Playlist => {
                if self.playlist.pages.list.is_empty() {
                    return;
                }
                if self.playlist.list_state.selected() <= Some(self.playlist.pages.total - 2) {
                    self.playlist.list_state.select_next();
                }
            }
            //TODO: Implement Artist block
            ActiveBlock::Artist => { /* Not implemented yet */ }
            ActiveBlock::Popup => {
                if self.track_popup.list_state.selected() <= Some(self.track_popup.list.len() - 2) {
                    self.track_popup.list_state.select_next();
                }
            }
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub async fn init(&mut self) -> color_eyre::Result<()> {
        let spotify_client = &self.spotify_client;
        self.user = Some(spotify_rs::get_current_user_profile(spotify_client).await?);

        let playlists = spotify_rs::current_user_playlists()
            .get(spotify_client)
            .await?;
        self.user_library.user_playlists.page = Some(playlists.clone());
        self.user_library.user_playlists.total = usize::try_from(playlists.total)?;
        self.user_library.user_playlists.list = playlists.items;

        let top_tracks = spotify_rs::current_user_top_tracks()
            .get(spotify_client)
            .await?;
        self.user_library.user_top_tracks.page = Some(top_tracks.clone());
        self.user_library.user_top_tracks.total = usize::try_from(top_tracks.total)?;
        self.user_library.user_top_tracks.list = top_tracks.items;

        let top_artists = spotify_rs::current_user_top_artists()
            .get(spotify_client)
            .await?;
        self.user_library.user_top_artists.page = Some(top_artists.clone());
        self.user_library.user_top_artists.total = usize::try_from(top_artists.total)?;
        self.user_library.user_top_artists.list = top_artists.items;

        Ok(())
    }
}
