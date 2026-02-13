use anyhow::{Context, Ok};
use clap::builder::Str;
use ssh2::Session;
use std::io::Read;

pub fn run_remote_cmd(session: &Session, cmd: &str) -> anyhow::Result<(String, i32)> {
    let mut channel = session
        .channel_session()
        .context("Failed to open SSH channel")?;

    channel
        .exec(cmd)
        .context(format!("Failed to execute cmd: {}", cmd))?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;

    channel.send_eof()?;
    channel.wait_eof()?;

    channel.wait_close()?;

    let exit_status = channel.exit_status()?;
    Ok((output, exit_status))
}

pub fn harden_ssh(session: &Session) -> anyhow::Result<()> {
    println!("disabling password authentication");

    let cmd = r#"
        sudo sed -i 's/^#\?PasswordAuthentication .*/PasswordAuthentication no/' /etc/ssh/sshd_config && \
        sudo sed -i 's/^#\?ChallengeResponseAuthentication .*/ChallengeResponseAuthentication no/' /etc/ssh/sshd_config && \
        (sleep 1 && sudo systemctl restart ssh) > /dev/null 2>&1 &
        echo "DONE"
    "#;

    let mut channel = session.channel_session()?;
    channel.exec(cmd)?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;

    if output.contains("DONE") {
        println!("SSH now locked to key-only access (restarting in 1s)");
    } else {
        println!("SSH command sent, but verify the restart manually");
    };

    Ok(())
}
