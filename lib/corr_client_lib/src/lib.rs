extern crate rand;
extern crate serde;
extern crate websocket;
extern crate serde_json;
use corr_core::runtime::VariableDesciption;
use corr_core::runtime::RawVariableValue;
use corr_core::io::StringIO;
use corr_core::runtime::Messanger;
use corr_core::io::StdStringIO;
use corr_websocket::{DesiredAction};
use websocket::{ClientBuilder, OwnedMessage};
use websocket::client::sync::Client;
use websocket::stream::sync::TcpStream;
pub struct CliClient<T,U> where T:Server,U:StringIO {
    pub server:Box<T>,
    pub io:Messanger<U>
}
impl CliClient<WebSocketServer,StdStringIO>{
    pub fn new(server:String) -> CliClient<WebSocketServer, StdStringIO> {
        CliClient {
            server:connect(server),
            io:Messanger::new(StdStringIO{})
        }
    }

}
pub struct WebSocketServer {
    client: Client<TcpStream>
}


#[derive(Debug,PartialEq)]
pub struct Filter{
    pub value:String
}


pub trait Server {
    fn start(&mut self)->Result<(), Box<dyn std::error::Error>>;
    fn whatsNext(&mut self)->Result<DesiredAction,Box<dyn std::error::Error>>;
    fn tell(&mut self,variable:RawVariableValue)->Result<(),Box<dyn std::error::Error>>;
}
pub trait JourneyClient {
    fn run(&mut self,filter:Filter)-> Result<(), Box<dyn std::error::Error>>;
    fn ask(&mut self,var_Desc:VariableDesciption)->RawVariableValue;
    fn tell(&mut self,words:String);
}
fn connect(server: String)->Box<WebSocketServer> {
    let client = ClientBuilder::new(format!("ws://{}:9876",server).as_str())
        .unwrap()
        .add_protocol("rust-websocket")
        .connect_insecure()
        .expect(format!("Failed to connect to server {}",server).as_str());
    Box::new(WebSocketServer {
        client
    })
}
impl<T,U> JourneyClient for CliClient<T,U> where T:Server,U:StringIO {
    fn run(&mut self,filter:Filter)-> Result<(), Box<dyn std::error::Error>>{
        self.tell(format!("running filter {:?}",filter));
        self.server.start()?;
        loop {
            let next = self.server.whatsNext()?;
            match next {
                DesiredAction::Quit => return Ok(()),
                DesiredAction::Tell(var)=>{
                    let val=self.ask(var);
                    self.server.tell(val);
                },
                DesiredAction::Listen(words)=>{
                    self.tell(words);
                }
            }
        }
    }

    fn ask(&mut self, var_desc: VariableDesciption)->RawVariableValue {
        let clone=var_desc.clone();
        let var = self.io.ask(clone);
        if var.is_valid(){
            var
        }
        else {
            self.io.tell(format!("Given value is not valid {:?}",var_desc.data_type));
            self.io.ask(var_desc)
        }
    }

    fn tell(&mut self, words: String) {
        self.io.tell(words)
    }
}
impl Server for WebSocketServer {
    fn start(&mut self) ->Result<(), Box<dyn std::error::Error>>{
        Ok(())
    }

    fn whatsNext(&mut self)->Result<DesiredAction,Box<dyn std::error::Error>> {
        loop{
            let message=self.client.recv_message().unwrap();
            match message {
                OwnedMessage::Text(val)=>{
                    let da:DesiredAction=serde_json::from_str(&val.as_str()).unwrap();
                    match da {
                        DesiredAction::Quit=>{
                            self.client.send_message(&OwnedMessage::Close(None));
                            self.client.shutdown();
                        },
                        _=>{

                        }
                    }
                    return Ok(da);
                },
                OwnedMessage::Close(_)=>{
                    self.client.send_message(&OwnedMessage::Close(None));
                    self.client.shutdown();
                    return Ok(DesiredAction::Quit)
                }
                _=>{
                }
            }
        }
    }

