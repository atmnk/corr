pub mod parser;



use std::time::{Duration, Instant};
use async_trait::async_trait;
use num_traits::ToPrimitive;
use crate::journey::{Executable};

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
    Exit(ExitStep),
    Print(PrintStep),
    ForLoop(ForLoopStep),
    Condition(ConditionalStep),
    Assignment(AssignmentStep),
    Undefine(VariableReferenceName),
    Push(PushStep),
    LoadAssign(LoadAssignStep),
    Sync(SyncStep),
    Background(Vec<Step>),
    JourneyStep(JourneyStep),
    Transaction(TransactionStep),
    Metric(MetricStep),
    While(WhileStep)
    // Comment(String)

}
#[derive(Debug, Clone,PartialEq)]
pub struct TransactionStep {
    name:Expression,
    block:Vec<Step>,
}
#[derive(Debug, Clone,PartialEq)]
pub struct MetricStep {
    tags:Vec<Expression>,
    value: Expression
}
#[derive(Debug, Clone,PartialEq)]
pub struct WhileStep {
    condition:Expression,
    steps:Vec<Step>,
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
    WithAssignable(Assignable,bool)
}
#[derive(Debug, Clone,PartialEq)]
pub enum WaitStep{
    WithTime(Expression)
}
#[derive(Debug, Clone,PartialEq)]
pub enum ExitStep{
    WithCode(Expression)
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
impl Executable for TransactionStep{
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let name=self.name.evaluate(context).await.to_string();
        let mut handles = vec![];
        let start = Instant::now();
        for step in &self.block {
            handles.append(&mut step.execute(&context).await);
        }
        let duration = start.elapsed();
        context.scrapper.ingest("transaction",duration.as_millis() as f64,vec![("name".to_string(),name.clone().to_string())]).await;
        context.tr_stats_store.push_stat((name,duration.as_millis())).await;
        // context.rest_stats_store.push_stat((req.method,req.url,duration.as_millis())).await;
        handles
    }

    fn get_deps(&self) -> Vec<String> {
        let mut deps = Vec::new();
        for step in &self.block {
            deps.append(&mut step.get_deps());
        }
        deps
    }
}
#[async_trait]
impl Executable for WhileStep{

    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let exp = self.condition.clone();
        let mut handles = vec![];
        let mut continue_loop = exp.evaluate(context).await.to_bool();
        while continue_loop {
            for step in &self.steps {
                handles.append(&mut step.execute(&context).await);
            }
            continue_loop = exp.evaluate(context).await.to_bool();
        }
        handles
    }

    fn get_deps(&self) -> Vec<String> {
        let mut deps = Vec::new();
        for step in &self.steps {
            deps.append(&mut step.get_deps());
        }
        deps
    }
}
#[async_trait]
impl Executable for MetricStep{

    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let mut tags = vec![];
        for tag in &self.tags{
            tags.push(("tag".to_string(),tag.evaluate(context).await.to_string()))
        }
        let val = self.value.evaluate(context).await;
        let metric = match val {
            Value::Double(m)=>{
                m
            },
            Value::PositiveInteger(i)=>i as f64,
            Value::Integer(i)=>i as f64,
            Value::String(s)=>s.parse().unwrap(),
            _=>0.0
        };
        context.scrapper.ingest("metric",metric,tags).await;
        vec![]
    }

    fn get_deps(&self) -> Vec<String> {
        vec![]
    }
}
#[async_trait]
impl Executable for JourneyStep{

    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let mut handles = vec![];
        let mut jn = context.get_local_journey(self.journey.clone()).await;
        if jn.is_none() {
            jn = context.get_global_journey(self.journey.clone());
        }
        if let Some(journey)= jn{
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
            context.write(format!("{} Journey Not Found",self.journey)).await;
            context.exit(-1).await;
        }
        handles
    }

    fn get_deps(&self) -> Vec<String> {
        return vec![self.journey.clone()]
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

    fn get_deps(&self) -> Vec<String> {
        let mut deps = vec![];
        for ip in &self.if_parts {
            for step in &ip.block{
                deps.append(&mut step.get_deps());
            }
        }
        for step in &self.else_part {
            deps.append(&mut step.get_deps());
        }
        deps
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

    fn get_deps(&self) -> Vec<String> {
        vec![]
    }
}
#[async_trait]
impl Executable for PrintStep{

    async fn execute(&self,context: &Context)->Vec<JoinHandle<bool>> {
        match self {
            PrintStep::WithAssignable(asgn,debug)=>{
                let to_print=asgn.fill(context).await.to_string();
                if *debug {
                    if context.debug {
                        context.write(to_print).await;
                    }
                } else {
                    context.write(to_print).await;
                }

            }
        }
        return vec![]
    }

    fn get_deps(&self) -> Vec<String> {
        vec![]
    }
}
#[async_trait]
impl Executable for WaitStep {

    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        match &self {
            WaitStep::WithTime(time_exp)=>{
                let wt = time_exp.evaluate(context).await.to_number().unwrap_or(Number::Integer(128)).as_usize().unwrap();
                sleep(Duration::from_millis(wt.to_u64().unwrap())).await;
                return vec![];
            }
        }
    }

    fn get_deps(&self) -> Vec<String> {
        vec![]
    }
}
#[async_trait]
impl Executable for ExitStep {

    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        match &self {
            ExitStep::WithCode(code)=>{
                let code = code.evaluate(context).await.to_number().unwrap_or(Number::Integer(0)).as_i32().unwrap();
                context.exit(code).await;
                return vec![];
            }
        }
    }

    fn get_deps(&self) -> Vec<String> {
        vec![]
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

    fn get_deps(&self) -> Vec<String> {
        let mut deps = Vec::new();
        match &self {
            ForLoopStep::WithVariableReference(_,_,_,steps)=>{
                for step in steps {
                    deps.append(&mut step.get_deps());
                }
            }
        }
        deps
    }
}
#[async_trait]
impl Executable for SystemStep{

