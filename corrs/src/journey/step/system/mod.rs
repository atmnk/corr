use crate::journey::step::Step;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::journey::{Executable, Context, IO};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemStep{
    Print,
    Collection(Vec<Step>)
}
#[async_trait]
impl Executable for SystemStep{
    async fn execute(&self,context: &Context) {
        match self {
            SystemStep::Print=>{
                context.write(format!("Hello World")).await;
            },
            SystemStep::Collection(steps)=>{
                for step in steps {
                    step.execute(context).await
                }
            }
        }
    }
}