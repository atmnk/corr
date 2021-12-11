pub mod system;
pub mod rest;
pub mod parser;
pub mod listner;
pub mod db;
use crate::journey::{Executable};
use crate::journey::step::system::SystemStep;
use async_trait::async_trait;
use crate::core::runtime::Context;
use crate::journey::step::rest::RestSetp;
use crate::journey::step::listner::StartListenerStep;
use tokio::task::JoinHandle;
use crate::journey::step::db::{DefineConnectionStep, ExecuteStep};

#[derive(Debug, Clone,PartialEq)]
pub enum Step{
    System(SystemStep),
    Rest(RestSetp),
    Listner(StartListenerStep),
    DefineConnection(DefineConnectionStep),
    InsertStep(ExecuteStep)
    // Rest(RestStep)
}


#[async_trait]
impl Executable for Step{
    async fn execute(&self,context: &Context)->Vec<JoinHandle<bool>> {
        match self {
            Step::System(sys_step)=>{
                return sys_step.execute(context).await
            },
            Step::Rest(rst_step)=>{
                return rst_step.execute(context).await
            }
            Step::Listner(sls)=>{
                return sls.execute(context).await
            },
            Step::DefineConnection(dcs)=>{
                return dcs.execute(context).await
            },
            Step::InsertStep(is)=>{
                return is.execute(context).await
            }
            // Step::Rest(rest_step)=>{
            //     rest_step.execute(context).await
            // }
        }
    }

}
#[cfg(test)]
mod tests{
    use crate::core::{ DataType};
    use crate::core::proto::{Input, Output};
    use std::sync::{Arc, Mutex};
    use crate::journey::{ Executable};
    use crate::journey::step::Step;
    use crate::core::runtime::Context;
    use crate::parser::Parsable;

    #[tokio::test]
    async fn should_execute_system_step(){
        let text = r#"print text `Hello World`;"#;
        let (_,step)=Step::parser(text).unwrap();
        let input = vec![Input::new_continue("choice".to_string(),"0".to_string(),DataType::PositiveInteger)];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        step.execute(&context).await;
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_know_that("Hello World".to_string()));

    }
}