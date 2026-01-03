use serde::{Deserialize, Serialize};

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
