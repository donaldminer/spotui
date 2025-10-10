use crate::app::App;
use log::LevelFilter;
use tui_logger;

mod app;
mod event;
mod ui;
mod widgets;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(LevelFilter::Debug);
    color_eyre::install()?;
    let terminal = ratatui::init();
    log::info!("Starting application");
    let result = App::new().await.run(terminal).await;
    ratatui::restore();
    result
}
