use std::str::FromStr;
use futures_util::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tokio_tungstenite::{Connector};
use tokio_tungstenite::tungstenite::{Message};
use crate::core::runtime::Context;
use crate::journey::{Executable};
use crate::template::{Expression, Fillable, VariableReferenceName};
use async_trait::async_trait;
use anyhow::{bail, Result};
use hyper::header::HeaderValue;
use hyper_tls::native_tls::TlsConnector;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::HeaderName;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use crate::core::Value;
use crate::journey::step::Step;
use crate::template::rest::FillableRequestHeaders;

pub mod parser;
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketHook{
    variable:VariableReferenceName,
    block:Vec<Step>
}
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketClientConnectStep{
    url:Expression,
    headers: Option<FillableRequestHeaders>,
    connection_name: Expression,
    hook: WebSocketHook,
}
#[derive(Debug, Clone,PartialEq)]
pub struct WebSocketCloseStep{
    name: Expression
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

    async fn execute(&self,context: &Context)->Result<Vec<JoinHandle<Result<bool>>>> {
        let url = self.url.evaluate(context).await?.to_string();
        let name= self.connection_name.evaluate(context).await?.to_string();
        let mut req = url.clone().into_client_request()?;//http::Request::get(url.as_str());
        if let Some(headers) = &self.headers {
            let fh = headers.fill(context).await?;
            let h =req.headers_mut();

            for header in fh.headers {
                h.append(HeaderName::from_str(header.key.as_str())?,HeaderValue::from_str(header.value.clone().as_str())?);
            }
        }
        let start = Instant::now();
        let conn=if url.starts_with("wss") {
            tokio_tungstenite::connect_async_tls_with_config(req, Some(WebSocketConfig::default()), false,Some(Connector::NativeTls(TlsConnector::builder().danger_accept_invalid_certs(true).build()?))).await
        } else {
            tokio_tungstenite::connect_async_with_config(req,Some(WebSocketConfig::default()),false).await
        };//;
        let duration = start.elapsed();
        context.scrapper.ingest("connection_time",duration.as_millis() as f64,vec![(format!("name"),name.clone())]).await;
        match conn {
            Ok((socket, _))=> {
                let (ssk,mut ssm) = socket.split();
                context.websocket_connection_store.define(name.clone(),ssk).await;
                let hook = self.hook.clone();
                let new_ct = context.clone();
                let handle:JoinHandle<Result<bool>> = tokio::spawn(async move {
                    loop {
                        if let Some(Ok(m)) = ssm.next().await{
                            if m.is_text(){
                                let sv = serde_json::from_str(&m.to_string()).unwrap_or(serde_json::Value::String(m.to_string()));
                                new_ct.define(hook.variable.to_string(),Value::from_json_value(sv)).await;
                                let mut handles = vec![];
                                for step in &hook.block {
                                    let mut inner_handles = step.execute(&new_ct).await?;
                                    handles.append(&mut inner_handles);
                                }
                                futures::future::join_all(handles).await;
                            }
                        } else {
                            return Ok(true);
                        }
                    }
                });
                return Ok(vec![handle])
            },
            Err(e)=> {
                context.scrapper.ingest("errors",1.0,vec![(format!("message"),format!("{}",e.to_string())),(format!("api"),format!("{}",url))]).await;
                eprintln!("Error while connecting websocket {} - {}",url,e.to_string());
                bail!("Error while connecting websocket {} - {}",url,e.to_string())
            }
        }


    }

    fn get_deps(&self) -> Vec<String> {
        let mut deps = vec![];
        for step in &self.hook.block {
            deps.append(&mut step.get_deps());
        }
        deps
    }
}
#[async_trait]
impl Executable for WebSocketSendStep{

    async fn execute(&self,context: &Context)->Result<Vec<JoinHandle<Result<bool>>>> {
        let conn_name = self.name.evaluate(context).await?.to_string();
        if let Some(conn) = context.websocket_connection_store.get(conn_name.clone()).await{
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
impl Executable for WebSocketCloseStep{

    async fn execute(&self,context: &Context)->Result<Vec<JoinHandle<Result<bool>>>> {
        let conn_name = self.name.evaluate(context).await?.to_string();
        if let Some(conn) = context.websocket_connection_store.get(conn_name.clone()).await{
            let mut connection = conn.lock().await;

            if let Err(e)=(*connection).close().await{
                context.scrapper.ingest("errors",1.0,vec![(format!("message"),format!("{}",e.to_string())),(format!("connection"),format!("{}",conn_name.clone()))]).await;
                eprintln!("Error {} while closing websocket connection named  {}",e.to_string(),conn_name.clone());
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
#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio_tungstenite::accept_async;
    use crate::core::runtime::Context;
    use crate::journey::Executable;
    use crate::journey::step::websocket::client::WebSocketClientConnectStep;
    use crate::parser::Parsable;
    async fn start_server(rx:tokio::sync::oneshot::Receiver<()>){
        let addr = format!("0.0.0.0:{}",9002);
        let listner = TcpListener::bind(&addr).await.expect("Can't listen");
        let accept_connection = async  move |peer: SocketAddr, stream: TcpStream|{
            let  handle_connection= async move |_peer: SocketAddr, stream: TcpStream| -> anyhow::Result<()> {
                let mut ws_stream = accept_async(stream).await.expect("Failed to accept");
                while let Some(Ok(m)) = ws_stream.next().await {
                    if m.is_text() {
                        ws_stream.send(m).await?
                    }
                }
                Ok(())
            };
            if let Err(e) = handle_connection(peer, stream).await {
                println!("Error processing connection: {}", e)
            }
        };
        let connect = async move||{
            while let Ok((stream,_)) = listner.accept().await{
                let peer = stream.peer_addr().expect("connected streams should have a peer address");
                tokio::spawn(accept_connection(peer,stream));
            };
        };
        tokio::select! {
            _ = connect() => {},
            _ = rx => {},
        }
    }
    #[tokio::test]
    async fn should_execute_websocket_client_connect_step() {
        let (tx,rx)=tokio::sync::oneshot::channel();
        let t=tokio::spawn(start_server(rx));
        let text = r#"connect websocket named "demo" with url "ws://localhost:9002", headers { "x-api-key":"test"} and listener msg => {
        let counter = counter + 1
        print text `<%msg.message%>`
    }
    let ranks = object [1,2,3,4,5,6,7,8,9]
    ranks.for(i)=>{
        send object {"name":"Atmaram","rank": i} on websocket named "demo"
    }"#;
        let (_, step) = WebSocketClientConnectStep::parser(text).unwrap();
        let input = vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context = Context::mock(input, buffer.clone());
        step.execute(&context).await.unwrap();
        tx.send(()).unwrap();
        t.await.unwrap();
        // assert_eq!(context.get_var_from_store(format!("id")).await, Option::Some(Value::PositiveInteger(1)));
        // assert_eq!(context.get_var_from_store(format!("a")).await, Option::Some(Value::String("Hello".to_string())))
    }
}
