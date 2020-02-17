extern crate tokio;
extern crate websocket;
extern crate corr_core;
extern crate serde_json;
use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;
use websocket::sync::Client;
use self::websocket::websocket_base::stream::sync::Splittable;
use std::convert::TryInto;
use self::corr_core::{Action, DesiredAction, VariableDesciption, VarType, RawVariableValue};
use std::thread::Thread;
use std::time::Duration;

pub trait IO {
    fn send(&mut self,desired_action:DesiredAction);
    fn wait_for_action(&mut self)->Action;
}
impl<T> IO for Client<T> where T:std::io::Read+std::io::Write+Splittable{
    fn send(&mut self,desired_action: DesiredAction) {
        self.send_message(&OwnedMessage::Text(serde_json::to_string(&desired_action).unwrap()));
    }

    fn wait_for_action(&mut self)-> Action {
        let message = self.recv_message().unwrap();
        match message {
            OwnedMessage::Close(_) => {
                Action::Quit
            }
            OwnedMessage::Ping(ping) => {
                Action::Ping
            }
            OwnedMessage::Text(val) => {
                let var:RawVariableValue=serde_json::from_str(&val.as_str()).unwrap();
                Action::Told(var)
            },
            OwnedMessage::Pong(pong)=>{
                Action::Pong
            },
            OwnedMessage::Binary(data)=>{
                Action::Ignorable
            }
        }
    }
}

pub fn start(mut io:Box<IO>){
    let desired_actions = vec![
        DesiredAction::Listen(format!("Welcome to data journey project")),
        DesiredAction::Tell(VariableDesciption{
        name:"name".to_string(),
        data_type:VarType::String
    }),DesiredAction::Tell(VariableDesciption{
        name:"age".to_string(),
        data_type:VarType::Long
    }),DesiredAction::Tell(VariableDesciption{
        name:"gender".to_string(),
        data_type:VarType::Boolean
    }),DesiredAction::Tell(VariableDesciption{
        name:"hobby.length".to_string(),
        data_type:VarType::Long
    }),DesiredAction::Quit];
    let mut cp = 0;

    for  desired_action in desired_actions {
        match &desired_action {
            DesiredAction::Listen(act)=>{
                io.send(desired_action);
                continue;
            },
            DesiredAction::Quit=>{
                io.send(desired_action);
                while let client_action=io.wait_for_action() {
                    match &client_action {
                        Action::Quit=>{
                            return
                        },
                        _=>{
                            return
                        }
                    }
                }
            },
            DesiredAction::Tell(t)=>{
                io.send(desired_action.clone());
                while let client_action=io.wait_for_action() {
                    match &client_action {
                        Action::Quit=>{
                            return
                        },
                        Action::Told(var)=>{
                            if var.is_valid() {
                                println!("told me variable {:?}", var);
                                io.send(DesiredAction::Listen(format!("Thanks got variable {:?}", var)));
                                break;
                            } else {
                                io.send(DesiredAction::Listen(format!("Not Valid {:?}", var.data_type)));
                                io.send(desired_action.clone());
                            }
                        },
                        _=>{}
                    }
                }

            }
        }
    }

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

            let mut client = request.use_protocol("rust-websocket").accept().unwrap();

            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);
            start(Box::new(client))
        });
    }
}
