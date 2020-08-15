pub mod system;
pub mod parser;
use crate::journey::{Executable};
use crate::journey::step::system::SystemStep;
use async_trait::async_trait;
use crate::core::runtime::Context;

#[derive(Debug, Clone)]
pub enum Step{
    System(SystemStep)
}


#[async_trait]
impl Executable for Step{
    async fn execute(&self,context: &Context) {
        match self {
            Step::System(sys_step)=>{
                sys_step.execute(context).await
            }
        }
    }

}