use crate::mem::MemInfo;
use anyhow::Result;
use log::{error, info, warn};
use ssh2::Session;
use std::io::Read;
use tokio::net::TcpStream;
use tokio::task;

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
        log::trace!("Connecting to {}...", self.hostname);
        let tcp_stream = TcpStream::connect(&self.hostname).await?;
        let mut session: Session = Session::new()?;
        session.set_tcp_stream(tcp_stream);
        log::trace!("Session handshake...");
        session.handshake()?;
        log::trace!("Session user auth with password...");
        session.userauth_password(self.username.as_str(), self.password.as_str())?;
        log::trace!("Session userauth with password ok!");
        Ok(session)
    }

    pub async fn command(&self, cmd: impl Into<String>) -> Result<CommandResult> {
        let session = self.connect().await?;
        log::trace!("Creating new channel");
        let mut channel = session.channel_session()?;

        let cmd: String = cmd.into();
        let command_result = task::spawn_blocking(move || -> Result<CommandResult> {
            log::info!("Executing command '{}'", &cmd);
            channel.exec(cmd.as_str())?;
            let mut buffer = String::new();
            channel.read_to_string(&mut buffer)?;
            log::trace!("Sending EOF");
            channel.send_eof()?;
            log::trace!("Waiting for close...");
            channel.wait_close()?;
            let exit_status = channel.exit_status()?;
            log::info!("Exit status: {exit_status}");
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
        let cmd = "cat /proc/meminfo | grep -e 'Mem' -e 'Cached' -e 'Buffers'";
        let result = self
            .command(cmd)
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
            match &line.to_lowercase() {
                x if x.contains("memtotal:") => {
                    mem_info.mem_total = kb.replace(" kB", "").parse()?
                }
                x if x.contains("memfree:") => mem_info.mem_free = kb.replace(" kB", "").parse()?,
                x if x.contains("memavailable:") => {
                    mem_info.mem_available = kb.replace(" kB", "").parse()?
                }
                x if x.contains("buffers:") => mem_info.buffers = kb.replace(" kB", "").parse()?,
                x if x.contains("cached:") => mem_info.cached = kb.replace(" kB", "").parse()?,
                _ => (),
            };
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
