use libquillcom::socket::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    prelude::*,
    threads::power::{find_session},
};

#[allow(dead_code)]
pub struct QinitThread {
    greetd_sender: Sender<MessageToGreetd>,
}

pub const QINIT_SOCKET_PATH: &'static str = "/run/qinit_rootfs.sock";

impl QinitThread {
    pub async fn init(greetd_sender: Sender<MessageToGreetd>) {
        tokio::spawn(async move {
            while !Path::new(QINIT_SOCKET_PATH).exists() {
                sleep(Duration::from_millis(200)).await;
            }
            let qinit = QinitThread {
                greetd_sender,
            };
            qinit.main_loop().await;
        });
    }

    async fn main_loop(self) {
        info!("Qinit main loop entered");
        loop {
            let Ok(answer) = read_write_socket::<CommandToQinit, AnswerFromQinit>(
                QINIT_SOCKET_PATH,
                CommandToQinit::GetLoginCredentials,
            )
            .await
            else {
                error!("Failed to get socket answer for credentials");
                continue;
            };

            match answer {
                AnswerFromQinit::Login(login_form) => match login_form {
                    Some(credentials) => {
                        debug!("Received credentials from qinit: {:?}", credentials);
                        self.greetd_sender
                            .send(MessageToGreetd::LogIn(
                                credentials.username,
                                credentials.password,
                            ))
                            .await
                            .unwrap();
                        info!("Sent login credentials to greetd");
                        break;
                    }
                    None => {
                        // info!("Login form was None")
                    }
                },
                _ => {
                    error!("Received unrelated answer to credentials");
                    continue;
                },
            }
            sleep(Duration::from_millis(300)).await;
        }
        info!("Qinit main_loop exits");
    }
}
