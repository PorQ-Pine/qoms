use serde::{Deserialize, Serialize};
use log::debug;
use tokio::process::Command;

pub const QOMS_SOCKET_PATH: &'static str = "/run/qoms.sock";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Splash {
    PowerOff,
    Reboot,
    Sleep,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SendToQoms {
    RequestSplash(Splash),
    RequestReLogin,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AnswerFromQoms {

}

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

// Outputs sessions ID, User
pub async fn find_session() -> Option<(String, String)> {
    let string = run_cmd("loginctl list-sessions --no-legend").await;
    // debug!("String is: {}", string);
    for line in string.split("\n") {
        if line.contains("seat0") {
            let vec: Vec<String> = line.split(" ").map(|s| s.to_string()).collect();
            if vec.len() > 2 {
                if vec[2].clone() == "greetd" {
                    continue;
                }
                if vec[0].clone().is_empty() {
                    continue;
                }
                let to_return = Some((vec[0].clone(), vec[2].clone()));
                debug!("session: {:?}", to_return);
                return to_return;
            }
            return None;
        }
    }
    None
}
