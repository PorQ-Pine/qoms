use crate::prelude::*;

#[allow(dead_code)]
pub struct SocketThread {
    splash_tx: Sender<Splash>,
}

impl SocketThread {
    pub async fn init(splash_tx: Sender<Splash>) {
        tokio::spawn(async move {
            let power = SocketThread { splash_tx };
            power.main_loop().await;
        });
    }

    async fn main_loop(self) {
        info!("Socket main loop entered");

        async fn open_listener() -> UnixListener {
            let path = std::path::Path::new(&QOMS_SOCKET_PATH);
            if path.exists() {
                let _ = fs::remove_file(&path).await;
            }

            loop {
                match UnixListener::bind(&path) {
                    Ok(listener) => {
                        let _ = fs::set_permissions(&path, std::fs::Permissions::from_mode(0o777))
                            .await;
                        info!("Successfully created: {:?}", path);
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        return listener;
                    }
                    Err(_) => {
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
                            }
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
