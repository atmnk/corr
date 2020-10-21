pub mod parser;
use async_trait::async_trait;
use crate::journey::{Executable};
use crate::template::text::{Text};
use crate::core::runtime::{Context, IO};
use crate::core::{Value};
use crate::template::{VariableReferenceName, Fillable, Assignable};
use crate::journey::step::Step;

#[derive(Debug, Clone,PartialEq)]
pub enum SystemStep{
    Print(PrintStep),
    ForLoop(ForLoopStep),
    Assignment(AssignmentStep),
    Comment(String)

}
#[derive(Debug, Clone,PartialEq)]
pub enum AssignmentStep {
    WithVariableName(VariableReferenceName,Assignable)
}

#[derive(Debug, Clone,PartialEq)]
pub enum PrintStep{
    WithText(Text)
}
#[derive(Debug, Clone,PartialEq)]
pub enum ForLoopStep{
    WithVariableReference(VariableReferenceName,Option<VariableReferenceName>,Option<VariableReferenceName>,Vec<Step>)
}
#[async_trait]
impl Executable for PrintStep{
    async fn execute(&self,context: &Context) {
        match self {
            PrintStep::WithText(txt)=>{
                context.write(txt.fill(context).await).await;
            }
        }

    }
}
#[async_trait]
impl Executable for ForLoopStep{
    async fn execute(&self,context: &Context) {
        match self {
            ForLoopStep::WithVariableReference(on,with,index_var,inner_steps)=>{
                context.iterate(on.to_string(),with.clone().map(|val|{val.to_string()}),async move |context,index|{
                    if let Some(iv)=index_var.clone(){
                        context.define(iv.to_string(),Value::PositiveInteger(index)).await
                    }
                    for step in inner_steps.clone() {
                        step.execute(&context).await;
                    }
                }).await;
                //To Do
            }
        }

    }
}
#[async_trait]
impl Executable for SystemStep{
    async fn execute(&self,context: &Context) {
        match self {
            SystemStep::Print(ps)=>{
                ps.execute(context).await
            },
            SystemStep::ForLoop(fls)=>{
                fls.execute(context).await
            },
            SystemStep::Assignment(asst)=>asst.execute(context).await,
            SystemStep::Comment(_)=>{}
        }

    }
}
#[async_trait]
impl Executable for AssignmentStep {
    async fn execute(&self, context: &Context) {
        match self {
            AssignmentStep::WithVariableName(var, asbl)=>{
                context.define(var.to_string(),asbl.fill(context).await).await;
            }
        }
    }
}
#[cfg(test)]
mod tests{
    use crate::core::{DataType, Variable, Value, VariableValue};
    use crate::core::proto::{Input, Output};
    use crate::journey::step::system::SystemStep;
    use std::sync::{Arc, Mutex};
    use crate::journey::{ Executable};
    use crate::core::runtime::{Context, IO};
    use crate::parser::Parsable;

    #[tokio::test]
    async fn should_execute_print_step(){
        let text = r#"print text `Hello World`;"#;
        let (_,step)=SystemStep::parser(text).unwrap();
        let input = vec![Input::new_continue("choice".to_string(),"0".to_string(),DataType::PositiveInteger)];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        step.execute(&context).await;
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_know_that("Hello World".to_string()));

    }
    #[tokio::test]
    async fn should_execute_assignment_step(){
        let text = r#"let name = concat("Atmaram"," Naik")"#;
        let (_,step)=SystemStep::parser(text).unwrap();
        let input = vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        step.execute(&context).await;
        assert_eq!(context.read(Variable {
            name:format!("name"),
            data_type:Option::Some(DataType::String)
        }).await,VariableValue {
            name:format!("name"),
            value:Value::String("Atmaram Naik".to_string())
        })

    }
    #[tokio::test]
    async fn should_execute_for_step(){
        let text = r#"persons . for(person,i)=>{print text `Hello <%i%>-<%person.name%>`
            print text `Next`
        }"#;
        let (_,step)=SystemStep::parser(text).unwrap();
        let input = vec![
            Input::new_continue("persons::length".to_string(),"2".to_string(),DataType::PositiveInteger),
            Input::new_continue("person.name".to_string(),"Atmaram".to_string(),DataType::String),
            Input::new_continue("person.name".to_string(),"Yogesh".to_string(),DataType::String),
        ];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        step.execute(&context).await;
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_tell_me("persons::length".to_string(),DataType::PositiveInteger));
        assert_eq!(buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_tell_me("person.name".to_string(),DataType::String));
        assert_eq!(buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_know_that("Hello 0-Atmaram".to_string()));
        assert_eq!(buffer.lock().unwrap().get(3).unwrap().clone(),Output::new_know_that("Next".to_string()));

        assert_eq!(buffer.lock().unwrap().get(4).unwrap().clone(),Output::new_tell_me("person.name".to_string(),DataType::String));
        assert_eq!(buffer.lock().unwrap().get(5).unwrap().clone(),Output::new_know_that("Hello 1-Yogesh".to_string()));
        assert_eq!(buffer.lock().unwrap().get(6).unwrap().clone(),Output::new_know_that("Next".to_string()));

    }
}
