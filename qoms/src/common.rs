use tokio::process::Command;
use crate::prelude::*;

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
