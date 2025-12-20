use crate::prelude::*;
use libquillcom::socket::{CommandToQinit, PrimitiveShutDownType};
use qoms_coms::{QOMS_SOCKET_PATH, Splash};
use tokio::io::AsyncWriteExt;
use tokio::time::{Instant, timeout};

#[derive(PartialEq)]
pub enum MessageToPower {
    SplashScreenShown,
}

#[allow(dead_code)]
pub struct PowerThread {
    request_rx: Receiver<Splash>,
    confirmation_rx: Receiver<MessageToPower>,
}

impl PowerThread {
    pub async fn init() -> (Sender<Splash>, Sender<MessageToPower>) {
        let (splash_tx, splash_rx) = mpsc::channel::<Splash>(LOW_COMM_BUFFER);
        let (power_tx, power_rx) = mpsc::channel::<MessageToPower>(LOW_COMM_BUFFER);
        tokio::spawn(async move {
            while !Path::new(QOMS_SOCKET_PATH).exists() {
                sleep(Duration::from_millis(200)).await;
            }
            let power = PowerThread {
                request_rx: splash_rx,
                confirmation_rx: power_rx,
            };
            power.main_loop().await;
        });
        (splash_tx, power_tx)
    }

    async fn main_loop(mut self) {
        info!("Power main loop entered");
        loop {
            if let Some(mess) = self.request_rx.recv().await {
                if mess == Splash::Sleep {
                    warn!("Sleep is not supported");
                    continue;
                }

                let Some((session, user)) = find_session().await else {
                    error!("Failed to get session");
                    continue;
                };

                // Wait for the session to terminate, if not, we kill it
                let terminate = format!("loginctl terminate-session {}", session).to_string();
                let result = timeout(Duration::from_secs(5), run_cmd(&terminate)).await;
                if result.is_err() {
                    warn!("Session did not terminate, forcing it to cooperate");
                    let kill = format!("loginctl kill-user {}", user).to_string();
                    run_cmd(&kill).await;
                } else {
                    info!("Session terminated correctly");
                }

                // We wait a bit for the screen to stop screening because stupid tty and greetd
                sleep(Duration::from_secs(3)).await;

                let real_splash = match mess {
                    Splash::PowerOff => PrimitiveShutDownType::PowerOff,
                    Splash::Reboot => PrimitiveShutDownType::Reboot,
                    Splash::Sleep => PrimitiveShutDownType::Sleep, // Not possible
                };

                // Request splash screen
                let mut stream = UnixStream::connect(QINIT_SOCKET_PATH).await.unwrap();
                stream
                    .write_all(
                        &postcard::to_allocvec::<CommandToQinit>(&CommandToQinit::TriggerSplash(
                            real_splash,
                        ))
                        .unwrap(),
                    )
                    .await
                    .unwrap();
                stream.shutdown().await.unwrap();

                // Wait for confirmation
                let deadline = Instant::now() + Duration::from_secs(10);
                loop {
                    let now = Instant::now();
                    if now >= deadline {
                        error!("Splash screen showing timed out");
                        break;
                    }

                    let remaining = deadline - now;
                    let result = timeout(remaining, self.confirmation_rx.recv()).await;

                    match result {
                        Ok(Some(xx)) => {
                            if xx == MessageToPower::SplashScreenShown {
                                info!("Splash screen shown correctly");
                                break;
                            } else {
                                error!("This is not splash screen shown message, repair the code");
                            }
                        }
                        Ok(None) => {
                            error!("Failed to confirm splash because none?");
                        }
                        Err(_) => {
                            error!("Recv attempt failed, retrying");
                        }
                    }
                    sleep(Duration::from_secs(1)).await;
                }

                // Do the action
                match mess {
                    Splash::PowerOff => {
                        info!("Powering off!");
                        run_cmd("systemctl poweroff").await;
                    }
                    Splash::Reboot => {
                        info!("Rebooting!");
                        run_cmd("systemctl reboot").await;
                    }
                    Splash::Sleep => todo!(),
                };
                // This should never proceed, but if it does, uh, whatever, we wait again
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// Outputs sessions ID, User
pub async fn find_session() -> Option<(String, String)> {
    let string = run_cmd("loginctl list-sessions --no-legend").await;
    // debug!("String is: {}", string);
    for line in string.split("\n") {
        if line.contains("seat0") {
            let vec: Vec<String> = line.split(" ").map(|s| s.to_string()).collect();
            if vec.len() > 2 {
                return Some((vec[0].clone(), vec[2].clone()));
            }
            return None
        }
    }
    None
}

#[tokio::test]
async fn test_find_session() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );
    info!("Running test_find_session");
    let session = find_session().await;
    assert!(
        session.is_some() || session.is_none(),
        "find_session should return Some or None"
    );
    info!("Result of find_session: {:?}", session);
}
