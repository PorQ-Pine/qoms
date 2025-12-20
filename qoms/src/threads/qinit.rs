use libquillcom::socket::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{prelude::*, threads::power::MessageToPower};

#[allow(dead_code)]
pub struct QinitThread {
    greetd_sender: Sender<MessageToGreetd>,
    power_tx: Sender<MessageToPower>,
}

const QINIT_SOCKET_PATH: &'static str = "/run/qinit_rootfs.sock";

impl QinitThread {
    pub async fn init(greetd_sender: Sender<MessageToGreetd>, power_tx: Sender<MessageToPower>) {
        tokio::spawn(async move {
            while !Path::new(QINIT_SOCKET_PATH).exists() {
                sleep(Duration::from_millis(200)).await;
            }
            let qinit = QinitThread {
                greetd_sender,
                power_tx,
            };
            qinit.main_loop().await;
        });
    }

    async fn main_loop(self) {
        info!("Qinit main loop entered");
        loop {
            // info!("Qinit main loop loop");
            let mut stream = UnixStream::connect(QINIT_SOCKET_PATH).await.unwrap();
            // info!("Connected to QINIT socket");
            stream
                .write_all(
                    &postcard::to_allocvec::<CommandToQinit>(&CommandToQinit::GetLoginCredentials)
                        .unwrap(),
                )
                .await
                .unwrap();
            stream.shutdown().await.unwrap();
            // info!("Sent GetLoginCredentials command and shut down write");

            let mut buf = [0u8; 512];
            let mut message_bytes = Vec::new();
            let mut attempts = 0;

            loop {
                let n = stream.read(&mut buf).await.unwrap();
                // info!("Read attempt {} from QINIT", attempts + 1);
                if n > 0 {
                    message_bytes.extend_from_slice(&buf[..n]);
                    // info!("Received data from QINIT");
                    break;
                } else {
                    attempts += 1;
                    if attempts >= 5 {
                        // info!("Max read attempts reached");
                        break;
                    }
                    sleep(Duration::from_millis(50)).await;
                    // info!("Retrying read");
                }
            }

            if message_bytes.is_empty() {
                // info!("No message received, restarting loop");
                continue;
            } else {
                // info!("Received data: {:?}", message_bytes);
            }

            match postcard::from_bytes::<AnswerFromQinit>(&message_bytes).unwrap() {
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
                        //info!("Login form was None")
                    }
                },
                AnswerFromQinit::SplashReady => {
                    if let Err(err) = self.power_tx.send(MessageToPower::SplashScreenShown).await {
                        error!("Failed to send to power: {:?}", err);
                    }
                }
            }
            sleep(Duration::from_millis(200)).await;
            // info!("Qinit Loop");
        }
        info!("Qinit main_loop exits");
    }
}
