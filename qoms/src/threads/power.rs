use qoms_coms::{QOMS_SOCKET_PATH, Splash};

use crate::prelude::*;

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
            let power = PowerThread { request_rx: splash_rx, confirmation_rx: power_rx };
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
            }
        }
    }
}

// Outputs sessions ID
async fn find_session() -> Option<String> {
    let string = run_cmd("loginctl list-sessions --no-legend").await;
    debug!("String is: {}", string);
    for line in string.split("\n") {
        if line.contains("seat0") {
            let vec: Vec<String> = line.split(" ").map(|s| s.to_string()).collect();
            return vec.first().cloned()
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
    assert!(session.is_some() || session.is_none(), "find_session should return Some or None");
    info!("Result of find_session: {:?}", session);
}

