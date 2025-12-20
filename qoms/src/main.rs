pub mod consts;
pub mod prelude;
pub mod threads;
pub mod common;

use crate::{prelude::*, threads::{power::PowerThread, qoms_socket::SocketThread}};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    // The env is named RUST_LOG
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );
    info!("Qoms welcomes");
    let (splash_tx, power_tx) = PowerThread::init().await;
    SocketThread::init(splash_tx).await;
    let (message_to_greetd, _answer_from_greetd) = GreetdThread::init().await;
    QinitThread::init(message_to_greetd, power_tx).await;

    // Testing
    // sleep(Duration::from_secs(3)).await;
    // message_to_greetd.send(MessageToGreetd::LogIn("root".to_string(), "rooD123".to_string())).await.unwrap();

    // Final waiter
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}
