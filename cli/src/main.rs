use clap::Parser;
use tokio::net::UnixStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::path::Path;
use std::io::{self, Write};

#[derive(Parser)]
#[clap(name = "web_scraper_cli")]
struct Cli {
    #[clap(short, long, default_value = "/tmp/web_scraper.sock")]
    socket_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Check if server socket exists
    if !Path::new(&cli.socket_path).exists() {
        eprintln!("Server is not running.");
        return Ok(());
    }

    // Connect to the server and keep the connection open
    let stream = UnixStream::connect(&cli.socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut response_line = String::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let command = input.trim();
        if command.is_empty() {
            continue;
        }
        if command == "exit" || command == "quit" {
            break;
        }

        // Send the command to the server
        writer.write_all(format!("{}\n", command).as_bytes()).await?;
        writer.flush().await?;

        // Read the response from the server
        response_line.clear();
        while reader.read_line(&mut response_line).await? != 0 {
            let response = response_line.trim_end();
            if response.is_empty() {
                break; // End of response
            }
            println!("{}", response);
            response_line.clear();
        }
    }

    Ok(())
}