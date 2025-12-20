use crate::prelude::*;
use anyhow::Result;
use qoms_coms::{QOMS_SOCKET_PATH, SendToQoms, Splash};
use tokio::{
    io::{AsyncReadExt, BufReader},
    net::UnixListener,
};

#[allow(dead_code)]
pub struct SocketThread {
    splash_tx: Sender<Splash>,
}

impl SocketThread {
    pub async fn init(splash_tx: Sender<Splash>) {
        tokio::spawn(async move {
            while !Path::new(QOMS_SOCKET_PATH).exists() {
                sleep(Duration::from_millis(200)).await;
            }
            let power = SocketThread { splash_tx };
            power.main_loop().await;
        });
    }

    async fn main_loop(self) {
        info!("Socket main loop entered");

        async fn open_listener() -> UnixListener {
            let path = std::path::Path::new(&QOMS_SOCKET_PATH);
            loop {
                match UnixListener::bind(&path) {
                    Ok(stream) => {
                        info!("Successfully connected to socket: {:?}", path);
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        return stream;
                    }
                    Err(_e) => {
                        // debug!("Waiting for socket at {}: {}", socket_path, e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
                    }
                }
            }
        }

        async fn handle_client(stream: UnixStream) -> Result<SendToQoms> {
            let mut reader = BufReader::new(stream);
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).await?;

            let mess: SendToQoms = postcard::from_bytes(&mut buf)?;
            Ok(mess)
        }

        let listener = open_listener().await;
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    debug!("New client connected to request socket");
                    if let Ok(mess) = handle_client(stream).await {
                        match mess {
                            SendToQoms::RequestSplash(splash) => {
                                if let Err(err) = self.splash_tx.send(splash).await {
                                    error!("Failed to send splash: {:?}", err);
                                }
                            },
                        }
                    } else {
                        error!("Failed to handle client for qoms socket");
                    }
                }
                Err(e) => {
                    error!("Failed to accept request client: {}", e);
                }
            }
        }
    }
}
