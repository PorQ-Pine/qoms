use pinenote_service::{
    drivers::rockchip_ebc::RockchipEbc,
    pixel_manager::ComputedHints,
    types::rockchip_ebc::{DitherMode, DriverMode, Hint, HintBitDepth, HintConvertMode, Mode},
};

use crate::prelude::*;

#[allow(dead_code)]
pub struct PowerThread {
    request_rx: Receiver<Splash>,
}

impl PowerThread {
    pub async fn init() -> Sender<Splash> {
        let (splash_tx, splash_rx) = channel::<Splash>(LOW_COMM_BUFFER);
        tokio::spawn(async move {
            while !Path::new(QOMS_SOCKET_PATH).exists() {
                sleep(Duration::from_millis(200)).await;
            }
            let power = PowerThread {
                request_rx: splash_rx,
            };
            power.main_loop().await;
        });
        splash_tx
    }

    async fn main_loop(mut self) {
        info!("Power main loop entered");
        loop {
            if let Some(mess) = self.request_rx.recv().await {
                if mess == Splash::Sleep {
                    warn!("Sleep is not supported");
                    continue;
                }

                if logout_session().await.is_err() {
                    error!("Failed to logout, we continue anyway in power");
                }

                let real_splash = match mess {
                    Splash::PowerOff => PrimitiveShutDownType::PowerOff,
                    Splash::Reboot => PrimitiveShutDownType::Reboot,
                    Splash::Sleep => PrimitiveShutDownType::Sleep, // Not possible
                };

                // Request splash screen
                let answer = timeout(
                    Duration::from_secs(20),
                    read_write_socket::<CommandToQinit, AnswerFromQinit>(
                        QINIT_SOCKET_PATH,
                        CommandToQinit::TriggerSplash(real_splash),
                    ),
                )
                .await;
                match answer {
                    Ok(answer2) => match answer2 {
                        Ok(answer3) => {
                            if answer3 == AnswerFromQinit::SplashReady {
                                info!("Splash shown correctly");
                            } else {
                                error!("Received wrong answer in splash request?, {:?}", answer3);
                            }
                        }
                        Err(err) => {
                            error!("Failed to recv answer to request splash screen: {:?}", err)
                        }
                    },
                    Err(_) => error!("Requesting splash screen timed out"),
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
                if vec[2].clone() == "greetd" {
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

pub async fn logout_session() -> Result<(), ()> {
    let Some((session, user)) = find_session().await else {
        error!("Failed to get session");
        return Err(());
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

    // Change to y4
    let ebc = RockchipEbc::new();
    match ebc.set_mode(Mode {
        driver_mode: Some(DriverMode::Normal),
        dither_mode: Some(DitherMode::Bayer),
        redraw_delay: None,
    }) {
        Ok(_) => info!("Set succesfully eink normal mode"),
        Err(err) => error!("Failed to set eink mode: {:?}", err),
    }
    let mut hints = ComputedHints::new();
    hints.default_hint = Some(Hint::new(
        HintBitDepth::Y4,
        HintConvertMode::Threshold,
        false,
    ));
    match ebc.upload_rect_hints(hints) {
        Ok(_) => info!("Succesfully set Y4 mode"),
        Err(err) => error!("Failed to set Y4: {:?}", err),
    }

    // We wait for tty & niri & greetd to shut up
    // Idk if there is a better way.
    sleep(Duration::from_secs(14)).await;

    Ok(())
}
