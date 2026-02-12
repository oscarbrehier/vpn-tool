use std::io::Read;
use anyhow::Context;
use ssh2::Session;

pub fn run_remote_cmd(session: &Session, cmd: &str) -> anyhow::Result<(String, i32)> {
    let mut channel = session
        .channel_session()
        .context("Failed to open SSH channel")?;

    channel
        .exec(cmd)
        .context(format!("Failed to execute cmd: {}", cmd))?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    channel.wait_close()?;

    let exit_status = channel.exit_status()?;
    Ok((output, exit_status))
}
