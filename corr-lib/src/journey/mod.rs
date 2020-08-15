pub mod step;
pub mod parser;
use crate::journey::step::Step;
use crate::core::{runtime::Context, runtime::IO, Variable, DataType, Value};
use async_trait::async_trait;
#[derive(Debug, Clone)]
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
pub async fn start(journies:&Vec<Journey>,_filter: String,context:Context) {
    loop {
        context.write(format!("Please Enter value between 0 to {}",journies.len()-1)).await;
        let choice=context.read(Variable{
            name:format!("choice"),
            data_type:Option::Some(DataType::PositiveInteger)
        }).await;
        if let Value::PositiveInteger(val) = choice.value.clone(){
            if val < journies.len(){
                journies.get(val).unwrap().execute(&context).await;
                break;
            } else {
                println!("Invalid Value");
                context.write(format!("Invalid Value")).await;
                context.delete(format!("choice")).await;
                continue;
            }
        }
    }

}

#[cfg(test)]
mod tests{
    use crate::core::{ DataType};
    use crate::core::proto::{Input, Output};
    use crate::journey::step::system::SystemStep;
    use std::sync::{Arc, Mutex};
    use crate::journey::{Journey, start};
    use crate::journey::step::Step;
    use crate::template::text::{Text, Block};
    use crate::core::runtime::Context;

    #[tokio::test]
    async fn should_start_journey(){
        let step=SystemStep::Print(Text{
            blocks:vec![Block::Final("Hello World".to_string())]
        });
        let journes = vec![Journey{ name:"test".to_string(),steps:vec![Step::System(step)] }];
        let input = vec![Input::new_continue("choice".to_string(),"0".to_string(),DataType::PositiveInteger)];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        start(&journes,"hello".to_string(),context).await;
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_know_that("Please Enter value between 0 to 0".to_string()));
        assert_eq!(buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_tell_me("choice".to_string(),DataType::PositiveInteger));
        assert_eq!(buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_know_that("Executing Journey test".to_string()));
        assert_eq!(buffer.lock().unwrap().get(3).unwrap().clone(),Output::new_know_that("Hello World".to_string()));

    }
    #[tokio::test]
    async fn should_ask_for_choice_again_journey(){
        let step=SystemStep::Print(Text{
            blocks:vec![Block::Final("Hello World".to_string())]
        });
        let journes = vec![Journey{ name:"test".to_string(),steps:vec![Step::System(step)] }];
        let input = vec![Input::new_continue("choice".to_string(),"3".to_string(),DataType::PositiveInteger),Input::new_continue("choice".to_string(),"0".to_string(),DataType::PositiveInteger)];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        start(&journes,"hello".to_string(),context).await;
            assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_know_that("Please Enter value between 0 to 0".to_string()));
            assert_eq!(buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_tell_me("choice".to_string(),DataType::PositiveInteger));
            assert_eq!(buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_know_that("Invalid Value".to_string()));
            assert_eq!(buffer.lock().unwrap().get(3).unwrap().clone(),Output::new_know_that("Please Enter value between 0 to 0".to_string()));
            assert_eq!(buffer.lock().unwrap().get(4).unwrap().clone(),Output::new_tell_me("choice".to_string(),DataType::PositiveInteger));
            assert_eq!(buffer.lock().unwrap().get(5).unwrap().clone(),Output::new_know_that("Executing Journey test".to_string()));
            assert_eq!(buffer.lock().unwrap().get(6).unwrap().clone(),Output::new_know_that("Hello World".to_string()));


    }
}

