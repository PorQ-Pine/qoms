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
        let mut stream = UnixStream::connect(QINIT_SOCKET_PATH).await.unwrap();
        loop {
            info!("Qinit main loop loop");
            stream
                .write_all(
                    &postcard::to_allocvec::<CommandToQinit>(&CommandToQinit::GetLoginCredentials)
                        .unwrap(),
                )
                .await
                .unwrap();
            let mut message_bytes = Vec::new();
            stream.read_to_end(&mut message_bytes).await.unwrap();

            match postcard::from_bytes::<libquillcom::socket::AnswerFromQinit>(&message_bytes)
                .unwrap()
            {
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
                        break;
                    }
                    None => (),
                },
            }
            sleep(Duration::from_millis(100)).await
        }
    }
}
