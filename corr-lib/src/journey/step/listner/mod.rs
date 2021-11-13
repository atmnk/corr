pub mod parser;

use crate::template::rest::{RestVerb, MultipartField};

use crate::journey::Executable;
use crate::core::runtime::{Context, Client};
use crate::template::{Expression};
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use async_trait::async_trait;




use crate::core::proto::{Output, Input};

use std::sync::{Arc};


use hyper::server::conn::AddrStream;
use tokio::task::JoinHandle;

use crate::journey::step::Step;
use crate::template::text::extractable::ExtractableText;
use crate::template::object::extractable::{Extractable};
use crate::template::rest::extractable::{ExtractableRestData};
use multer::Multipart;
use crate::core::Value;
use lazy_static::lazy_static; // 1.4.0
use tokio::sync::Mutex;
lazy_static! {
    static ref LOGS: Mutex<Vec<Option<u16>>> = Mutex::new(vec![]);
}
async fn handle(
    context: Context,
    sls:StartListenerStep,
    _addr: SocketAddr,
    req: Request<Body>,
    _lock:Arc<tokio::sync::RwLock<u16>>
) -> hyper::http::Result<Response<Body>> {
    let mut ret = Option::None;
    let mut ret_st = Option::Some(404);
    {
        let mut logs = LOGS.lock().await;
        {
            for stub in sls.stubs {
                let context = Context::from_without_fallback(&context).await;
                if stub.url.capture(&req.uri().to_string(), &context).await && req.method().to_string().to_lowercase().eq(&stub.method.as_str().to_lowercase()) {
                    let opt_bd = req.headers().get(hyper::header::CONTENT_TYPE).and_then(|ct| ct.to_str().ok()).and_then(|ct| multer::parse_boundary(ct).ok());
                    let (parts, body) = req.into_parts();
                    if let Some(boundary) = opt_bd {
                        let mut mp = Multipart::new(body, boundary);
                        let mut fields = vec![];
                        while let Ok(Some(field)) = mp.next_field().await {
                            fields.push(
                                MultipartField {
                                    name: field.name().map(|n| n.to_string()),
                                    content_type: field.content_type().map(|ct| ct.to_string()),
                                    file_name: field.file_name().map(|n| n.to_string()),
                                    contents: field.bytes().await.ok(),

                                });
                        }
                        stub.rest_data.extract_from(&context, (fields, parts.headers.clone())).await;
                    } else {
                        if let Ok(data) = hyper::body::to_bytes(body).await {
                            let sv = serde_json::from_str::<serde_json::Value>(String::from_utf8_lossy(&data).as_ref());
                            match sv {
                                Ok(val) => {
                                    stub.rest_data.extract_from(&context, (val, parts.headers.clone())).await;
                                },
                                Err(_) => {
                                }
                            }
                        } else {
                        }
                    }


                    let context = Context::from(&context).await;
                    for step in stub.steps {
                        step.execute(&context).await;
                    }

                    let resp = stub.response.body.evaluate(&context).await;
                    let status: Option<u16> = stub.response.status.evaluate(&context).await.parse();
                    ret_st = status.clone();
                    ret = Option::Some(Response::builder()
                        .status(StatusCode::from_u16(status.unwrap_or(200)).unwrap_or(StatusCode::OK))
                        .header("Content-Type", "application/json")
                        .body(Body::from(resp.to_string())));
                    break;
                }
            }
        }
        logs.push(ret_st);
    }
    if let Some(f_ret) = ret {
        f_ret
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found".to_string()))
    }
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
        let service = service_fn( move |req| {
            handle(context.clone(),sls.clone(), addr, req,lock.clone())
        });

        // Return the service to hyper.
        async move { Ok::<_, Infallible>(service) }
    });

    // Run the server like above...
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let server = Server::bind(&addr).serve(make_service);
    println!("starting server on {:}",port);
    let handle = tokio::spawn(async {
        if let Some(_) = server.await.ok(){
            return true
        }
        return false
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
    rest_data:ExtractableRestData,
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
    status : Expression,
    body : Expression
}
impl StubResponse {
    pub fn from(status:Option<Expression>,body:Expression)->Self{
        Self {
            status:status.map(|s|s ).unwrap_or(Expression::Constant(Value::PositiveInteger(200))),
            body
        }
    }
}

#[async_trait]
impl Executable for StartListenerStep {
    async fn execute(&self, context: &Context)->Vec<JoinHandle<bool>> {
        start_imposter_on_port(context,self.clone()).await
    }
}