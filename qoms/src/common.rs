use crate::prelude::*;
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

pub async fn run_cmd(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    debug!("Running run_cmd as: {} {:?}", parts[0], &parts[1..]);
    let out = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .await
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub async fn read_write_socket<Send, Receive>(
    socket_path: &str,
    send: Send,
) -> anyhow::Result<Receive>
where
    Send: Serialize,
    Receive: DeserializeOwned,
{
    let vec_to_send = postcard::to_allocvec(&send)?;
    let mut last_error = None;

    for _ in 0..5 {
        let result = async {
            let mut stream = UnixStream::connect(socket_path).await?;
            stream.write_all(&vec_to_send).await?;
            stream.shutdown().await?;
            let mut message_bytes = Vec::new();
            stream.read_to_end(&mut message_bytes).await?;
            let response = postcard::from_bytes(&message_bytes)?;
            Ok(response)
        }
        .await;

        match result {
            Ok(value) => return Ok(value),
            Err(e) => last_error = Some(e),
        }
        sleep(Duration::from_millis(200)).await;
    }

    error!("read_write_socket error: {:?}", last_error);
    Err(last_error.unwrap())
}
