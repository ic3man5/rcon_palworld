use anyhow::Result;
use clap::Parser;
use palworld_server::{
    mem,
    rcon::{PalworldRCON, DEFAULT_SOURCE_PORT},
    ssh,
};
use serde_json::json;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Args {
    #[arg(short = 'd', long = "debug_level")]
    log_level_verbosity: Option<String>,

    #[arg(value_name = "localhost")]
    /// Host of the palworld server, defaults to localhost if not specified
    server_ip: Option<String>,

    #[arg(short = 'P', long = "port", value_name = "25575")]
    /// Port of the palworld server, defaults to 25575 or 22 if not specified
    server_port: Option<u16>,

    /// Password of the palworld server (RCON or SSH)
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

    /// Tell the server to save
    #[arg(short, long)]
    save: bool,

    /// Tell the server to shutdown with a delay in seconds
    #[arg(short = 'S', long, value_name = "30")]
    shutdown: Option<u64>,

    /// Broadcast a message to the server
    #[arg(short, long)]
    broadcast: Option<String>,

    /// Broadcast space replacement String.
    #[arg(short, long)]
    replace_broadcast_space: Option<String>,

    /// Send a command to the server, result is sent to stdout.
    #[arg(short, long)]
    command: Option<String>,

    /// Get memory usage of the server
    #[arg(short, long)]
    memory: bool,

    /// Get memory usage of the server through SSH
    #[arg(short = 'M', long = "memory_ssh")]
    memory_ssh: bool,

    /// Username to use with an SSH connection
    #[arg(short, long)]
    username: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize the log before we do anything else
    initialize_log(args.log_level_verbosity)?;

    // Setup server credentials
    let server_ip = args.server_ip.unwrap_or("localhost".to_string());
    let server_port = args.server_port.unwrap_or(DEFAULT_SOURCE_PORT);

    // Connect to the server
    let server = PalworldRCON::new(&server_ip, server_port, &args.password);

    // Player info
    if args.player_info {
        let player_info = server.get_player_info().await?;
        if args.json {
            let output = serde_json::to_string(&player_info)?;
            println!("{output}");
        } else {
            println!("Got player info: found {} online!", player_info.len());
            println!("Name\tUID\tSteamID");
            for player in &player_info {
                println!("{}\t{}\t{}", player.name, player.uid, player.steamid);
            }
        }
    }
    // Server version
    if args.server_version {
        let mut version = server.get_version().await?;
        if args.json {
            version = json!({"version": version}).to_string();
        }
        println!("{}", version);
    }
    // save the server
    if args.save {
        println!("Saved: {}", server.save().await?);
    }
    // Shutdown the server
    match args.shutdown {
        Some(delay) => {
            let success = server
                .shutdown(Some(std::time::Duration::from_secs(delay)), "")
                .await?;
            println!("Shutdown: {success}");
        }
        None => {}
    }
    // Broadcast message
    match args.broadcast {
        Some(msg) => {
            let result = server.broadcast(msg, args.replace_broadcast_space).await?;
            println!("{result}");
        }
        None => {}
    }
    // Send a command
    match args.command {
        Some(cmd) => {
            let result = server.send_command(cmd.as_str()).await?;
            println!("{result}");
        }
        None => {}
    }
    // Get memory usage
    if args.memory {
        let mem_info = mem::MemInfo::get_memory_info()?;
        if args.json {
            println!("{}", serde_json::to_string(&mem_info)?);
        } else {
            println!("{mem_info:#?}");
        }
    } else if args.memory_ssh {
        // Dual purpose server_port here. We are going to grab it again and set to 22 (SSH default port now)
        let server_port = args.server_port.unwrap_or(22);
        let ssh_hostname = format!("{}:{server_port}", &server_ip);
        let username = args.username.or(Some("root".to_string())).unwrap();
        let connection = ssh::PalworldConnection::new(ssh_hostname, username, &args.password);
        let mem_info = connection.get_memory_info().await?;
        if args.json {
            println!("{}", serde_json::to_string(&mem_info)?);
        } else {
            println!("{mem_info:#?}");
        }
    }
    log::debug!("Done.");
    Ok(())
}

fn initialize_log(log_level: Option<String>) -> Result<()> {
    let log_level = match &log_level {
        Some(ll) => match &ll.to_lowercase() {
            x if x.contains("trace") => log::LevelFilter::Trace,
            x if x.contains("debug") => log::LevelFilter::Debug,
            x if x.contains("info") => log::LevelFilter::Info,
            x if x.contains("warn") => log::LevelFilter::Warn,
            x if x.contains("error") => log::LevelFilter::Error,
            _ => log::LevelFilter::Off,
        },
        None => log::LevelFilter::Off,
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stdout())
        //.chain(fern::log_file("output.log")?)
        .apply()?;

    Ok(())
}
