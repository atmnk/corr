use futures_util::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use crate::core::runtime::Context;
use crate::journey::Executable;
// use crate::journey::step::websocket::server::OnMessage;
use crate::template::{Expression, VariableReferenceName};
use async_trait::async_trait;
use url::Url;
use crate::core::Value;
use crate::journey::step::Step;
pub mod parser;
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketHook{
    variable:VariableReferenceName,
    block:Vec<Step>
}
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketClientConnectStep{
    url:Expression,
    connection_name: Expression,
    hook: WebSocketHook,
}
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketSendStep{
    name: Expression,
    message: Expression,
    is_binary:bool,
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
        let mut hook = self.hook.clone();
        let new_ct = context.clone();
        let handle:JoinHandle<bool> = tokio::spawn(async move {
            while true {
                if let Some(Ok(m)) = ssm.next().await{
                    if m.is_text(){
                        let sv = serde_json::from_str(&m.to_string()).unwrap_or(serde_json::Value::String(m.to_string()));
                        new_ct.define(hook.variable.to_string(),Value::from_json_value(sv)).await;
                        let mut handles = vec![];
                        for step in &hook.block {
                            let mut inner_handles = step.execute(&new_ct).await;
                            handles.append(&mut inner_handles);
                        }
                        futures::future::join_all(handles).await;
                    }
                } else {
                    return true;
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
        let msg = if self.is_binary {
            Message::Binary(self.message.evaluate(context).await.to_binary())
        } else {
            Message::Text(self.message.evaluate(context).await.to_string())
        };
        (*connection).send(msg).await;
        return vec![];
    }
}