    async fn execute(&self,context: &Context)->Vec<JoinHandle<bool>> {
        match self {
            SystemStep::Undefine(vrn)=>{
                context.delete(vrn.to_string()).await;
                vec![]
            }
            SystemStep::Wait(ws)=>{
                ws.execute(context).await
            },
            SystemStep::Exit(es)=>{
                es.execute(context).await
            },
            SystemStep::Print(ps)=>{
                ps.execute(context).await
            },
            SystemStep::ForLoop(fls)=>{
                fls.execute(context).await
            },
            SystemStep::While(wl)=>{
                wl.execute(context).await
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
            SystemStep::Transaction(tr)=>tr.execute(context).await,
            SystemStep::Metric(ms)=>ms.execute(context).await,
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

    fn get_deps(&self) -> Vec<String> {
        match self {
            SystemStep::Undefine(_vrn)=>{
                vec![]
            }
            SystemStep::Wait(ws)=>{
                ws.get_deps()
            },
            SystemStep::Exit(es)=>{
                es.get_deps()
            },
            SystemStep::Print(ps)=>{
                ps.get_deps()
            },
            SystemStep::ForLoop(fls)=>{
                fls.get_deps()
            },
            SystemStep::While(wl)=>{
                wl.get_deps()
            },
            SystemStep::Push(pt)=>{
                pt.get_deps()
            },
            SystemStep::Condition(pt)=>{
                pt.get_deps()
            },
            SystemStep::LoadAssign(pt)=>{
                pt.get_deps()
            },
            SystemStep::Sync(pt)=>{
                pt.get_deps()
            }
            SystemStep::Assignment(asst)=>asst.get_deps(),
            SystemStep::Transaction(tr)=>tr.get_deps(),
            SystemStep::Metric(ms)=>ms.get_deps(),
            SystemStep::Background(steps)=>{
                let mut deps = Vec::new();
                for step in steps {
                    deps.append(&mut step.get_deps());
                }
                deps
            },
            SystemStep::JourneyStep(js)=>{
                js.get_deps()
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

    fn get_deps(&self) -> Vec<String> {
        vec![]
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

    fn get_deps(&self) -> Vec<String> {
        vec![]
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

    fn get_deps(&self) -> Vec<String> {
        vec![]
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
