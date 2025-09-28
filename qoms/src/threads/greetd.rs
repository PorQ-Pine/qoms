use crate::prelude::*;

pub enum MessageToGreetd {}

pub enum AnswerFromGreetd {}

pub struct GreetdThread {
    m_rx: Receiver<MessageToGreetd>,
    a_tx: Sender<AnswerFromGreetd>,
}

impl GreetdThread {
    pub async fn init() -> (Sender<MessageToGreetd>, Receiver<AnswerFromGreetd>) {
        let (m_tx, m_rx) = mpsc::channel::<MessageToGreetd>(LOW_COMM_BUFFER);
        let (a_tx, a_rx) = mpsc::channel::<AnswerFromGreetd>(LOW_COMM_BUFFER);
        let greetd = GreetdThread::new(m_rx, a_tx).await;
        tokio::spawn(async move {
            greetd.main_loop().await;
        });
        (m_tx, a_rx)
    }

    async fn new(m_rx: Receiver<MessageToGreetd>, a_tx: Sender<AnswerFromGreetd>) -> Self {
        
        GreetdThread {m_rx, a_tx}
    }

    async fn main_loop(mut self) {
        loop {
            match self.m_rx.recv().await {
                Some(_) => todo!(),
                None => (),
            }
        }
    }
}
