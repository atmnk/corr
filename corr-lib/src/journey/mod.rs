pub mod step;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures::lock::Mutex;
use crate::core::proto::{Output, Input};
use crate::journey::step::Step;
use crate::core::{Variable, VariableValue, Context, IO};
use async_trait::async_trait;

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



#[async_trait]
impl Executable for Journey{
    async fn execute(&self, context: &Context) {
        context.write(format!("Executing Journey {}",self.name)).await;
        for step in self.steps.iter() {
            step.execute(context).await
        }
    }
}