    fn tell(&mut self, variable: RawVariableValue)->Result<(),Box<dyn std::error::Error>>  {
        self.client.send_message(&OwnedMessage::Text(serde_json::to_string(&variable).unwrap()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    pub fn newWithMocks(server:Box<MockServer>, io:Messanger<VecBufferStringIO>) -> CliClient<MockServer, VecBufferStringIO> {
        CliClient {
            server,
            io
        }
    }
    pub struct VecBufferStringIO {
        pointer:usize,
        told:Vec<String>,
        captured:Vec<String>
    }
    impl VecBufferStringIO {
        pub fn new()-> VecBufferStringIO {
            VecBufferStringIO {
                pointer:0,
                captured:Vec::new(),
                told:Vec::new()
            }
        }
    }

    impl StringIO for VecBufferStringIO {
        fn write(&mut self, value: String) {
            self.told.push(value);
        }

        fn read_raw(&mut self) -> String {

            let ret=self.captured.get(self.pointer).unwrap().clone();
            self.pointer = self.pointer + 1;
            ret
        }
    }

    pub struct MockServer {
        pub base_url:String,
        pub actions:Vec<DesiredAction>,
        pub pointer:usize,
        told:Vec<RawVariableValue>
    }
    fn connect_mock(server: String,actions:Vec<DesiredAction>)->Box<MockServer> {
        Box::new(MockServer {
            base_url:format!("http://{}:9876",server),
            actions:actions,
            pointer:0,
            told:Vec::new()
        })
    }
    impl Server for MockServer {
        fn start(&mut self) ->Result<(), Box<dyn std::error::Error>>{
            Ok(())
        }

        fn whatsNext(&mut self)->Result<DesiredAction,Box<dyn std::error::Error>>  {
            self.pointer += 1;
            if self.actions.len()>=self.pointer {
                Ok(self.actions[self.pointer-1].clone())
            } else {
                Ok(DesiredAction::Quit)
            }


        }

        fn tell(&mut self, variable: RawVariableValue)->Result<(),Box<dyn std::error::Error>>  {
            self.told.push(variable);
            Ok(())
        }
    }
    use corr_core::io::StringIO;
use corr_core::runtime::Messanger;
use corr_core::runtime::VariableDesciption;
use corr_core::runtime::{RawVariableValue,VarType};
use corr_websocket::DesiredAction;
use crate::{CliClient, JourneyClient, Filter, Server};

    #[test]
    fn should_run_journey(){
        let mut actions= vec![
            DesiredAction::Listen("Whats Your Choise".to_string()),
            DesiredAction::Tell(VariableDesciption{
                name:"choice".to_string(),
                data_type:VarType::Long
            }),
            DesiredAction::Tell(VariableDesciption{
                name:"name".to_string(),
                data_type:VarType::String
            })
        ];
        let mut server_ip = "localhost".to_string();
        let mut server = connect_mock(server_ip,actions);
        let mut io = Messanger{
            string_io:Box::new(VecBufferStringIO {
                pointer:0,
                captured:vec![format!("3.2\n"),format!("3\n"),format!("Atmaram\n\t")],
                told:Vec::new()
            })
        };
        let mut client = newWithMocks(server,io
        );
        client.run(Filter {
            value:format!("create data")
        });
        assert_eq!(client.io.string_io.told,vec![
            format!("running filter Filter {{ value: \"create data\" }}"),
            format!("Whats Your Choise"),
            format!("Please enter value for choice of type Long"),
            format!("Given value is not valid Long"),
            format!("Please enter value for choice of type Long"),
            format!("Please enter value for name of type String"),
        ]);
        assert_eq!(client.server.told,vec![
            RawVariableValue{
                name:format!("choice"),
                value:Option::Some(format!("3")),
                data_type:VarType::Long
            },
            RawVariableValue{
                name:format!("name"),
                value:Option::Some(format!("Atmaram")),
                data_type:VarType::String
            }
        ]);
    }
}