pub mod parser;
use async_trait::async_trait;
use crate::journey::{Executable};
use crate::template::text::{Text, Fillable};
use crate::core::runtime::{Context, IO};

#[derive(Debug, Clone,PartialEq)]
pub enum SystemStep{
    Print(PrintStep),
    // For(Variable,Variable,Box<Step>,Option<Variable>),
    // Collection(Vec<Step>),
    // Assign(String,Expression)
}
#[derive(Debug, Clone,PartialEq)]
pub enum PrintStep{
    WithText(Text)
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
impl Executable for SystemStep{
    async fn execute(&self,context: &Context) {
        match self {
            SystemStep::Print(ps)=>{
                ps.execute(context).await
            },
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
}
