extern crate tokio;
extern crate websocket;
extern crate corr_core;
extern crate serde_json;
extern crate corr_journeys;
use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;
use websocket::sync::Client;
use self::websocket::websocket_base::stream::sync::Splittable;
use corr_journeys::Journey;
use self::corr_journeys::{JourneyStore, Interactable};
use std::cell::RefCell;
use std::rc::Rc;
use corr_websocket::{Action, DesiredAction};
use corr_core::runtime::{Variable, VarType, Value, RawVariableValue, VariableDesciption};
use self::corr_core::runtime::{ValueProvider, Environment, RCValueProvider};
use std::collections::HashMap;
use serde::export::PhantomData;
use corr_rest::{GetStep, PostStep};
#[derive(Debug)]
pub struct SocketClient<T>(T) where T:IO;

pub trait IO {
    fn send(&mut self,desired_action:DesiredAction);
    fn wait_for_action(&mut self)->Action;
    fn close(&mut self);
}
impl<T> IO for Client<T> where T:std::io::Read+std::io::Write+Splittable{
    fn send(&mut self,desired_action: DesiredAction) {
        self.send_message(&OwnedMessage::Text(serde_json::to_string(&desired_action).unwrap())).unwrap();
    }

    fn wait_for_action(&mut self)-> Action {
        let resp=self.recv_message();
        match resp {
            Ok(message)=>{
                match message {
                    OwnedMessage::Close(_) => {
                        Action::Quit
                    }
                    OwnedMessage::Ping(_) => {
                        Action::Ping
                    }
                    OwnedMessage::Text(val) => {
                        let var:RawVariableValue=serde_json::from_str(&val.as_str()).unwrap();
                        Action::Told(var)
                    },
                    OwnedMessage::Pong(_)=>{
                        Action::Pong
                    },
                    OwnedMessage::Binary(_)=>{
                        Action::Ignorable
                    }
                }
            },
            Err(_)=>{
                Action::Quit
            }
        }
        
    }

    fn close(&mut self) {
    }
}
impl<T> ValueProvider for SocketClient<T> where T:IO{


    fn read(&mut self, variable: Variable) -> Value {
        let desired_action = DesiredAction::Tell(VariableDesciption{
           name: variable.name.clone(),
            data_type:match variable.data_type {
                Option::None=>{
                    VarType::String
                }
                Option::Some(dt)=>{
                    dt
                }
            }
        });

        self.0.send(desired_action.clone());
        loop{
            let client_action=self.0.wait_for_action();
            match &client_action {
                Action::Told(var)=>{
                    if var.is_valid() {
                        println!("told me variable {:?}", var);
                        return var.to_value()
                    } else {
                        self.0.send(DesiredAction::Listen(format!("Not Valid {:?}", var.data_type)));
                        self.0.send(desired_action.clone());
                        continue;
                    }
                },
                _=>{
                    continue;
                }
            }
        }
    }
    fn write(&mut self, text: String) {
        self.0.send(DesiredAction::Listen(text))
    }

    fn close(&mut self) {
        self.0.send(DesiredAction::Quit);
        loop{
            let client_action=self.0.wait_for_action();
            match &client_action {
                Action::Quit=>{
                    self.0.close();
                    return;
                },
                _=>{
                    continue;
                }
            }
        }

    }
    fn set_index_ref(&mut self, _: Variable, _: Variable) { 

    }
    fn drop(&mut self, _: String) { 

    }

    fn load_ith_as(&mut self, i: usize, index_ref_var: Variable, list_ref_var: Variable) {
            
    }
}
pub fn start<T>(io:SocketClient<T>) where T:IO{
    let journeys=vec![
        Journey{
            name:format!("Post Step"),
            steps:vec![Box::new(PostStep{})]
        },
        Journey{
            name:format!("Get Step"),
            steps:vec![Box::new(GetStep{})]
        }
    ];
    let js = JourneyStore {
        journeys
    };
    let mut rc_channel=RCValueProvider {
        value_store:Vec::new(),
        reference_store:Rc::new(RefCell::new(HashMap::new())),
        fallback_provider:io,
        indexes:HashMap::new()
    };
    js.start_with(format!("hello"),Environment{ channel:Rc::new(RefCell::new(rc_channel))});
}
pub fn create_server() {
    let server = Server::bind("127.0.0.1:9876").unwrap();

    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(|| {
            if !request.protocols().contains(&"rust-websocket".to_string()) {
                request.reject().unwrap();
                return;
            }

            let client = request.use_protocol("rust-websocket").accept().unwrap();

            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);
            start(SocketClient(client))
        });
    }
}
