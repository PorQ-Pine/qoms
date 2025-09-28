pub mod consts;
pub mod prelude;
pub mod threads;

use crate::{prelude::*, threads::greetd::GreetdThread};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let (message_to_greetd, answer_from_greetd) = GreetdThread::init().await;

    // Final waiter
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}
