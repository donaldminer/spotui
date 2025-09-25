# spotui

A Spotify TUI client in Rust.
This project is a work in progress and is not yet functional. It is intended to be a terminal user interface for Spotify, allowing users to interact with their Spotify account and control playback from the terminal.

## Current Features

- Browse User's Playlists, Top Tracks, and Albums

## Built With

- [Rust](https://www.rust-lang.org/) - The programming language used
- [Crossterm](https://crates.io/crates/crossterm)
  - For handling terminal input and output
- [Spotify Web API](https://developer.spotify.com/documentation/web-api/) - For interacting with Spotify's services
- [spotify-rs](https://github.com/spotify-rs/spotify-rs) - For interacting with Spotify's services

## Getting Started

To get a local copy up and running, follow these steps.

### Prerequisites

- Rust and Cargo installed. You can download them from [here](https://www.rust-lang.org/tools/install).
- A Spotify Developer account. You can sign up for one [here](https://developer.spotify.com/dashboard/applications).
- Create a new application in the Spotify Developer Dashboard and get your Client ID and Client Secret.
- Set the Redirect URI in your Spotify Developer Dashboard to `http://localhost:8888/callback`.

### Installation

1. Clone the repository
   ```sh
   git clone https://github.com/donaldminer/spotui.git
   cd spotui
   ```
2. Set your Spotify Client ID and Client Secret as environment variables in a .env file
   ```sh
   export SPOTIFY_CLIENT_ID="your_client_id"
   export SPOTIFY_CLIENT_SECRET="your_client_secret"
   ```
3. Build the project
   ```sh
   cargo build --release
   ```
4. Run the project
   ```sh
   cargo run --release
   ```
5. Follow the instructions in the terminal to authenticate with Spotify.

## Usage

- Use the arrow keys or `h`/`j`/`k`/`l` to navigate through the menus.
- Press `Enter` to select an item.
- Press `q`/`Ctrl+C` to quit the application.
- Press `Tab` to toggle between Widgets.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any changes or improvements
you would like to make.

## Acknowledgments

- Thanks to the developers of the libraries used in this project.

