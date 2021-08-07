#![feature(generators, generator_trait)]
#![feature(async_closure)]
use async_trait::async_trait;
use corr_lib::core::runtime::{Context, Client};
use std::sync::{Arc, Mutex};
use std::convert::Infallible;
use lazy_static::lazy_static;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use corr_lib::core::proto::{Output, Input, TellMeOutput};
use corr_lib::core::{Value};
use corr_lib::journey::step::system::AssignmentStep;
use corr_lib::template::{VariableReferenceName, Assignable, Expression};
use corr_lib::journey::Executable;

lazy_static! {
    static ref LastAsk: Mutex<Vec<TellMeOutput>> = Mutex::new(vec![]);
    static ref GlobalContext: Context = {
        Context::new(Arc::new(futures::lock::Mutex::new(SystemRuntime)))
    };

}
struct SystemRuntime;
#[async_trait]
impl Client for SystemRuntime{
    async fn send(&self, output: Output) {
        match output {
            Output::TellMe(a)=>{
                LastAsk.lock().unwrap().push(a);
                // eprintln!("Don't know value for {:?} of type {:?}",a.name,a.data_type)
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
        let ask = LastAsk.lock().unwrap().pop().unwrap();
        if ask.name.ends_with("::length") {
            Input::new_continue(ask.name.clone(),format!("0"),ask.data_type.clone())
        } else {
            Input::new_continue(ask.name.clone(),format!("null"),ask.data_type.clone())
        }
    }
}
#[tokio::main]
async fn main() {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let _store = 1123;
    let assmnt = AssignmentStep::WithVariableName(VariableReferenceName::from("name"),Assignable::Expression(Expression::Constant(Value::String(format!("Hello World")))));
    assmnt.execute(&GlobalContext).await;
    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn::<_,_, _>( async move |_req: Request<Body>|->Result<Response<Body>, Infallible>{
            let value = GlobalContext.get_var_from_store(format!("name")).await;
            return Ok(Response::new(format!("{0}",value.unwrap().to_string()).into()));
        }))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
