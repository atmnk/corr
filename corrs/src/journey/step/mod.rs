pub mod system;
use serde::{Deserialize, Serialize};
use crate::journey::{Executable, Context};
use crate::journey::step::system::SystemStep;
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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