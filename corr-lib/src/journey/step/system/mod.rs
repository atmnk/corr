pub mod parser;

use std::time::Duration;
use async_trait::async_trait;
use num_traits::ToPrimitive;
use crate::journey::{Executable};
use crate::template::text::{Text};
use crate::core::runtime::{Context, IO};
use crate::core::{Number, Value};
use crate::template::{VariableReferenceName, Fillable, Assignable, Expression};
use crate::journey::step::Step;
use tokio::task::JoinHandle;
use tokio::fs::{OpenOptions};
use tokio::io::{AsyncWriteExt};
use tokio::time::sleep;

#[derive(Debug, Clone,PartialEq)]
pub enum SystemStep{
    Wait(WaitStep),
    Print(PrintStep),
    ForLoop(ForLoopStep),
    Condition(ConditionalStep),
    Assignment(AssignmentStep),
    Push(PushStep),
    LoadAssign(LoadAssignStep),
    Sync(SyncStep),
    Background(Vec<Step>),
    JourneyStep(JourneyStep)
    // Comment(String)

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
pub enum WaitStep{
    WithTime(Expression)
}
#[derive(Debug, Clone,PartialEq)]
pub struct JourneyStep{
    journey:String,
    args:Vec<Expression>
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
impl Executable for JourneyStep{
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let mut handles = vec![];
        if let Some(journey)= context.journeys.iter().find(|journey|journey.name.eq(&self.journey)){
            let mut i = 0;
            let mut defines = vec![];
            for arg in self.args.clone() {
                if let Some(param) = journey.params.get(i) {
                    context.define(param.name.clone(),arg.evaluate(context).await).await;
                    defines.push(param.name.clone());
                }
                i = i + 1
            }
            handles.append(&mut journey.execute(context).await);
            for var in defines {
                context.undefine(var).await;
            }
        } else {
            context.write(format!("Skipping call to {0} as {0} is not defined in current bundle",self.journey)).await;
        }
        handles
    }
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
impl Executable for WaitStep {
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        match &self {
            WaitStep::WithTime(time_exp)=>{
                let wt = time_exp.evaluate(context).await.to_number().unwrap_or(Number::Integer(128)).as_usize().unwrap();
                sleep(Duration::from_secs(wt.to_u64().unwrap())).await;
                return vec![];
            }
        }
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
            SystemStep::Wait(ws)=>{
                ws.execute(context).await
            },
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
            SystemStep::Background(steps)=>{
                let context = context.clone();
                let steps_to_pass = steps.clone();
                let step = async move ||{
                    let mut handles = vec![];
                    for step in steps_to_pass {
                        let mut inner_handles = step.execute(&context).await;
                        handles.append(&mut inner_handles);
                    }
                    futures::future::join_all(handles).await;
                    true
                };
                vec![tokio::spawn(step())]
            },
            SystemStep::JourneyStep(js)=>{
                js.execute(context).await
            }
            // SystemStep::Comment(_)=>{vec![]}
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
                self.default_value.evaluate(context).await
            }
        } else {
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
        if let Ok(mut file) = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.clone())
            .await {
            if let Some(data) = context.get_var_from_store(self.variable.to_string()).await{
                if let Ok(_)=file.write(data.to_string().as_bytes()).await{
                } else {
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
