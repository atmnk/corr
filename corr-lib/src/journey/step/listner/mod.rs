pub mod parser;
use crate::template::object::extractable::{Extractable};
use crate::template::rest::{ RequestBody, RequestHeaders, RestVerb, FillableRequest};
use crate::template::rest::extractable::{ ExtractableResponse, CorrResponse};
use crate::journey::Executable;
use crate::core::runtime::{Context, IO, Client};
use isahc::prelude::*;
use crate::template::{Fillable, Expression};
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use async_trait::async_trait;
use crate::core::{Variable, Value, Number};
use serde_json::error::Category::Data;
use crate::template::text::Text;
use crate::parser::Parsable;
use crate::core::proto::{Output, Input};
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use std::future::Future;
use hyper::server::conn::AddrStream;
use tokio::task::JoinHandle;
use regex::bytes::Regex;
use crate::journey::step::Step;
use crate::template::text::extractable::ExtractableText;

async fn handle(
    context: Context,
    sls:StartListenerStep,
    addr: SocketAddr,
    req: Request<Body>,
    lock:Arc<tokio::sync::RwLock<u16>>
) -> hyper::http::Result<Response<Body>> {
    let _ = lock.write().await;
    {
        for stub in sls.stubs{
            let context = Context::from(&context).await;
            if stub.url.capture(&req.uri().to_string(),&context).await && req.method().to_string().to_lowercase().eq(&stub.method.as_str().to_lowercase()) {
                let context = Context::from(&context).await;
                for step in stub.steps {
                    step.execute(&context).await;
                }

                let resp = stub.response.body.evaluate(&context).await;

                return Response::builder()
                    .status(StatusCode::from_u16(stub.response.status).unwrap_or(StatusCode::OK))
                    .body(Body::from(resp.to_string()));
            }
        }
    }

    return Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not found".to_string()));

}


async fn start_imposter_on_port(context:&Context,sls :StartListenerStep)->Vec<JoinHandle<bool>>{

    let port:u16 = sls.port.evaluate(context).await.parse().unwrap_or(8100);
    let context = context.clone();
    let cloned = sls.clone();
    let lock = Arc::new(tokio::sync::RwLock::new(port));
    let make_service = make_service_fn(move |conn: &AddrStream| {
        // We have to clone the context to share it with each invocation of
        // `make_service`. If your data doesn't implement `Clone` consider using
        // an `std::sync::Arc`.
        let context = context.clone();
        let sls = cloned.clone();

        // You can grab the address of the incoming connection like so.
        let addr = conn.remote_addr();
        let lock = lock.clone();

        // Create a `Service` for responding to the request.
        let service = service_fn(move |req| {
            handle(context.clone(),sls.clone(), addr, req,lock.clone())
        });

        // Return the service to hyper.
        async move { Ok::<_, Infallible>(service) }
    });

    // Run the server like above...
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let server = Server::bind(&addr).serve(make_service);
    println!("starting server on {:}",port);
    let handle = tokio::spawn(async {
        server.await;

        return true
    });
    vec![handle]
    // if let Err(e) = server.await {
    //     eprintln!("server error: {}", e);
    // }
}
struct SystemRuntime;
#[async_trait]
impl Client for SystemRuntime{
    async fn send(&self, output: Output) {
        match output {
            Output::TellMe(a)=>{
                eprintln!("Don't know value for {:?} of type {:?}",a.name,a.data_type)
            },
            Output::KnowThat(k)=>{
                println!("{:?}",k.message)
            },
            _=>{
                println!("Don't know what to do")
            }
        }
    }

    async fn get_message(&mut self) -> Input {
        panic!("Should not happen")
    }
}
#[derive(Debug, Clone,PartialEq)]
pub struct Stub{
    method: RestVerb,
    url: ExtractableText,
    steps:Vec<Step>,
    response: StubResponse
}

#[derive(Debug, Clone,PartialEq)]
pub struct StartListenerStep{
    port : Expression,
    stubs : Vec<Stub>
}
#[derive(Debug, Clone,PartialEq)]
pub struct StubResponse{
    status : u16,
    body : Expression
}
impl StubResponse {
    pub fn from(status:Option<u128>,body:Expression)->Self{
        Self {
            status:status.map(|s|s as u16).unwrap_or(200),
            body
        }
    }
}

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World".into()))
}
#[async_trait]
impl Executable for StartListenerStep {
    async fn execute(&self, context: &Context)->Vec<JoinHandle<bool>> {
        start_imposter_on_port(context,self.clone()).await
    }
}