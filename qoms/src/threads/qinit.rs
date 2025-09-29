use libquillcom::socket::CommandToQinit;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::prelude::*;

pub enum MessageToQinitThread {}

pub enum AnswerFromQinitThread {}

pub struct QinitThread {
    m_rx: Receiver<MessageToQinitThread>,
    a_tx: Sender<AnswerFromQinitThread>,
    greetd_sender: Sender<MessageToGreetd>,
}

const QINIT_SOCKET_PATH: &'static str = "/run/qinit_rootfs.sock";

impl QinitThread {
    pub async fn init(
        greetd_sender: Sender<MessageToGreetd>,
    ) -> (
        Sender<MessageToQinitThread>,
        Receiver<AnswerFromQinitThread>,
    ) {
        let (m_tx, m_rx) = mpsc::channel::<MessageToQinitThread>(LOW_COMM_BUFFER);
        let (a_tx, a_rx) = mpsc::channel::<AnswerFromQinitThread>(LOW_COMM_BUFFER);
        let qinit = QinitThread::new(m_rx, a_tx, greetd_sender).await;
        tokio::spawn(async move {
            qinit.main_loop().await;
        });
        (m_tx, a_rx)
    }

    async fn new(
        m_rx: Receiver<MessageToQinitThread>,
        a_tx: Sender<AnswerFromQinitThread>,
        greetd_sender: Sender<MessageToGreetd>,
    ) -> Self {
        while !Path::new(QINIT_SOCKET_PATH).exists() {
            sleep(Duration::from_millis(200)).await;
        }
        QinitThread {
            m_rx,
            a_tx,
            greetd_sender,
        }
    }

async fn main_loop(self) {
    info!("Qinit main loop entered");
    loop {
        info!("Qinit main loop loop");
        let mut stream = UnixStream::connect(QINIT_SOCKET_PATH).await.unwrap();
        info!("Connected to QINIT socket");
        stream
            .write_all(
                &postcard::to_allocvec::<CommandToQinit>(&CommandToQinit::GetLoginCredentials)
                    .unwrap(),
            )
            .await
            .unwrap();
        stream.shutdown().await.unwrap();
        info!("Sent GetLoginCredentials command and shut down write");

        let mut buf = [0u8; 512];
        let mut message_bytes = Vec::new();
        let mut attempts = 0;

        loop {
            let n = stream.read(&mut buf).await.unwrap();
            info!("Read attempt {} from QINIT", attempts + 1);
            if n > 0 {
                message_bytes.extend_from_slice(&buf[..n]);
                info!("Received data from QINIT");
                break;
            } else {
                attempts += 1;
                if attempts >= 5 {
                    info!("Max read attempts reached");
                    break;
                }
                sleep(Duration::from_millis(50)).await;
                info!("Retrying read");
            }
        }

        if message_bytes.is_empty() {
            info!("No message received, restarting loop");
            continue;
        } else {
            info!("Received data: {:?}", message_bytes);
        }

        match postcard::from_bytes::<libquillcom::socket::AnswerFromQinit>(&message_bytes).unwrap() {
            libquillcom::socket::AnswerFromQinit::Login(login_form) => match login_form {
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
                None => info!("Login form was None"),
            },
        }
        sleep(Duration::from_millis(200)).await;
        info!("Loop iteration complete");
    }
}
}
