pub mod common;
pub mod consts;
pub mod prelude;
pub mod threads;

use crate::{prelude::*, threads::login::MessageToLogin};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    // The env is named RUST_LOG
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );
    info!("Qoms started");
    let splash_tx = PowerThread::init().await;
    let (message_to_greetd, _answer_from_greetd) = GreetdThread::init().await;

    let (login_tx, login_rx) = channel::<MessageToLogin>(LOW_COMM_BUFFER);

    LoginThread::init(message_to_greetd, login_rx).await;
    SocketThread::init(splash_tx, login_tx).await;

    // Testing
    // sleep(Duration::from_secs(3)).await;
    // message_to_greetd.send(MessageToGreetd::LogIn("root".to_string(), "rooD123".to_string())).await.unwrap();

    // Final waiter
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}
