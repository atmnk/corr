pub mod step;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures::lock::Mutex;
use crate::hub::User;
use crate::proto::{Output, Input};
use crate::journey::step::Step;
use crate::core::{Variable, VariableValue};

#[async_trait]
pub trait JourneyController{
    async fn start(&mut self,journies:&Vec<Journey>,filter:String,context:Context);
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Journey{
    pub name:String,
    pub steps:Vec<Step>
}


#[async_trait]
pub trait Executable{
    async fn execute(&self,context:&Context);
}
pub struct Context{
    pub user:Arc<Mutex<User>>
}
#[async_trait]
impl IO for Context {
    async fn write(&self, data:String){
        self.user.lock().await.send(Output::new_know_that(data));
    }

    async fn read(&self, variable: Variable)->VariableValue{
        self.user.lock().await.send(Output::new_tell_me(variable.name.clone(),variable.data_type.clone()));
        loop{
            let message=self.user.lock().await.get_message().await;
            if let Some(var) =match message {
                Input::Continue(continue_input)=>continue_input.convert(),
                _=>Option::None
            }{
                if var.name.eq(&variable.name){
                    return var;
                } else {
                    continue;
                }
            } else {
                self.user.lock().await.send(Output::new_know_that(format!("Invalid Value")));
                self.user.lock().await.send(Output::new_tell_me(variable.name.clone(),variable.data_type.clone()));
            }
        }
    }
}
#[async_trait]
pub trait IO {
    async fn write(&self,data:String);
    async fn read(&self,variable:Variable)->VariableValue;
}
#[async_trait]
impl Executable for Journey{
    async fn execute(&self, context: &Context) {
        context.write(format!("Executing Journey {}",self.name)).await;
        for step in self.steps.iter() {
            step.execute(context).await
        }
        context.write(format!("Done Executing Journey {}",self.name)).await;
    }
}
