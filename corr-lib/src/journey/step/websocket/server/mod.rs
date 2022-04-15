pub mod parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use crate::core::runtime::Context;
use crate::journey::Executable;
use crate::journey::step::Step;
use crate::template::{Expression, VariableReferenceName};

use async_trait::async_trait;
use std::net::SocketAddr;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tokio_tungstenite::tungstenite::{Message, Result};
use crate::core::Value;

#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketServerHook{
    variable:VariableReferenceName,
    block:Vec<WebSocketStep>
}
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketServerStep {
    port:Expression,
    hook: WebSocketServerHook,
    // on_message:OnMessage
}
// #[derive(Debug, Clone,PartialEq)]
// pub struct OnMessage {
//     pub extract:ExtractableObject,
//     pub steps:Vec<WebSocketStep>,
// }
#[derive(Debug, Clone,PartialEq)]
pub enum WebSocketStep{
    SendStep(Expression),
    NormalStep(Step)
}
#[async_trait]
impl Executable for WebSocketServerStep {
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let addr = format!("0.0.0.0:{}",self.port.evaluate(context).await.to_string());
        let listner = TcpListener::bind(&addr).await.expect("Can't listen");
        let accept_connection = async  move |peer: SocketAddr, stream: TcpStream,ctx_para:Context,hook:WebSocketServerHook|{
            let ctx = ctx_para.clone();
            let  handle_connection= async move |peer: SocketAddr, stream: TcpStream| -> Result<()> {
                let mut ws_stream = accept_async(stream).await.expect("Failed to accept");
                println!("New WebSocket connection: {}", peer);
                while let Some(Ok(m)) = ws_stream.next().await {
                    if m.is_text() {
                        let sv = serde_json::from_str(&m.to_string()).unwrap_or(serde_json::Value::String(m.to_string()));
                        ctx.define(hook.variable.to_string(),Value::from_json_value(sv)).await;
                        let mut handles = vec![];
                        for step in &hook.block {
                            match step {
                                WebSocketStep::SendStep(snd)=>{
                                    ws_stream.send(Message::Text(format!("{}",snd.evaluate(&ctx).await.to_string()) )).await?;
                                },
                                WebSocketStep::NormalStep(stp)=>{
                                    let mut inner_handles = stp.execute(&ctx).await;
                                    handles.append(&mut inner_handles);
                                }
                            }
                        }
                        futures::future::join_all(handles).await;
                    }
                }

                Ok(())
            };
            if let Err(e) = handle_connection(peer, stream).await {
                match e {
                    Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                    err => println!("Error processing connection: {}", err),
                }
            }
        };
        let new_ct_out = context.clone();
        let om_out=self.hook.clone();

        let connect = async move||{
            while let Ok((stream,_)) = listner.accept().await{
                let om=om_out.clone();
                let new_ct = new_ct_out.clone();
                let peer = stream.peer_addr().expect("connected streams should have a peer address");
                tokio::spawn(accept_connection(peer,stream,new_ct,om));
            };
            true
        };
        let handle = tokio::spawn(connect());
        vec![handle]
    }
}