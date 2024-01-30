use palworldrcon::{self, DEFAULT_SOURCE_PORT};
use clap::Parser;
use anyhow::Result;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Args {
    #[arg(value_name = "localhost")]
    /// Host of the palworld server, defaults to localhost if not specified
    server_ip: Option<String>,

    #[arg(short = 'P', long = "port", value_name = "25575")]
    /// Port of the palworld server, defaults to 25575 if not specified
    server_port: Option<u16>,

    /// Password of the palworld server
    #[arg(short = 'p', long)]
    password: String,

    /// output in json format
    #[arg(short, long)]
    json: bool,

    /// Get player name, Unique ID, and SteamID
    #[arg(short = 'l', long = "list")]
    player_info: bool,

    /// Get server version
    #[arg(short = 'v', long = "server_version")]
    server_version: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    // Setup server credentials
    let server_ip = args.server_ip.unwrap_or("localhost".to_string());
    let server_port = args.server_port.unwrap_or(DEFAULT_SOURCE_PORT);
    //let hostname = format!("{server_ip}:{server_port}");

    // Connect to the server
    let server = palworldrcon::PalworldRCON::new(server_ip, server_port, args.password);

    // Player info
    if args.player_info {
        let player_info = server.get_player_info().await?;
        if args.json {
            let output = serde_json::to_string(&player_info)?;
            println!("{output}");
        }
        else {
            println!("Got player info: found {} online!", player_info.len());
            println!("Name\tUID\tSteamID");
            for player in &player_info {
                println!("{}\t{}\t{}", player.name, player.uid, player.steamid);
            }
        }
    } 
    if args.server_version {
        let mut version = server.get_version().await?;
        if args.json {
            version = json!({"version": version}).to_string();            
        }
        println!("{}", version);
    }
    
    Ok(())
}
