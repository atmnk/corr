use futures_util::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use crate::core::runtime::Context;
use crate::journey::Executable;
use crate::journey::step::websocket::server::OnMessage;
use crate::template::Expression;
use async_trait::async_trait;
use url::Url;
pub mod parser;
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketClientConnectStep{
    url:Expression,
    connection_name: Expression
}
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketSendStep{
    name: Expression,
    message: Expression,
}
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketSendBinaryStep{
    name: Expression
}
#[async_trait]
impl Executable for WebSocketClientConnectStep {
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let (mut socket, _)=connect_async(Url::parse(self.url.evaluate(context).await.to_string().as_str()).unwrap()).await.unwrap();
        let (mut ssk,mut ssm) = socket.split();
        context.websocket_connection_store.define(self.connection_name.evaluate(context).await.to_string(),ssk).await;
        let handle:JoinHandle<bool> = tokio::spawn(async move {
            while true {
                let m = ssm.next().await.unwrap();
                match m {
                    Result::Ok(ms)=>{
                        println!("{:?}",ms)
                    },
                    Result::Err(err)=>{
                        return true;
                    }
                }
            }
            return true;
        });
        return vec![handle]

    }
}
#[async_trait]
impl Executable for WebSocketSendStep{
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let conn = context.websocket_connection_store.get(self.name.evaluate(context).await.to_string()).await.unwrap();
        let mut connection = conn.lock().await;
        (*connection).send(Message::Text(self.message.evaluate(context).await.to_string())).await;
        return vec![];
    }
}
// #[async_trait]
// impl Executable for WebSocketSendBinaryStep{
//     async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
//         let conn = context.websocket_connection_store.get(self.name.evaluate(context).await.to_string()).await.unwrap();
//         let mut connection = conn.lock().await;
//         (*connection).send(Message::Binary()).await;
//         return vec![];
//     }
// }