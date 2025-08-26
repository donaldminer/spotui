use crate::event::{AppEvent, Event, EventHandler};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub struct App {
    state: AppState,
    active_block: ActiveBlock,
    events: EventHandler,
}

#[derive(Debug)]
enum AppState {
    Running,
    Exiting,
}
#[derive(Debug)]
enum ActiveBlock {
    Left,
    TopRight,
    BottomRight,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Running,
            active_block: ActiveBlock::Left,
            events: EventHandler::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
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
            _ => {}
        }
        Ok(())
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }
}
