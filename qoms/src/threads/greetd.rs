use crate::prelude::*;

pub enum MessageToGreetd {
    LogIn(String, String), // Provide the username, password
}

pub enum AnswerFromGreetd {
    LoginStatus(bool),
}

pub struct GreetdThread {
    greetd_socket_path: String,
    m_rx: Receiver<MessageToGreetd>,
    a_tx: Sender<AnswerFromGreetd>,
}

const GREETD_SOCKET_READ_PATH: &'static str = "/tmp/qoms/greetd_sock_path.txt";
const GREETD_BUGGER: usize = 10;

impl GreetdThread {
    pub async fn init() -> (Sender<MessageToGreetd>, Receiver<AnswerFromGreetd>) {
        let (m_tx, m_rx) = channel::<MessageToGreetd>(GREETD_BUGGER);
        let (a_tx, a_rx) = channel::<AnswerFromGreetd>(GREETD_BUGGER);
        tokio::spawn(async move {
            let greetd = GreetdThread::new(m_rx, a_tx).await;
            greetd.main_loop().await;
        });
        (m_tx, a_rx)
    }

    async fn new(m_rx: Receiver<MessageToGreetd>, a_tx: Sender<AnswerFromGreetd>) -> Self {
        while !Path::new(GREETD_SOCKET_READ_PATH).exists() {
            sleep(Duration::from_millis(200)).await;
        }
        let greetd_socket_path = fs::read_to_string(GREETD_SOCKET_READ_PATH)
            .await
            .unwrap()
            .trim()
            .to_string();
        info!("greetd_socket_path: {}", greetd_socket_path);
        GreetdThread {
            m_rx,
            a_tx,
            greetd_socket_path,
        }
    }

    async fn main_loop(mut self) {
        info!("Greetd main loop entered");
        loop {
            match self.m_rx.recv().await {
                Some(message) => match message {
                    MessageToGreetd::LogIn(username, password) => {
                        info!("Received LogIn message for user: {}", username);
                        match login(username, password, &self.greetd_socket_path).await {
                            Ok(status) => match status {
                                true => {
                                    info!("Login returned true");
                                    self.a_tx
                                        .send(AnswerFromGreetd::LoginStatus(true))
                                        .await
                                        .ok();
                                }
                                false => {
                                    info!("Login returned false");
                                    error!("Failed to log in, but regular bool");
                                    self.a_tx
                                        .send(AnswerFromGreetd::LoginStatus(false))
                                        .await
                                        .ok();
                                }
                            },
                            Err(err) => {
                                error!("Failed to log in: {:?}", err);
                                self.a_tx
                                    .send(AnswerFromGreetd::LoginStatus(false))
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                },
                None => {
                    // error!("Channel closed, bad");
                    sleep(Duration::from_millis(5000)).await;
                }
            }
            // info!("Greetd Thread main_loop loop");
        }
    }
}

async fn login(
    username: String,
    password: String,
    greetd_sock: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = UnixStream::connect(greetd_sock).await?;

    let mut next_request = Request::CreateSession { username };
    let mut starting = false;
    loop {
        next_request.write_to(&mut stream).await?;

        match Response::read_from(&mut stream).await? {
            Response::AuthMessage {
                auth_message,
                auth_message_type,
            } => {
                debug!(
                    "Received auth message: {:?} and {:?}",
                    auth_message, auth_message_type
                );
                let response = match auth_message_type {
                    AuthMessageType::Visible => Some(password.clone()),
                    AuthMessageType::Secret => Some(password.clone()),
                    AuthMessageType::Info => {
                        info!("info: {auth_message}");
                        None
                    }
                    AuthMessageType::Error => {
                        info!("error: {auth_message}");
                        None
                    }
                };

                next_request = Request::PostAuthMessageResponse { response };
            }
            Response::Success => {
                if starting {
                    return Ok(true);
                } else {
                    starting = true;
                    let command = "niri --session";
                    next_request = Request::StartSession {
                        env: vec![],
                        cmd: vec![command.to_string()],
                    }
                }
            }
            Response::Error {
                error_type,
                description,
            } => {
                Request::CancelSession.write_to(&mut stream).await?;
                match error_type {
                    ErrorType::AuthError => return Ok(false),
                    ErrorType::Error => return Err(format!("login error: {description:?}").into()),
                }
            }
        }
        info!("Greetd loop");
        sleep(Duration::from_millis(250)).await;
    }
}
