//! API to interact with a Palworld server via RCON.
//! 
//! # Example:
//! ```
//! use palworld_rcon::{PalworldRCON, DEFAULT_SOURCE_PORT};
//! 
//! #[tokio::main]
//! async fn main() {
//!     // Create a new rcon connection.
//!     let rcon = PalworldRCON::new("localhost", Some(DEFAULT_SOURCE_PORT), "MyRCONPassword");
//!     // Get players logged in.
//!     let players = rcon.get_player_info().await.unwrap();
//!     // Print out all the active players.
//!     println!("{} Active player(s)!", players.len());
//!     for player in &players {
//!         println!("{}", player.name)
//!     }
//! }
//! ```

use anyhow::Result;
use rcon;
use tokio;

/// Default Source Engine port, Palworld uses the same port also.
pub static DEFAULT_SOURCE_PORT: u16 = 25575;

/// Representation of /showplayers rcon command
#[derive(Debug)]
pub struct PlayerInfo {
    /// Player's name.
    pub name: String,
    /// Player's Unique ID inside the server.
    pub uid: String,
    /// Player's Steam ID.
    pub steamid: String,
}

/// Palworld Server RCON
#[derive(Debug, PartialEq)]
pub struct PalworldRCON {
    /// Server hostname or IP address. "localhost" or "127.0.0.1" for the same machine.
    pub host: String,
    /// Server port, typically (DEFAULT_SOURCE_PORT).
    pub port: u16,
    /// Server RCON password.
    pub password: String,
}



impl PalworldRCON {
    /// Create a new [PalworldRCON] instance.
    ///
    /// # Example:
    /// ```
    /// use palworld_rcon::{PalworldRCON, DEFAULT_SOURCE_PORT};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let port = DEFAULT_SOURCE_PORT;
    ///     let rcon = PalworldRCON::new("localhost", Some(port), "MyRCONPassword");
    ///     assert_eq!(rcon, 
    ///         PalworldRCON { 
    ///             host: "localhost".to_string(), 
    ///             port: port, 
    ///             password: "MyRCONPassword".to_string() 
    ///     });
    /// }
    /// ```
    pub fn new(host: impl Into<String>, port: Option<u16>, password: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: match &port {
                Some(p) => p.to_owned(),
                None => DEFAULT_SOURCE_PORT,
            },
            password: password.into(),
        }
    }

    /// Connect to the server.
    async fn connect(&self) -> Result<rcon::Connection<tokio::net::TcpStream>> {
        let host = format!("{}:{}", self.host, self.port);
        println!("Connecting...");
        let connection = <rcon::Connection<tokio::net::TcpStream>>::builder()
            .enable_factorio_quirks(true)
            .connect(host.as_str(), self.password.as_str())
            .await?;
        println!("connected!");

        Ok(connection)
    }

    /// Sends a command to the server via RCON. Returns a string of the command result.
    pub async fn send_command(&self, cmd: impl Into<&str>) -> Result<String> {
        let mut conn = self.connect().await?;

        Ok(conn.cmd(cmd.into()).await?)
    }

    /// Sends a broadcast command to the server via RCON. Returns a string of the command result.
    /// 
    /// # Arguments:
    /// * `message` - The message to broadcast to the server
    /// * `replace_string` - Replace spaces with underscores, as of v0.1.3 server this is needed.
    pub async fn broadcast(
        &self,
        message: impl Into<String>,
        replace_space: bool,
    ) -> Result<String> {
        let message: String = message.into();
        let message = if replace_space {
            message.replace(" ", "_")
        } else {
            message.into()
        };
        let message = format!("broadcast {message}");
        self.send_command(message.as_str()).await
    }

    /// Gets active player information. Returns a vector of [PlayerInfo].
    /// 
    /// # Example:
    /// ```
    /// use palworld_rcon::{PalworldRCON, DEFAULT_SOURCE_PORT};
    /// 
    /// #[tokio::main]
    /// async fn main() {
    /// // Create a new rcon connection.
    ///     let rcon = PalworldRCON::new("localhost", Some(DEFAULT_SOURCE_PORT), "MyRCONPassword");
    ///     // Get players logged in.
    ///     let players = rcon.get_player_info().await.unwrap();
    ///     // Print out all the active players.
    ///     println!("{} Active player(s)!", players.len());
    ///     for player in &players {
    ///         println!("{}", player.name)
    ///     }
    /// }
    /// ```
    pub async fn get_player_info(&self) -> Result<Vec<PlayerInfo>> {
        let results = self
            .send_command("showplayers")
            .await?
            .split("\n")
            .skip(1)
            .filter_map(|info| {
                let split = info.split(",").collect::<Vec<&str>>();
                if split.len() != 3 {
                    return None;
                }
                Some(PlayerInfo {
                    name: split[0].to_string(),
                    uid: split[1].to_string(),
                    steamid: split[2].to_string(),
                })
            })
            .collect::<Vec<PlayerInfo>>();

        Ok(results)
    }

    /// Sends a save command to the server via RCON. Returns true if server successfully saved.
    pub async fn save(&self) -> Result<bool> {
        let msg = self.send_command("save").await?;
        Ok(msg.contains("Complete Save"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_commands() {
        let server = PalworldRCON::new("localhost", Some(DEFAULT_SOURCE_PORT), "MyRCONPassword");
        for i in 0..100 {
            println!("{i} {}", server.send_command("info").await.unwrap());
        }
    }

    #[tokio::test]
    async fn test_save() {
        let server = PalworldRCON::new("localhost", Some(DEFAULT_SOURCE_PORT), "MyRCONPassword");
        println!("{:#?}", server.save().await.unwrap());
    }

    #[tokio::test]
    async fn test_get_player_info() {
        let server = PalworldRCON::new("localhost", Some(DEFAULT_SOURCE_PORT), "MyRCONPassword");
        let players = server.get_player_info().await.unwrap();
        println!("Player count: {}", players.len());
        println!(
            "{}",
            server
                .broadcast(format!("DEBUG: Player Count: {}", players.len()), true)
                .await
                .unwrap()
        );
        for (i, player) in players.into_iter().enumerate() {
            println!(
                "{}",
                server
                    .broadcast(format!("DEBUG: {i}. {}", player.name), true)
                    .await
                    .unwrap()
            );
        }
    }

    #[tokio::test]
    async fn test_commands_asdf() {
        let server = PalworldRCON::new("localhost", Some(DEFAULT_SOURCE_PORT), "MyRCONPassword");

        let mem = psutil::memory::virtual_memory().unwrap();
        println!(
            "{}",
            server
                .broadcast(
                    format!("DEBUG: memory free:  {}MiB", mem.free() / (1024 * 1024)),
                    true
                )
                .await
                .unwrap()
        );
        println!(
            "{}",
            server
                .broadcast(
                    format!(
                        "DEBUG: memory used:  {}MiB ({:.2}%)",
                        mem.used() / (1024 * 1024),
                        mem.percent()
                    ),
                    true
                )
                .await
                .unwrap()
        );
        println!(
            "{}",
            server
                .broadcast(
                    format!("DEBUG: memory total: {}MiB", mem.total() / (1024 * 1024)),
                    true
                )
                .await
                .unwrap()
        );
        println!("{}", server.send_command("info").await.unwrap());
    }
}
