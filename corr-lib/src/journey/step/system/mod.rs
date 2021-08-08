pub mod parser;
use async_trait::async_trait;
use crate::journey::{Executable};
use crate::template::text::{Text};
use crate::core::runtime::{Context, IO};
use crate::core::{Value};
use crate::template::{VariableReferenceName, Fillable, Assignable, Expression};
use crate::journey::step::Step;
use tokio::task::JoinHandle;
use tokio::fs::{OpenOptions};
use tokio::io::{AsyncWriteExt};

#[derive(Debug, Clone,PartialEq)]
pub enum SystemStep{
    Print(PrintStep),
    ForLoop(ForLoopStep),
    Condition(ConditionalStep),
    Assignment(AssignmentStep),
    Push(PushStep),
    LoadAssign(LoadAssignStep),
    Sync(SyncStep),
    Comment(String)

}
#[derive(Debug, Clone,PartialEq)]
pub enum AssignmentStep {
    WithVariableName(VariableReferenceName,Assignable)
}
#[derive(Debug, Clone,PartialEq)]
pub enum PushStep {
    WithVariableName(VariableReferenceName,Assignable)
}

#[derive(Debug, Clone,PartialEq)]
pub enum PrintStep{
    WithText(Text)
}
#[derive(Debug, Clone,PartialEq)]
pub struct IfPart{
    condition:Expression,
    block:Vec<Step>
}
#[derive(Debug, Clone,PartialEq)]
pub struct ConditionalStep{
    if_parts:Vec<IfPart>,
    else_part:Vec<Step>
}
#[derive(Debug, Clone,PartialEq)]
pub struct SyncStep{
    sandbox:Option<Expression>,
    variable:VariableReferenceName
}
#[derive(Debug, Clone,PartialEq)]
pub struct LoadAssignStep{
    sandbox:Option<Expression>,
    variable:VariableReferenceName,
    default_value:Expression
}
#[derive(Debug, Clone,PartialEq)]
pub enum ForLoopStep{
    WithVariableReference(VariableReferenceName,Option<VariableReferenceName>,Option<VariableReferenceName>,Vec<Step>)
}
#[async_trait]
impl Executable for ConditionalStep{
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let mut handles = vec![];
        let mut found = false;
        for ip in &self.if_parts {
            if ip.condition.evaluate(context).await.parse::<bool>().unwrap_or(false) {
                found = true;
                for step in &ip.block{
                    handles.append(&mut step.execute(context).await)
                }
                break;
            }
        }
        if !found {
            for step in &self.else_part{
                handles.append(&mut step.execute(context).await)
            }
        }
        handles
    }
}
#[async_trait]
impl Executable for PushStep{
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        match self {
            PushStep::WithVariableName(var, asbl)=>{
                context.push(var.to_string(),asbl.fill(context).await).await;
            }
        }
        return vec![]
    }
}
#[async_trait]
impl Executable for PrintStep{
    async fn execute(&self,context: &Context)->Vec<JoinHandle<bool>> {
        match self {
            PrintStep::WithText(txt)=>{
                context.write(txt.fill(context).await).await;
            }
        }
        return vec![]
    }
}
#[async_trait]
impl Executable for ForLoopStep{
    async fn execute(&self,context: &Context)->Vec<JoinHandle<bool>> {
        match self {
            ForLoopStep::WithVariableReference(on,with,index_var,inner_steps)=>{
                let outer_handles:Vec<Vec<JoinHandle<bool>>> = context.iterate(on.to_string(),with.clone().map(|val|{val.to_string()}),async move |context,index|{
                    let mut handles = vec![];
                    if let Some(iv)=index_var.clone(){
                        context.define(iv.to_string(),Value::PositiveInteger(index as u128)).await
                    }
                    for step in inner_steps.clone() {
                        handles.append(&mut step.execute(&context).await);
                    }
                    handles
                }).await;
                outer_handles.into_iter().flatten().collect()
                //To Do
            }
        }

    }
}
#[async_trait]
impl Executable for SystemStep{
    async fn execute(&self,context: &Context)->Vec<JoinHandle<bool>> {
        match self {
            SystemStep::Print(ps)=>{
                ps.execute(context).await
            },
            SystemStep::ForLoop(fls)=>{
                fls.execute(context).await
            },
            SystemStep::Push(pt)=>{
                pt.execute(context).await
            },
            SystemStep::Condition(pt)=>{
                pt.execute(context).await
            },
            SystemStep::LoadAssign(pt)=>{
                pt.execute(context).await
            },
            SystemStep::Sync(pt)=>{
                pt.execute(context).await
            }
            SystemStep::Assignment(asst)=>asst.execute(context).await,
            SystemStep::Comment(_)=>{vec![]}
        }

    }
}

#[async_trait]
impl Executable for AssignmentStep {
    async fn execute(&self, context: &Context)->Vec<JoinHandle<bool>> {
        match self {
            AssignmentStep::WithVariableName(var, asbl)=>{
                context.define(var.to_string(),asbl.fill(context).await).await;
            }
        }
        return vec![]
    }
}
#[async_trait]
impl Executable for LoadAssignStep {
    async fn execute(&self, context: &Context)->Vec<JoinHandle<bool>> {
        let dir:String = if let Some(sb)=&self.sandbox{
            sb.evaluate(context).await.to_string()
        } else {
            format!("data")
        };
        let path = format!("./{0}/{1}.json",dir,self.variable.to_string());
        let val = if let Ok(data) = tokio::fs::read(path).await{
            let file_contents= serde_json::from_str(String::from_utf8_lossy(&data).as_ref());
            // Read the JSON contents of the file as an instance of `User`.
            if let Ok(value) = file_contents{
                Value::from_json_value(value)
            } else {
                println!("Shit 1");
                self.default_value.evaluate(context).await
            }

        } else {
            println!("Shit 2");
            self.default_value.evaluate(context).await
        };
        context.define(self.variable.to_string(),val).await;
        return vec![]
    }
}
#[async_trait]
impl Executable for SyncStep {
    async fn execute(&self, context: &Context)->Vec<JoinHandle<bool>> {
        let dir:String = if let Some(sb)=&self.sandbox{
            sb.evaluate(context).await.to_string()
        } else {
            format!("data")
        };
        let path = format!("./{0}/{1}.json",dir,self.variable.to_string());
        std::fs::create_dir_all(dir.clone()).unwrap();
        println!("Wrote to file{0}",path);
        if let Ok(mut file) = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.clone())
            .await {
            if let Some(data) = context.get_var_from_store(self.variable.to_string()).await{
                if let Ok(_)=file.write(data.to_string().as_bytes()).await{
                    println!("Wrote to file: {0}",path);
                } else {
                    eprintln!("Failed to write to file: {0}",path)
                }

            }

        }
        return vec![]
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
