pub mod parser;
use async_trait::async_trait;
use crate::journey::{Executable};
use crate::template::text::{Text, Fillable};
use crate::core::runtime::{Context, IO};
use crate::core::{Value};
use crate::template::VariableReferenceName;
use crate::journey::step::Step;

#[derive(Debug, Clone,PartialEq)]
pub enum SystemStep{
    Print(PrintStep),
    ForLoop(ForLoopStep),
    // For(Variable,Variable,Box<Step>,Option<Variable>),
    // Collection(Vec<Step>),
    // Assign(String,Expression)
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
                    context
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
            }
            // SystemStep::Collection(steps)=>{
            //     for step in steps {
            //         step.execute(context).await
            //     }
            // }
            // SystemStep::For(temp,on,inner,index_var)=>{
            //     context.iterate(on.name.clone(),temp.name.clone(),async move |context,i|{
            //         if let Some(iv)=index_var.clone(){
            //             context.define(iv.name,Value::PositiveInteger(i)).await
            //         }
            //         inner.execute(&context).await;
            //         context
            //     }).await;
            //
            // },
            // SystemStep::Assign(var,expr)=>{
            //     context.define(var.clone(),expr.evaluate(context).await).await;
            // }
        }

    }
}
#[cfg(test)]
mod tests{
    use crate::core::{ DataType};
    use crate::core::proto::{Input, Output};
    use crate::journey::step::system::SystemStep;
    use std::sync::{Arc, Mutex};
    use crate::journey::{ Executable};
    use crate::core::runtime::Context;
    use crate::parser::Parsable;

    #[tokio::test]
    async fn should_execute_print_step(){
        let text = r#"print fillable text `Hello World`;"#;
        let (_,step)=SystemStep::parser(text).unwrap();
        let input = vec![Input::new_continue("choice".to_string(),"0".to_string(),DataType::PositiveInteger)];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        step.execute(&context).await;
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_know_that("Hello World".to_string()));

    }
    #[tokio::test]
    async fn should_execute_for_step(){
        let text = r#"names . for(name,i)=>{print fillable text `Hello <%i%>-<%name%>`;
            print fillable text `Next`;
        }"#;
        let (_,step)=SystemStep::parser(text).unwrap();
        let input = vec![
            Input::new_continue("names::length".to_string(),"2".to_string(),DataType::PositiveInteger),
            Input::new_continue("name".to_string(),"Atmaram".to_string(),DataType::String),
            Input::new_continue("name".to_string(),"Yogesh".to_string(),DataType::String),
        ];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context= Context::mock(input,buffer.clone());
        step.execute(&context).await;
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_tell_me("names::length".to_string(),DataType::PositiveInteger));
        assert_eq!(buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_tell_me("name".to_string(),DataType::String));
        assert_eq!(buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_know_that("Hello 0-Atmaram".to_string()));
        assert_eq!(buffer.lock().unwrap().get(3).unwrap().clone(),Output::new_know_that("Next".to_string()));

        assert_eq!(buffer.lock().unwrap().get(4).unwrap().clone(),Output::new_tell_me("name".to_string(),DataType::String));
        assert_eq!(buffer.lock().unwrap().get(5).unwrap().clone(),Output::new_know_that("Hello 1-Yogesh".to_string()));
        assert_eq!(buffer.lock().unwrap().get(6).unwrap().clone(),Output::new_know_that("Next".to_string()));

    }
}
