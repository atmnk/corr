pub mod parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use crate::core::runtime::Context;
use crate::journey::Executable;
use crate::journey::step::Step;
use crate::template::Expression;
use crate::template::object::extractable::{Extractable, ExtractableObject};
use async_trait::async_trait;
use std::net::SocketAddr;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tokio_tungstenite::tungstenite::{Message, Result};
use crate::core::Value;

#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketServerStep {
    port:Expression,
    on_message:OnMessage
}
#[derive(Debug, Clone,PartialEq)]
pub struct OnMessage {
    pub extract:ExtractableObject,
    pub steps:Vec<WebSocketStep>,
}
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
        let accept_connection = async  move |peer: SocketAddr, stream: TcpStream,ctx_para:Context,om:OnMessage|{
            let ctx = ctx_para.clone();
            let  handle_connection= async move |peer: SocketAddr, stream: TcpStream| -> Result<()> {
                let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

                println!("New WebSocket connection: {}", peer);

                while let Some(msg) = ws_stream.next().await {
                    let msg = msg?;
                    if msg.is_text() {
                        let strng=msg.to_string();
                        om.extract.extract_from(&ctx,serde_json::from_str::<serde_json::Value>(strng.as_str()).unwrap()).await;
                        for step in &om.steps {
                            match step {
                                WebSocketStep::SendStep(snd)=>{
                                    ws_stream.send(Message::Text(format!("{}",snd.evaluate(&ctx).await.to_string()) )).await?;
                                },
                                WebSocketStep::NormalStep(stp)=>{
                                    stp.execute(&ctx).await;
                                }
                            }
                        }

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
        let om_out=self.on_message.clone();

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