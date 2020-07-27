pub mod step;
use serde::{Deserialize, Serialize};
use crate::journey::step::Step;
use crate::core::{Context, IO, Variable, DataType, Value};
use async_trait::async_trait;

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
pub async fn start(journies:&Vec<Journey>,_filter: String,context:Context) {
    loop {
        context.write(format!("Please Enter value between 0 to {}",journies.len()-1)).await;
        let choice=context.read(Variable{
            name:format!("choice"),
            data_type:DataType::Long
        }).await;
        if let Value::Long(val) = choice.value.clone(){
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
    use crate::core::{Context, ReferenceStore, DataType};
    use crate::core::proto::{Input, Output};
    use crate::journey::step::system::SystemStep;
    use std::sync::{Arc};
    use crate::journey::{Journey, start};
    use crate::journey::step::Step;
    use crate::core::tests::MockClient;

    #[tokio::test]
    async fn should_start_journey(){
        let step=SystemStep::Print;
        let journes = vec![Journey{ name:"test".to_string(),steps:vec![Step::System(step)] }];
        let user = Arc::new(     futures::lock::Mutex::new(MockClient::new(vec![Input::new_continue("choice".to_string(),"0".to_string(),DataType::Long)])));
        let context= Context {
            user: user.clone(),
            store:ReferenceStore::new()
        };
        start(&journes,"hello".to_string(),context).await;
        assert_eq!(user.lock().await.buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_know_that("Please Enter value between 0 to 0".to_string()));
        assert_eq!(user.lock().await.buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_tell_me("choice".to_string(),DataType::Long));
        assert_eq!(user.lock().await.buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_know_that("Executing Journey test".to_string()));
        assert_eq!(user.lock().await.buffer.lock().unwrap().get(3).unwrap().clone(),Output::new_know_that("Hello World".to_string()));

    }
    #[tokio::test]
    async fn should_ask_for_choice_again_journey(){
        let step=SystemStep::Print;
        let journes = vec![Journey{ name:"test".to_string(),steps:vec![Step::System(step)] }];
        let user= Arc::new(futures::lock::Mutex::new(MockClient::new(vec![Input::new_continue("choice".to_string(),"3".to_string(),DataType::Long),Input::new_continue("choice".to_string(),"0".to_string(),DataType::Long)])));
        let context= Context {
            user: user.clone(),
            store:ReferenceStore::new()
        };
        start(&journes,"hello".to_string(),context).await;
            assert_eq!(user.lock().await.buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_know_that("Please Enter value between 0 to 0".to_string()));
            assert_eq!(user.lock().await.buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_tell_me("choice".to_string(),DataType::Long));
            assert_eq!(user.lock().await.buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_know_that("Invalid Value".to_string()));
            assert_eq!(user.lock().await.buffer.lock().unwrap().get(3).unwrap().clone(),Output::new_know_that("Please Enter value between 0 to 0".to_string()));
            assert_eq!(user.lock().await.buffer.lock().unwrap().get(4).unwrap().clone(),Output::new_tell_me("choice".to_string(),DataType::Long));
            assert_eq!(user.lock().await.buffer.lock().unwrap().get(5).unwrap().clone(),Output::new_know_that("Executing Journey test".to_string()));
            assert_eq!(user.lock().await.buffer.lock().unwrap().get(6).unwrap().clone(),Output::new_know_that("Hello World".to_string()));


    }
}

