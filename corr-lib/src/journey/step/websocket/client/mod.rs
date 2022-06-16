use futures_util::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use crate::core::runtime::Context;
use crate::journey::Executable;
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
        let url = self.url.evaluate(context).await.to_string();
        let mut conn=connect_async(Url::parse(url.as_str()).unwrap()).await;
        match conn {
            Ok((socket, _))=> {
                let (ssk,mut ssm) = socket.split();
                context.websocket_connection_store.define(self.connection_name.evaluate(context).await.to_string(),ssk).await;
                let hook = self.hook.clone();
                let new_ct = context.clone();
                let handle:JoinHandle<bool> = tokio::spawn(async move {
                    loop {
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
                });
                return vec![handle]
            },
            Err(e)=> {
                context.scrapper.ingest("errors",1.0,vec![(format!("message"),format!("{}",e.to_string())),(format!("api"),format!("{}",url))]).await;
                eprintln!("Error while connecting websocket {} - {}",url,e.to_string());
                return vec![]
            }
        }


    }
}
#[async_trait]
impl Executable for WebSocketSendStep{
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let conn_name = self.name.evaluate(context).await.to_string();
        if let Some(conn) = context.websocket_connection_store.get(conn_name.clone()).await{
            let mut connection = conn.lock().await;
            let msg = if self.is_binary {
                Message::Binary(self.message.evaluate(context).await.to_binary())
            } else {
                Message::Text(self.message.evaluate(context).await.to_string())
            };
            if let Err(e)=(*connection).send(msg).await{
                context.scrapper.ingest("errors",1.0,vec![(format!("message"),format!("{}",e.to_string())),(format!("connection"),format!("{}",conn_name.clone()))]).await;
                eprintln!("Error while sending data over websocket {} - {}",conn_name.clone(),e.to_string());
                context.exit(1).await;
            }
        } else {
            // let msg = format!("Websocket with name {} not found",conn_name.clone());
            // context.scrapper.ingest("errors",1.0,vec![(format!("message"),format!("{}",msg.clone())),(format!("connection"),format!("{}",conn_name.clone()))]).await;
            // eprintln!("{}",msg);
        }
        return vec![];
    }
}
