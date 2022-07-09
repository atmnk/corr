pub mod step;
pub mod parser;

use std::collections::HashMap;
use std::sync::Arc;
use crate::journey::step::Step;
use crate::core::{runtime::Context, runtime::IO, Variable, DataType, Value};
use async_trait::async_trait;
use tokio::task::JoinHandle;
use crate::template::VariableReferenceName;

#[derive(Debug, Clone,PartialEq)]
pub struct Journey{
    pub import_statements:Vec<ImportStatement>,
    pub name:String,
    pub steps:Vec<Step>,
    pub params:Vec<Variable>
}

impl Journey {
    pub fn matching_import(&self,reference:String)->Option<String>{
        for is in &self.import_statements {
            if reference.eq(&is.logical_name.to_string()) {
                return Some(is.physical_name.to_string())
            }
        }
        return None
    }

}
#[derive(Debug, Clone,PartialEq)]
pub struct ImportStatement{
    pub physical_name:VariableReferenceName,
    pub logical_name:VariableReferenceName,
}
#[async_trait]
pub trait Executable{
    async fn execute(&self,context:&Context)->Vec<JoinHandle<bool>>;
    fn get_deps(&self)->Vec<String>;
}



#[async_trait]
impl Executable for Journey{
    async fn execute(&self, context: &Context)->Vec<JoinHandle<bool>>{
        for is in &self.import_statements{
            if let Some(jn)=context.get_global_journey(is.physical_name.to_string()){
                let mut hm = context.local_program_lookup.write().await;
                (*hm).insert(is.logical_name.to_string(),jn);
            } else {
                context.write(format!("Journey {} not loaded in bunle",is.physical_name.to_string())).await;
                context.exit(-1).await;
            }
        }
        // context.write(format!("Executing Journey {}",self.name)).await;
        let mut handles = vec![];
        for step in self.steps.iter() {
            handles.append(&mut step.execute(context).await)
        }
        handles
    }

    fn get_deps(&self) -> Vec<String> {
        let mut deps = Vec::new();
        for step in &self.steps {
            deps.append(&mut step.get_deps());
        };
        let mut ads =vec![];
        for dep in deps {
          if let Some(ad)=self.matching_import(dep.clone())  {
              ads.push(ad)
          } else {
              ads.push(dep.clone())
          }
        }
        ads
    }
}
pub fn filter(journies:HashMap<String,Arc<Journey>>,_filter:String)->HashMap<String,Arc<Journey>>{
    journies
}
pub async fn start(journies:&HashMap<String,Arc<Journey>>,filter_string: String,context:Context) {
    loop {
        let filtered=filter(journies.clone(),filter_string.clone());
        let mut i=0;
        context.write(format!("Choose from below matching journies")).await;
        let mut arr = vec![];
        for journey in filtered.clone() {
            context.write(format!("{})\t{}",i,journey.0)).await;
            arr.push(journey.1.clone());
            i=i+1
        }
        context.write(format!("Please Enter value between 0 to {}",filtered.len()-1)).await;
        let choice=context.read(Variable{
            name:format!("choice"),
            data_type:Option::Some(DataType::PositiveInteger)
        }).await;
        if let Value::PositiveInteger(val) = choice.value.clone(){
            if val < journies.len()  as u128{
                arr.get(val as usize).unwrap().execute(&context).await;
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
    use std::collections::HashMap;
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
        let mut journes = HashMap::new();
        journes.insert("test".to_string(),Arc::new(Journey{ import_statements:vec![],name:"test".to_string(),steps:vec![Step::System(step)] ,params:vec![]}));
        let input = vec![Input::new_continue("choice".to_string(),"0".to_string(),DataType::PositiveInteger),Input::new_continue("name".to_string(),"100.01".to_string(),DataType::Double)];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        start(&journes,"hello".to_string(),context).await;
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_know_that("Choose from below matching journies".to_string()));
        assert_eq!(buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_know_that("0)\ttest".to_string()));
        assert_eq!(buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_know_that("Please Enter value between 0 to 0".to_string()));
        assert_eq!(buffer.lock().unwrap().get(3).unwrap().clone(),Output::new_tell_me("choice".to_string(),DataType::PositiveInteger));
        assert_eq!(buffer.lock().unwrap().get(4).unwrap().clone(),Output::new_tell_me("name".to_string(),DataType::Double));
        assert_eq!(buffer.lock().unwrap().get(5).unwrap().clone(),Output::new_know_that("Hello World 100.01".to_string()));

    }
    #[tokio::test]
    async fn should_ask_for_choice_again_journey(){
        let text = r#"print text `Hello World`;"#;
        let (_,step)=SystemStep::parser(text).unwrap();
        let mut journes = HashMap::new();
        journes.insert("test".to_string(),Arc::new(Journey{ import_statements:vec![],name:"test".to_string(),steps:vec![Step::System(step)] ,params:vec![]}));
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
            assert_eq!(buffer.lock().unwrap().get(9).unwrap().clone(),Output::new_know_that("Hello World".to_string()));


    }
}

