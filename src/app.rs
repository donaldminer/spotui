use crate::event::{AppEvent, Event, EventHandler};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use reqwest::Url;
use rouille::{Response, Server};
use spotify_rs::model::Page;
use spotify_rs::{
    AuthCodePkceClient, AuthCodePkceFlow, RedirectUrl, Token,
    client::Client,
    model::{
        artist::Artist,
        player::{Device, Queue},
        playlist::SimplifiedPlaylist,
        track::Track,
    },
};
use std::sync::{Arc, Mutex};
use tui_logger::TuiWidgetState;

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

#[derive(Debug)]
pub struct UserProfile {
    pub display_name: String,
    pub product: String,
}
#[derive(Debug, Default)]
pub struct UserTopItems {
    pub tracks: Option<Page<Track>>,
    pub artists: Option<Page<Artist>>,
}
#[derive(Debug, Clone)]
pub struct UserPlaylists {
    pub playlists: Option<Page<SimplifiedPlaylist>>,
    pub playlists_state: ratatui::widgets::ListState,
}
impl Default for UserPlaylists {
    fn default() -> Self {
        Self {
            playlists: None,
            playlists_state: ratatui::widgets::ListState::default(),
        }
    }
}

#[derive(Debug)]
pub struct CurrentUser {
    pub user_profile: UserProfile,
    pub user_top_items: UserTopItems,
    pub user_playlists: UserPlaylists,
}
impl Default for CurrentUser {
    fn default() -> Self {
        Self {
            user_profile: UserProfile {
                display_name: String::new(),
                product: String::new(),
            },
            user_top_items: UserTopItems {
                tracks: None,
                artists: None,
            },
            user_playlists: UserPlaylists::default(),
        }
    }
}

#[derive(Debug)]
pub struct Player {
    pub device_id: Vec<Device>,
    pub queue: Option<Queue>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            device_id: vec![],
            queue: None,
        }
    }
}

pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub spotify_client: Client<Token, AuthCodePkceFlow>,
    pub current_user: CurrentUser,
    pub player: Player,
    pub state: TuiWidgetState,
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
            current_user: CurrentUser::default(),
            player: Player::default(),
            state: TuiWidgetState::new().set_default_display_level(log::LevelFilter::Debug),
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

        println!("Navigate to {url} to complete the OAuth process.");

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
            KeyCode::Tab => {}
            KeyCode::Up | KeyCode::Char('k') => self.up(),
            KeyCode::Down | KeyCode::Char('j') => self.down(),
            KeyCode::Left | KeyCode::Char('h') => {}
            KeyCode::Right | KeyCode::Char('l') => {}
            _ => {}
        }
        Ok(())
    }

    pub fn up(&mut self) {
        let i = match self.current_user.user_playlists.playlists_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.current_user
            .user_playlists
            .playlists_state
            .select(Some(i));
    }
    pub fn down(&mut self) {
        let i = match self.current_user.user_playlists.playlists_state.selected() {
            Some(i) => {
                if i >= self
                    .current_user
                    .user_playlists
                    .playlists
                    .as_ref()
                    .unwrap()
                    .items
                    .len()
                    - 1
                {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.current_user
            .user_playlists
            .playlists_state
            .select(Some(i));
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub async fn init(&mut self) -> color_eyre::Result<()> {
        self.get_user_playlists().await?;
        self.get_user_profile().await?;
        self.get_user_top_items().await?;
        self.get_player().await?;
        Ok(())
    }

    pub async fn get_player(&mut self) -> color_eyre::Result<()> {
        self.player = Player {
            device_id: spotify_rs::get_available_devices(&self.spotify_client).await?,
            queue: Some(spotify_rs::get_user_queue(&self.spotify_client).await?),
        };
        Ok(())
    }
    pub async fn get_user_playlists(&mut self) -> color_eyre::Result<()> {
        self.current_user.user_playlists.playlists = Some(
            spotify_rs::current_user_playlists()
                .get(&self.spotify_client)
                .await?,
        );
        Ok(())
    }

    pub async fn get_user_profile(&mut self) -> color_eyre::Result<()> {
        self.current_user.user_profile = spotify_rs::get_current_user_profile(&self.spotify_client)
            .await
            .map(|profile| UserProfile {
                display_name: profile.display_name.unwrap_or_default(),
                product: profile.product.unwrap_or_default(),
            })?;
        Ok(())
    }

    pub async fn get_user_top_items(&mut self) -> color_eyre::Result<()> {
        self.current_user.user_top_items = UserTopItems {
            tracks: Some(
                spotify_rs::current_user_top_tracks()
                    .get(&self.spotify_client)
                    .await?,
            ),
            artists: Some(
                spotify_rs::current_user_top_artists()
                    .get(&self.spotify_client)
                    .await?,
            ),
        };
        Ok(())
    }
}
