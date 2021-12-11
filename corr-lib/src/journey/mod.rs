pub mod step;
pub mod parser;
use crate::journey::step::Step;
use crate::core::{runtime::Context, runtime::IO, Variable, DataType, Value};
use async_trait::async_trait;
use tokio::task::JoinHandle;

#[derive(Debug, Clone,PartialEq)]
pub struct Journey{
    pub name:String,
    pub steps:Vec<Step>,
    pub params:Vec<Variable>
}


#[async_trait]
pub trait Executable{
    async fn execute(&self,context:&Context)->Vec<JoinHandle<bool>>;
}



#[async_trait]
impl Executable for Journey{
    async fn execute(&self, context: &Context)->Vec<JoinHandle<bool>>{
        context.write(format!("Executing Journey {}",self.name)).await;
        let mut handles = vec![];
        for step in self.steps.iter() {
            handles.append(&mut step.execute(context).await)
        }
        handles
    }
}
pub fn filter(journies:Vec<Journey>,_filter:String)->Vec<Journey>{
    let mut filtered=Vec::new();
    for journey in journies {
        filtered.push(journey);
    }
    return filtered;
}
pub async fn start(journies:&Vec<Journey>,filter_string: String,context:Context) {
    loop {
        let filtered=filter(journies.clone(),filter_string.clone());
        let mut i=0;
        context.write(format!("Choose from below matching journies")).await;
        for journey in filtered.clone() {
            context.write(format!("{})\t{}",i,journey.name)).await;
            i=i+1
        }
        context.write(format!("Please Enter value between 0 to {}",filtered.len()-1)).await;
        let choice=context.read(Variable{
            name:format!("choice"),
            data_type:Option::Some(DataType::PositiveInteger)
        }).await;
        if let Value::PositiveInteger(val) = choice.value.clone(){
            if val < journies.len()  as u128{
                journies.get(val as usize).unwrap().execute(&context).await;
                break;
            } else {
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
    use crate::core::runtime::Context;
    use crate::parser::Parsable;

    #[tokio::test]
    async fn should_start_journey(){
        let text = r#"print text `Hello World <%name:Double%>`;"#;
        let (_,step)=SystemStep::parser(text).unwrap();
        let journes = vec![Journey{ name:"test".to_string(),steps:vec![Step::System(step)] ,params:vec![]}];
        let input = vec![Input::new_continue("choice".to_string(),"0".to_string(),DataType::PositiveInteger),Input::new_continue("name".to_string(),"100.01".to_string(),DataType::Double)];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        start(&journes,"hello".to_string(),context).await;
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_know_that("Choose from below matching journies".to_string()));
        assert_eq!(buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_know_that("0)\ttest".to_string()));
        assert_eq!(buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_know_that("Please Enter value between 0 to 0".to_string()));
        assert_eq!(buffer.lock().unwrap().get(3).unwrap().clone(),Output::new_tell_me("choice".to_string(),DataType::PositiveInteger));
        assert_eq!(buffer.lock().unwrap().get(4).unwrap().clone(),Output::new_know_that("Executing Journey test".to_string()));
        assert_eq!(buffer.lock().unwrap().get(5).unwrap().clone(),Output::new_tell_me("name".to_string(),DataType::Double));
        assert_eq!(buffer.lock().unwrap().get(6).unwrap().clone(),Output::new_know_that("Hello World 100.01".to_string()));

    }
    #[tokio::test]
    async fn should_ask_for_choice_again_journey(){
        let text = r#"print text `Hello World`;"#;
        let (_,step)=SystemStep::parser(text).unwrap();
        let journes = vec![Journey{ name:"test".to_string(),steps:vec![Step::System(step)] ,params:vec![]}];
        let input = vec![Input::new_continue("choice".to_string(),"3".to_string(),DataType::PositiveInteger),Input::new_continue("choice".to_string(),"0".to_string(),DataType::PositiveInteger)];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        start(&journes,"hello".to_string(),context).await;
            assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_know_that("Choose from below matching journies".to_string()));
            assert_eq!(buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_know_that("0)\ttest".to_string()));
            assert_eq!(buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_know_that("Please Enter value between 0 to 0".to_string()));
            assert_eq!(buffer.lock().unwrap().get(3).unwrap().clone(),Output::new_tell_me("choice".to_string(),DataType::PositiveInteger));
            assert_eq!(buffer.lock().unwrap().get(4).unwrap().clone(),Output::new_know_that("Invalid Value".to_string()));
            assert_eq!(buffer.lock().unwrap().get(5).unwrap().clone(),Output::new_know_that("Choose from below matching journies".to_string()));
            assert_eq!(buffer.lock().unwrap().get(6).unwrap().clone(),Output::new_know_that("0)\ttest".to_string()));
            assert_eq!(buffer.lock().unwrap().get(7).unwrap().clone(),Output::new_know_that("Please Enter value between 0 to 0".to_string()));
            assert_eq!(buffer.lock().unwrap().get(8).unwrap().clone(),Output::new_tell_me("choice".to_string(),DataType::PositiveInteger));
            assert_eq!(buffer.lock().unwrap().get(9).unwrap().clone(),Output::new_know_that("Executing Journey test".to_string()));
            assert_eq!(buffer.lock().unwrap().get(10).unwrap().clone(),Output::new_know_that("Hello World".to_string()));


    }
}

