use palworldrcon;
use clap::Parser;
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Host of the palworld server
    #[arg(long)]
    host: String,

    #[arg(long)]
    /// Port of the palworld server
    port: u16,

    /// Password of the palworld server
    #[arg(long)]
    pass: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("Host: {}:{}\nPassword: {}", args.host, args.port, args.pass);

    println!("Connecting to {}:{}...", args.host, args.port);
    let server = palworldrcon::PalworldRCON::new(args.host, Some(args.port), args.pass);

    let player_info = server.get_player_info().await?;
    println!("Got player info: found {} online!", player_info.len());
    for player in &player_info {
        println!("{:#?}", player);
    }
    Ok(())
}
