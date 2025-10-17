use crate::{
    event::{AppEvent, Event, EventHandler},
    spotify_handler::SpotifyHandler,
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    widgets::{ListItem, ListState},
};
use spotify_rs::model::{
    Page,
    artist::Artist,
    playlist::{Playlist, PlaylistItem, SimplifiedPlaylist},
    track::Track,
    user::PrivateUser,
};
use tui_logger::TuiWidgetState;

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
    Logger,
}

#[derive(Debug)]
pub struct Route {
    pub active_block: ActiveBlock,
    pub hovered_block: ActiveBlock,
}

pub enum SelectedTab {
    Main = 0,
    Logger = 1,
}

pub struct SelectedItem {
    pub playlist: Option<SimplifiedPlaylist>,
    pub track: Option<Track>,
    pub artist: Option<Artist>,
}

pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub spotify_handler: SpotifyHandler,
    pub user_library: UserLibrary,
    pub directory: NavList,
    pub selected_state: ListState,
    pub user: Option<PrivateUser>,
    pub route: Route,
    pub playlist: TrackList<Playlist, PlaylistItem>,
    pub track_popup: NavList,
    pub logger_state: TuiWidgetState,
    pub selected_tab: SelectedTab,
    pub selected_item: SelectedItem,
}
impl App {
    pub async fn new() -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            spotify_handler: SpotifyHandler::new().await,
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
            selected_tab: SelectedTab::Main,
            selected_item: SelectedItem {
                playlist: None,
                track: None,
                artist: None,
            },
            logger_state: TuiWidgetState::default(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        terminal.clear()?;

        log::debug!("Sending Init event");
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
                    AppEvent::GetUserProfile => {
                        self.set_user(self.spotify_handler.get_user().await?);
                    }
                    AppEvent::GetPlaylist => {
                        self.set_playlist(
                            self.spotify_handler
                                .get_playlist(self.selected_item.playlist.as_ref())
                                .await?,
                        );
                    }
                    AppEvent::GetTrack => {
                        self.set_track(self.selected_item.track.as_ref().unwrap().clone())
                    }
                    AppEvent::GetUserPlaylists => {
                        self.set_user_playlists(self.spotify_handler.get_user_playlists().await?);
                    }
                    AppEvent::GetUserTopTracks => {
                        self.set_user_top_tracks(self.spotify_handler.get_top_tracks().await?);
                    }
                    AppEvent::GetUserTopArtists => {
                        self.set_user_top_artists(self.spotify_handler.get_top_artists().await?);
                    }
                },
            }
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        log::debug!("Key event: {:#?}", key_event);
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
            KeyCode::Char('1') => {
                self.selected_tab = SelectedTab::Main;
                self.route.active_block = self.route.hovered_block;
            }
            KeyCode::Char('2') => {
                self.selected_tab = SelectedTab::Logger;
                self.route.active_block = ActiveBlock::Logger;
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn next(&mut self) {
        log::info!("Switching active block from {:#?}", self.route.active_block);
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
                self.route.hovered_block = self.route.active_block;
            }
            ActiveBlock::UserPlaylists => {
                match self.user_library.user_playlists.list_state.selected() {
                    Some(i) => {
                        self.selected_item.playlist =
                            self.user_library.user_playlists.list[i].clone();

                        self.events.send(AppEvent::GetPlaylist);
                    }
                    _ => {}
                }
                self.route.active_block = ActiveBlock::Playlist;
                self.route.hovered_block = ActiveBlock::Playlist;
            }
            //TODO: Implement Track Selection
            ActiveBlock::UserTopTracks => {
                match self.user_library.user_top_tracks.list_state.selected() {
                    Some(_i) => {
                        self.track_popup.list_state.select(Some(0));
                        self.route.active_block = ActiveBlock::Popup;
                    }
                    _ => log::info!("No Track selected"),
                }
            }
            //TODO: Implement Artist Selection
            ActiveBlock::UserTopArtists => match self.selected_state.selected() {
                _ => log::info!("Artist selected"),
            },
            //TODO: Implement Track Selection
            ActiveBlock::Playlist => match self.playlist.list_state.selected() {
                Some(_i) => {
                    self.track_popup.list_state.select(Some(0));
                    self.route.active_block = ActiveBlock::Popup;
                }
                _ => log::info!("No Playlist Track selected"),
            },
            //TODO: Implement Artist block
            ActiveBlock::Artist => {
                log::info!("Artist block selected");
            }
            ActiveBlock::Popup => {
                self.route.active_block = self.route.hovered_block;
            }
            _ => { /* Do nothing */ }
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
            ActiveBlock::Logger => {
                self.logger_state
                    .transition(tui_logger::TuiWidgetEvent::PrevPageKey);
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
            ActiveBlock::Logger => {
                self.logger_state
                    .transition(tui_logger::TuiWidgetEvent::NextPageKey);
            }
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        log::info!("Quitting application");
        self.running = false;
    }

    pub async fn init(&mut self) -> color_eyre::Result<()> {
        self.events.send(AppEvent::GetUserProfile);
        self.events.send(AppEvent::GetUserPlaylists);
        self.events.send(AppEvent::GetUserTopTracks);
        self.events.send(AppEvent::GetUserTopArtists);

        Ok(())
    }

    fn set_playlist(&mut self, playlist: Playlist) {
        self.playlist.result = Some(playlist.clone());
        self.playlist.pages.list = playlist.tracks.items;
        self.playlist.pages.total = usize::try_from(playlist.tracks.total).unwrap();
        self.playlist.list_state = ListState::default();
    }

    fn set_track(&mut self, track: Track) {
        self.selected_item.track = Some(track);
    }

    fn set_user(&mut self, user: PrivateUser) {
        self.user = Some(user);
    }

    fn set_user_top_tracks(&mut self, top_tracks: Page<Track>) {
        self.user_library.user_top_tracks.page = Some(top_tracks.clone());
        self.user_library.user_top_tracks.total = usize::try_from(top_tracks.total).unwrap();
        self.user_library.user_top_tracks.list = top_tracks.items;
    }

    fn set_user_playlists(&mut self, playlists: Page<SimplifiedPlaylist>) {
        self.user_library.user_playlists.page = Some(playlists.clone());
        self.user_library.user_playlists.total = usize::try_from(playlists.total).unwrap();
        self.user_library.user_playlists.list = playlists.items;
    }

    fn set_user_top_artists(&mut self, top_artists: Page<Artist>) {
        self.user_library.user_top_artists.page = Some(top_artists.clone());
        self.user_library.user_top_artists.total = usize::try_from(top_artists.total).unwrap();
        self.user_library.user_top_artists.list = top_artists.items;
    }
}
