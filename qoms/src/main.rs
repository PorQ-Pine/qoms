pub mod consts;
pub mod prelude;
pub mod threads;

use crate::{prelude::*, threads::greetd::GreetdThread};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    // The env is named RUST_LOG
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    info!("Qoms welcomes");

    let (message_to_greetd, answer_from_greetd) = GreetdThread::init().await;

    // Final waiter
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}
