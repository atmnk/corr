pub mod parser;


use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use crate::core::runtime::Context;
use crate::journey::{Executable};
use crate::journey::step::Step;
use crate::template::{Expression, VariableReferenceName};
use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async};
use tokio_tungstenite::tungstenite::{Message};
use crate::core::Value;


#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketServerHook{
    variable:VariableReferenceName,
    block:Vec<Step>
}
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketServerStep {
    port:Expression,
    hook: WebSocketServerHook
}
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketServerSendToClient {
    id: Expression,
    message: Expression,
    is_binary:bool,
}
#[async_trait]
impl Executable for WebSocketServerSendToClient {

    async fn execute(&self,context: &Context)->Result<Vec<JoinHandle<Result<bool>>>> {
        let conn_name = self.id.evaluate(context).await?.to_string();

        if let Some(conn) = context.websocket_clients.get(conn_name.clone()).await{
            let mut connection = conn.lock().await;
            let msg = if self.is_binary {
                Message::Binary(self.message.evaluate(context).await?.to_binary())
            } else {
                Message::Text(self.message.evaluate(context).await?.to_string())
            };
            if let Err(e)=(*connection).send(msg).await{
                context.scrapper.ingest("errors",1.0,vec![(format!("message"),format!("{}",e.to_string())),(format!("connection"),format!("{}",conn_name.clone()))]).await;
                eprintln!("Error while sending data over websocket {} - {}",conn_name.clone(),e.to_string());
            }
        } else {
            let msg = format!("Websocket with name {} not found",conn_name.clone());
            context.scrapper.ingest("errors",1.0,vec![(format!("message"),format!("{}",msg.clone())),(format!("connection"),format!("{}",conn_name.clone()))]).await;
            eprintln!("{}",msg);
        }
        return Ok(vec![]);
    }

    fn get_deps(&self) -> Vec<String> {
        vec![]
    }
}
#[async_trait]
impl Executable for WebSocketServerStep {

    async fn execute(&self,context: &Context)->Result<Vec<JoinHandle<Result<bool>>>> {
        let addr = format!("0.0.0.0:{}",self.port.evaluate(context).await?.to_string());
        let listener = TcpListener::bind(&addr).await.expect("Can't listen");
        let new_ct_out = context.clone();
        let om_out=self.hook.clone();

        let connect = async move|listener:TcpListener, om_out:WebSocketServerHook, new_ct_out:Context|{
            let accept_connection = async  move |peer: SocketAddr, stream: TcpStream,ctx_para:Context,hook:WebSocketServerHook|{
                let ctx = ctx_para.clone();
                let  handle_connection= async move |peer: SocketAddr, stream: TcpStream| -> Result<()> {
                    let ctx = Context::from_without_fallback(&ctx).await;
                    let ws_stream = accept_async(stream).await.expect("Failed to accept");
                    let (tx,mut rx) = ws_stream.split();
                    let conn_id = uuid::Uuid::new_v4().to_string();
                    ctx.websocket_clients.define(conn_id.clone(), tx).await;
                    ctx.define("connectionId".to_string(),Value::String(conn_id)).await;
                    println!("New WebSocket connection: {}", peer);
                    while let Some(Ok(m)) = rx.next().await {
                        if m.is_text() {
                            let sv = serde_json::from_str(&m.to_string()).unwrap_or(serde_json::Value::String(m.to_string()));
                            ctx.define(hook.variable.to_string(),Value::from_json_value(sv)).await;
                            let mut handles = vec![];
                            for step in &hook.block {
                                let mut inner_handles = step.execute(&ctx).await?;
                                handles.append(&mut inner_handles);
                            }
                            futures::future::join_all(handles).await;
                        }
                    }

                    Ok(())
                };
                if let Err(e) = handle_connection(peer, stream).await {
                    println!("Error processing connection: {}", e)
                }
            };
            while let Ok((stream,_)) = listener.accept().await{
                let om=om_out.clone();
                let new_ct = new_ct_out.clone();
                let peer = stream.peer_addr().expect("connected streams should have a peer address");
                tokio::spawn(accept_connection(peer,stream,new_ct,om));
            };
            Ok(true)
        };
        let handle = tokio::spawn(connect(listener, om_out, new_ct_out));
        Ok(vec![handle])
    }

    fn get_deps(&self) -> Vec<String> {
        let mut deps = vec![];
        for step in &self.hook.block {
            deps.append(&mut step.get_deps());
        }
        deps
    }
}