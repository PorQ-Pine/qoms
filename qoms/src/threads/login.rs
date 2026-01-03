use crate::{
    prelude::*,
    threads::power::{find_session, logout_session},
};

pub enum MessageToLogin {
    ReLogin,
}

#[allow(dead_code)]
pub struct LoginThread {
    greetd_sender: Sender<MessageToGreetd>,
    relogin_receiver: Receiver<MessageToLogin>,
}

pub const QINIT_SOCKET_PATH: &'static str = "/run/qinit_rootfs.sock";

impl LoginThread {
    pub async fn init(
        greetd_sender: Sender<MessageToGreetd>,
        relogin_receiver: Receiver<MessageToLogin>,
    ) {
        tokio::spawn(async move {
            while !Path::new(QINIT_SOCKET_PATH).exists() {
                sleep(Duration::from_millis(200)).await;
            }
            let login = LoginThread {
                greetd_sender,
                relogin_receiver,
            };
            login.main_loop().await;
        });
    }

    async fn main_loop(mut self) {
        info!("Login main loop entered");
        // Initial login
        // Means we are logged in already. For deploy to work
        if find_session().await.is_none() {
            self.login_loop().await;
        }

        loop {
            match self.relogin_receiver.recv().await {
                Some(message) => match message {
                    MessageToLogin::ReLogin => {
                        info!("Relogin starting");

                        if logout_session().await.is_err() {
                            error!("Failed to logout session, we continue anyway?");
                        }

                        // Request the login page
                        let answer = timeout(
                            Duration::from_secs(3),
                            read_write_socket::<CommandToQinit, AnswerFromQinit>(
                                QINIT_SOCKET_PATH,
                                CommandToQinit::TriggerSwitchToLoginPage,
                            ),
                        )
                        .await;
                        match answer {
                            Ok(answer2) => match answer2 {
                                Ok(answer3) => {
                                    if answer3 == AnswerFromQinit::LoginPageReady {
                                        info!("Login page shown correctly");
                                    } else {
                                        error!(
                                            "Received wrong answer in login request?, {:?}",
                                            answer3
                                        );
                                    }
                                }
                                Err(err) => {
                                    error!(
                                        "Failed to recv answer to request login screen: {:?}",
                                        err
                                    )
                                }
                            },
                            Err(_) => error!("Requesting login screen timed out"),
                        }

                        sleep(Duration::from_millis(2000)).await;

                        // Actually log in
                        self.login_loop().await;
                    }
                },
                None => error!("Recv relogin none"),
            }
            sleep(Duration::from_millis(300)).await;
        }
    }

    async fn login_loop(&mut self) {
        info!("Real login loop entered");
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
                }
            }
            sleep(Duration::from_millis(300)).await;
        }
    }
}
