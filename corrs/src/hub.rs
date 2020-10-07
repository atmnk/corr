use tokio::sync::{mpsc};
use warp::ws::{Message, WebSocket};
use futures::stream::SplitStream;
use futures::{StreamExt};
use corr_lib::core::proto::{Input, Output, StartInput};
use corr_lib::core::proto::Result;
use corr_lib::journey::{Journey, start};
use async_trait::async_trait;
use corr_lib::core::runtime::{ Client, Context};
use std::sync::{Arc};
use futures::lock::Mutex;
use corr_lib::parser::{Parsable, result_option};
use app_dirs2::{app_root, AppDataType, AppInfo};
use std::fs::File;
use std::io::Read;
const APP_INFO: AppInfo = AppInfo{name: "corrs", author: "Atmaram Naik"};
pub struct Hub{
}
pub struct User {
    pub tx:mpsc::UnboundedSender<Result<Message>>,
    pub user_ws_rx:SplitStream<WebSocket>
}
impl User {
    pub fn new(tx:mpsc::UnboundedSender<Result<Message>>,user_ws_rx:SplitStream<WebSocket>)->User{
        User{
            tx,
            user_ws_rx
        }
    }

}
#[async_trait]
impl Client for User{
    fn send(&self,output:Output){
        if let Err(_disconnected) = self.tx.send(Ok(Message::text(serde_json::to_string(&output).unwrap()))) {

        }
    }
    async fn get_message(&mut self)->Input{
            let mut ret=Input::Start(StartInput{filter:format!("hello")});
            ret=if let Some(result) = self.user_ws_rx.next().await {
                let message = match result {
                    Ok(msg) => msg,
                    Err(e) => {
                        println!("{:?}",e);
                        unimplemented!()
                    }
                };
                eprintln!("{:?}",message);
                let input:Input = serde_json::from_str(message.to_str().unwrap()).unwrap();
                eprintln!("Got Message{:?}",input);
                input
            } else {
                ret
            };
            return ret;
    }
}
impl Hub {
    pub fn new() -> Self {
        Hub {
        }
    }
    pub async fn start(&self,user:User) {
        let shared_user = Arc::new(Mutex::new(user));
        loop {
            let message=shared_user.lock().await.get_message().await;
            let filter =match message {
                Input::Start(start_input)=>start_input.filter,
                _=>format!("")
            };
            let context = Context::new(shared_user.clone());
            let journies=get_journies();
            start(&journies,filter,context).await;
            shared_user.lock().await.send(Output::new_done("Done Executing Journey".to_string()));
        }

    }
}
pub fn get_journies()->Vec<Journey>{
    let mut journeys=Vec::new();
    let app_path=app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
    let path=app_path.join("journeys");
    for dir_entry in std::fs::read_dir(path).unwrap(){
        let dir_entry=dir_entry.unwrap().path();
        if dir_entry.is_file() {
            if let Some(extention) = dir_entry.extension() {
                match extention.to_str() {
                    Some("journey") => {
                        println!("reading from file {:?}",dir_entry);
                        let ctc=File::open(dir_entry).unwrap();
                        if let Some(journey)=read_journey_from_file(ctc){
                            journeys.push(journey);
                        }
                    },
                    _=>{}
                }
            }

        }
    }
    return journeys;
}
pub fn read_journey_from_file(mut file:File)->Option<Journey>{
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    result_option(contents.as_str(),Journey::parser(contents.as_str()))

}

