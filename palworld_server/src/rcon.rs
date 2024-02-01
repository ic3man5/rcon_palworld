//! API to interact with a Palworld server via RCON.
//!
//! # Example:
//! ```
//! use palworld_server::rcon::{PalworldRCON, DEFAULT_SOURCE_PORT};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a new rcon connection.
//!     let rcon = PalworldRCON::new("localhost", DEFAULT_SOURCE_PORT, "MyRCONPassword");
//!     // Get players logged in.
//!     let players = rcon.get_player_info().await.unwrap();
//!     // Print out all the active players.
//!     println!("{} Active player(s)!", players.len());
//!     for player in &players {
//!         println!("{}", player.name)
//!     }
//! }
//! ```

use std::time::Duration;

use anyhow::Result;
use rcon;
use regex::Regex;
use tokio;
use serde::{Deserialize, Serialize};

/// Default Source Engine port, Palworld uses the same port also.
pub static DEFAULT_SOURCE_PORT: u16 = 25575;

/// Representation of /showplayers rcon command
#[derive(Debug, Serialize, Deserialize)]
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
    /// use palworld_server::rcon::{PalworldRCON, DEFAULT_SOURCE_PORT};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let port = DEFAULT_SOURCE_PORT;
    ///     let rcon = PalworldRCON::new("localhost", port, "MyRCONPassword");
    ///     assert_eq!(rcon,
    ///         PalworldRCON {
    ///             host: "localhost".to_string(),
    ///             port: port,
    ///             password: "MyRCONPassword".to_string()
    ///     });
    /// }
    /// ```
    pub fn new(host: impl Into<String>, port: u16, password: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: port,
            password: password.into(),
        }
    }

    /// Connect to the server.
    async fn connect(&self) -> Result<rcon::Connection<tokio::net::TcpStream>> {
        let host = format!("{}:{}", self.host, self.port);
        let connection = <rcon::Connection<tokio::net::TcpStream>>::builder()
            .enable_factorio_quirks(true)
            .connect(host.as_str(), self.password.as_str())
            .await?;
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
    /// * `replace_string` - Replace spaces with a String, as of v0.1.3 server this is needed.
    /// 
    /// # Example:
    /// ```
    /// use palworld_server::rcon::{PalworldRCON, DEFAULT_SOURCE_PORT};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let port = DEFAULT_SOURCE_PORT;
    ///     let rcon = PalworldRCON::new("localhost", port, "MyRCONPassword");
    ///     assert_eq!(
    ///         rcon.broadcast(
    ///             "Test Message",
    ///             Some("_".to_string())
    ///         ).await.unwrap(), 
    ///             String::from("Broadcasted: Test_Message\n")
    ///     );
    /// }
    /// ```
    pub async fn broadcast(
        &self,
        message: impl Into<String>,
        replace_space: Option<String>,
    ) -> Result<String> {
        let message: String = message.into();
        let message = match &replace_space {
            Some(s) => message.replace(" ", s),
            None => message,
        };
        let message = format!("broadcast {message}");
        self.send_command(message.as_str()).await
    }

    /// Gets active player information. Returns a vector of [PlayerInfo].
    ///
    /// # Example:
    /// ```
    /// use palworld_server::rcon::{PalworldRCON, DEFAULT_SOURCE_PORT};
    ///
    /// #[tokio::main]
    /// async fn main() {
    /// // Create a new rcon connection.
    ///     let rcon = PalworldRCON::new("localhost", DEFAULT_SOURCE_PORT, "MyRCONPassword");
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

    /// Sends a save command to the server via RCON. Returns true if server successfully saved.
    pub async fn shutdown(&self, delay: Option<Duration>, msg: impl Into<String>) -> Result<bool> {
        let cmd = format!(
            "shutdown {} {}",
            delay.unwrap_or(Duration::new(30, 0)).as_secs(),
            msg.into()
        );
        let msg = self.send_command(cmd.as_str()).await?;
        println!("shutdown msg: {}", msg);
        Ok(msg.contains("The server will shut down in"))
    }

    pub async fn get_version(&self) -> Result<String> {
        // Welcome to Pal Server[v0.1.3.0] Default Palworld Server
        let result = self.send_command("info").await?;
        let re = Regex::new(r"\[v[0-9]{1,9}\.[0-9]{1,9}\.[0-9]{1,9}\.[0-9]{1,9}\]")?;
        if !re.is_match(&result) {
            anyhow::bail!("Failed to find version in info RCON command")
        }
        let version = re
            .find(result.as_str())
            .expect("Failed to find version information")
            .as_str();
        
        // remove the brackets at the front and end
        let version = Regex::new(r"(\[)|(\])").unwrap().replace_all(version, "");

        Ok(version.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    fn get_server() -> PalworldRCON {
        dotenv().unwrap();
        let hostname = std::env::var("HOSTNAME").expect("HOSTNAME env variable");
        let port = std::env::var("RCON_PORT")
            .expect("RCON_PORT env variable")
            .parse()
            .expect("Failed to convert RCON_PORT to u16");
        let password = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD env variable");

        PalworldRCON::new(hostname, port, password)
    }

    #[tokio::test]
    async fn test_commands_overload() {
        let server = get_server();
        for i in 0..100 {
            println!("{i} {}", server.send_command("info").await.unwrap());
        }
    }

    #[tokio::test]
    async fn test_save() {
        let server = get_server();
        assert!(server.save().await.unwrap());
    }

    #[tokio::test]
    async fn test_shutdown() {
        let server = get_server();
        assert!(server.shutdown(None, "MESSAGE_HERE").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_player_info() {
        let server = get_server();
        let players = server.get_player_info().await.unwrap();
        println!("Player count: {}", players.len());
        println!(
            "{}",
            server
                .broadcast(format!("DEBUG: Player Count: {}", players.len()), Some("_".to_string()))
                .await
                .unwrap()
        );
        for (i, player) in players.into_iter().enumerate() {
            println!(
                "{}",
                server
                    .broadcast(format!("DEBUG: {i}. {}", player.name), Some("_".to_string()))
                    .await
                    .unwrap()
            );
        }
    }

    #[tokio::test]
    async fn test_commands_memory_broadcast() {
        let server = get_server();

        let mem = psutil::memory::virtual_memory().unwrap();
        println!(
            "{}",
            server
                .broadcast(
                    format!("DEBUG: memory free:  {}MiB", mem.free() / (1024 * 1024)),
                    Some("_".to_string())
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
                    Some("_".to_string())
                )
                .await
                .unwrap()
        );
        println!(
            "{}",
            server
                .broadcast(
                    format!("DEBUG: memory total: {}MiB", mem.total() / (1024 * 1024)),
                    Some("_".to_string())
                )
                .await
                .unwrap()
        );
        println!("{}", server.send_command("info").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_version() {
        let server = get_server();
        let version = server.get_version().await.unwrap();
        assert!(version.contains("v0"));
        println!("{version}");
    }
}
