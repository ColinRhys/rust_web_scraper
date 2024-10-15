use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use scraper_lib::{manager::ScraperManager, utils::normalize_url};
use std::sync::Arc;
use clap::Parser;
use std::path::Path;
use log::{info, error, debug};

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "/tmp/web_scraper.sock")]
    socket_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Remove existing socket if present
    if Path::new(&args.socket_path).exists() {
        tokio::fs::remove_file(&args.socket_path).await?;
    }

    let listener = UnixListener::bind(&args.socket_path)?;
    info!("Server listening on {}", args.socket_path);

    let manager = Arc::new(ScraperManager::new());

    loop {
        let (socket, _) = listener.accept().await?;
        let manager = Arc::clone(&manager);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, manager).await {
                error!("Error handling connection: {}", e);
            }
        });
    }
}

async fn handle_connection(socket: tokio::net::UnixStream, manager: Arc<ScraperManager>) -> anyhow::Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? != 0 {
        let command = line.trim();
        debug!("Received command: {}", command);
        let mut parts = command.split_whitespace();
        match parts.next() {
            Some("start") => {
                if let Some(input_url) = parts.next() {
                    if let Some(url) = normalize_url(input_url) {
                    manager.start_scraping(url.to_string());
                    info!("Started scraping {}", url);
                    writer.write_all(b"Started scraping\n").await?;
                    } else {
                        writer.write_all(b"Missing URL\n").await?;
                    }
                }
            }
            Some("stop") => {
                if let Some(input_url) = parts.next() {
                    if let Some(url) = normalize_url(input_url) {
                        manager.stop_scraping(&url);
                        info!("Stopped scraping {}", url);
                        writer.write_all(b"Stopped scraping\n").await?;
                    } else {
                        writer.write_all(b"Invalid URL\n").await?;
                    }
                } else {
                    writer.write_all(b"Missing URL\n").await?;
                }
            }
            Some("list") => {
                let tasks = manager.list_tasks();
                info!("Listing command");
                for (url, status) in tasks {
                    writer.write_all(format!("{}: {}\n", url, status).as_bytes()).await?;
                }
            }
            Some("print") => {
                if let Some(input_url) = parts.next() {
                    if let Some(url) = normalize_url(input_url) {
                        if let Some(links) = manager.get_links(&url) {
                            for link in links {
                                writer.write_all(format!("{}\n", link).as_bytes()).await?;
                            }
                        } else {
                            writer.write_all(b"No links found or scraping not completed.\n").await?;
                        }
                    } else {
                        writer.write_all(b"Invalid URL\n").await?;
                    }
                } else {
                    writer.write_all(b"Missing URL\n").await?;
                }
            }
            Some("help") => {
                let help_text = "Available commands:
                    start <url> - Start scraping the specified URL
                    stop <url>  - Stop scraping the specified URL
                    list        - List current scraping tasks
                    print <url> - Print links found for the specified URL
                    exit        - Exit the CLI";
                writer.write_all(help_text.as_bytes()).await?;
                writer.write_all(b"\n").await?;
            }
            Some("exit") | Some("quit") => {
                info!("Client requested to close the connection.");
                break;
            }
            _ => {
                writer.write_all(b"Unknown command\n").await?;
            }
        }
        writer.write_all(b"\n").await?; // Send an empty line to indicate end of response
        writer.flush().await?; // Ensure the response is sent immediately
        line.clear();
    }

    Ok(())
}