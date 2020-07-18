use tokio::sync::{mpsc};
use warp::ws::{Message, WebSocket};
use futures::stream::SplitStream;
use futures::{StreamExt};
use crate::proto::{Input, Output, StartInput};
use crate::proto::Result;
use crate::journey::{JourneyController, Journey};
use async_trait::async_trait;
use crate::core::{Value, DataType};

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
    pub fn send(&self,output:Output){
        if let Err(_disconnected) = self.tx.send(Ok(Message::text(serde_json::to_string(&output).unwrap()))) {

        }
    }
    pub async fn get_message(&mut self)->Input{
            let mut ret=Input::Start(StartInput{filter:format!("hello")});
            loop{
                if let Some(result) = self.user_ws_rx.next().await {
                    let message = match result {
                        Ok(msg) => msg,
                        Err(e) => {
                            eprintln!("websocket error( {}", e);
                            break;
                        }
                    };
                    let input:Input = serde_json::from_str(message.to_str().unwrap()).unwrap();
                    eprintln!("Got Message{:?}",input);
                    ret=input;
                    break;
                }
            };
        return ret;
    }
}
impl Hub {
    pub fn new() -> Self {
        Hub {
        }
    }
    pub async fn start(&self,mut user:User) {
        let message=user.get_message().await;
        let filter =match message {
            Input::Start(start_input)=>start_input.filter,
            _=>format!("")
        };
        let mut user_journey_controller=UserJourneyContoller{
            user
        };
        user_journey_controller.start(&vec![Journey{name:format!("Wonderfull")}],filter).await
    }
}
struct UserJourneyContoller{
    user:User
}
#[async_trait]
impl JourneyController for UserJourneyContoller{
    async fn write_message(&self, message: String) {
        self.user.send(Output::new_know_that(message));
    }

    async fn start(&mut self,journies:&Vec<Journey>,_filter: String) {
        self.user.send(Output::new_know_that(format!("Please Enter value between 0 to {}",journies.len()-1)));
        self.user.send(Output::new_tell_me(format!("choice"),DataType::Long));
        loop{
            let message=self.user.get_message().await;
            if let Some(var) =match message {
                Input::Continue(continue_input)=>continue_input.convert(),
                _=>unimplemented!()
            }{
                match &var.value {
                    Value::Long(val)=>{
                        self.user.send(Output::new_know_that(format!("Executing Journey {}",journies.get(val.clone()).unwrap().name)));
                        self.execute(journies.get(val.clone()).unwrap().clone()).await;
                        break;
                    },
                    _=>continue
                }

            }
        }

    }
    async fn execute(&mut self,journey:Journey){
    }
}
