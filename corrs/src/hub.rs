use tokio::sync::{mpsc};
use warp::ws::{Message, WebSocket};
use futures::stream::SplitStream;
use futures::{StreamExt};
use crate::proto::{Input, Output, StartInput};
use crate::proto::Result;
use crate::journey::{JourneyController, Journey, Context, IO, Executable, Client};
use async_trait::async_trait;
use crate::core::{DataType, Variable, Value};
use std::sync::{Arc};
use futures::lock::Mutex;
use crate::journey::step::Step;
use crate::journey::step::system::SystemStep;

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
            let mut user_journey_controller=UserJourneyContoller{};
            let context = Context {
                user:shared_user.clone()
            };
            user_journey_controller.start(&vec![Journey{name:format!("Wonderfull"),steps:vec![Step::System(SystemStep::Print)]}],filter,context).await
        }

    }
}
struct UserJourneyContoller;
#[async_trait]
impl JourneyController for UserJourneyContoller{
    async fn start(&mut self,journies:&Vec<Journey>,_filter: String,context:Context) {
        loop{
            context.write(format!("Please Enter value between 0 to {}",journies.len()-1)).await;
            let choice=context.read(Variable{
                name:format!("choice"),
                data_type:DataType::Long
            }).await;
            if let Value::Long(val) = choice.value.clone(){
                if val < journies.len(){
                    journies.get(val).unwrap().execute(&context).await;
                } else {
                    continue;
                }
            }
        }
    }
}
