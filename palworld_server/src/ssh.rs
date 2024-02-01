use anyhow::Result;
use ssh2::Session;
use std::io::Read;
use tokio::net::TcpStream;
use tokio::task;
use crate::mem::MemInfo;
use log::{error, info, warn};

#[derive(Debug)]
pub struct PalworldConnection {
    pub hostname: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct CommandResult {
    output: String,
    exit_status: i32,
}


impl PalworldConnection {
    pub fn new(
        hostname: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            hostname: hostname.into(),
            username: username.into(),
            password: password.into(),
        }
    }

    async fn connect(&self) -> Result<Session> {
        log::debug!("Creating TcpStream...");
        let tcp_stream = TcpStream::connect(&self.hostname).await?;
        log::debug!("Creating Session...");
        let mut session: Session = Session::new()?;
        log::debug!("Setting TcpStream to session...");
        session.set_tcp_stream(tcp_stream);
        log::debug!("Session handshake...");
        session.handshake()?;
        log::debug!("Session user auth with password...");
        session.userauth_password(self.username.as_str(), self.password.as_str())?;
        Ok(session)
    }

    pub async fn command(&self, cmd: impl Into<String>) -> Result<CommandResult> {
        let session = self.connect().await?;
        let mut channel = session.channel_session()?;

        let cmd: String = cmd.into();
        let command_result = task::spawn_blocking(move || -> Result<CommandResult> {
            channel.exec(cmd.as_str())?;
            let mut buffer = String::new();
            channel.read_to_string(&mut buffer)?;
            channel.send_eof()?;
            channel.wait_close()?;
            let exit_status = channel.exit_status()?;
            Ok(CommandResult {
                output: buffer,
                exit_status,
            })
        })
        .await??;
        Ok(command_result)
    }

    pub async fn get_memory_info(&self) -> Result<MemInfo> {
        let bytes_regex = regex::Regex::new(r"[0-9]{1,99} kB$")?;
        let result = self
            .command("cat /proc/meminfo | grep -e 'Mem' -e 'Cached' -e 'Buffers'")
            .await?;
        let mut mem_info = MemInfo::default();
        for line in result.output.split("\n") {
            if !bytes_regex.is_match(line) {
                continue;
            }
            let kb = match bytes_regex.find(line) {
                Some(kb) => kb.as_str(),
                None => continue,
            };
            if line.contains("MemTotal:") {
                mem_info.mem_total = kb.replace(" kB", "").parse()?;
            }
            if line.contains("MemFree:") {
                mem_info.mem_free = kb.replace(" kB", "").parse()?;
            }
            if line.contains("MemAvailable:") {
                mem_info.mem_available = kb.replace(" kB", "").parse()?;
            }
            if line.contains("Buffers:") {
                mem_info.buffers = kb.replace(" kB", "").parse()?;
            }
            if line.contains("Cached:") && !line.contains("SwapCached") {
                mem_info.cached = kb.replace(" kB", "").parse()?;
            }
        }

        Ok(mem_info)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dotenv::dotenv;

    fn get_connection() -> PalworldConnection {
        dotenv().unwrap();
        let hostname = std::env::var("SSH_HOSTNAME").expect("SSH_HOSTNAME env variable");
        let port: u16 = std::env::var("SSH_PORT")
            .expect("SSH_PORT env variable")
            .parse()
            .expect("Failed to convert SSH_PORT to u16");
        let hostname = format!("{hostname}:{port}");
        let username = std::env::var("SSH_USERNAME").expect("SSH_USERNAME env variable");
        let password = std::env::var("SSH_PASSWORD").expect("SSH_PASSWORD env variable");

        PalworldConnection::new(hostname, username, password)
    }

    #[tokio::test]
    async fn test_command() -> Result<()> {
        let connection = get_connection();

        let result = connection
            .command("cat /proc/meminfo | grep \"Mem\"")
            .await?;
        assert_eq!(result.exit_status, 0);
        for line in result.output.split("\n") {
            println!("{line}");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_mem_info() -> Result<()> {
        let connection = get_connection();
        let mem_info = connection.get_memory_info().await?;
        assert_ne!(mem_info, MemInfo::default());
        
        println!("{mem_info:#?}");
        println!(
            "Used memory: {}MiB {}%",
            mem_info.used().unwrap().checked_div(1024).unwrap(),
            mem_info.used_percent().unwrap()
        );
        Ok(())
        
    }
}
