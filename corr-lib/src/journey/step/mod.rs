pub mod system;
pub mod rest;
pub mod parser;
use crate::journey::{Executable};
use crate::journey::step::system::SystemStep;
use async_trait::async_trait;
use crate::core::runtime::Context;
use crate::journey::step::rest::RestStep;

#[derive(Debug, Clone,PartialEq)]
pub enum Step{
    System(SystemStep),
    Rest(RestStep)
}


#[async_trait]
impl Executable for Step{
    async fn execute(&self,context: &Context) {
        match self {
            Step::System(sys_step)=>{
                sys_step.execute(context).await
            },
            Step::Rest(rest_step)=>{
                rest_step.execute(context).await
            }
        }
    }

}