pub mod parser;
use crate::journey::step::Step;
use async_trait::async_trait;
use crate::journey::{Executable};
use crate::core::{Variable, Value};
use crate::template::text::{Text, Fillable};
use crate::core::runtime::{Context, IO};
use crate::template::Expression;

#[derive(Debug, Clone,PartialEq)]
pub enum SystemStep{
    Print(PrintStep),
    For(Variable,Variable,Box<Step>,Option<Variable>),
    Collection(Vec<Step>),
    Assign(String,Expression)
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
            SystemStep::Collection(steps)=>{
                for step in steps {
                    step.execute(context).await
                }
            }
            SystemStep::For(temp,on,inner,index_var)=>{
                context.iterate(on.name.clone(),temp.name.clone(),async move |context,i|{
                    if let Some(iv)=index_var.clone(){
                        context.define(iv.name,Value::PositiveInteger(i)).await
                    }
                    inner.execute(&context).await;
                    context
                }).await;

            },
            SystemStep::Assign(var,expr)=>{
                context.define(var.clone(),expr.evaluate(context).await).await;
            }
        }

    }
}
#[cfg(test)]
mod tests{
    use crate::journey::step::system::SystemStep;
    use std::sync::Arc;
    use futures::lock::Mutex;
    use crate::core::proto::{Input, Output};
    use async_trait::async_trait;
    use crate::core::{ Variable, DataType, Value};
    use crate::journey::Executable;
    use crate::journey::step::Step;
    use crate::template::text::{Text, Block};
    use crate::core::runtime::{Client, Context, HeapObject};
    use crate::parser::Parsable;

    static mut MESSAGES:Vec<String> = vec![];

    struct DummyUser;

    impl DummyUser{
        pub fn new()->Self{
            return DummyUser{};
        }
    }

    #[async_trait]
    impl Client for DummyUser{
        fn send(&self, output: Output) {
            if let Output::KnowThat(kto)=output{
                unsafe {
                    MESSAGES.push(kto.message);
                }
            }
        }

        async fn get_message(&mut self) -> Input {
            unimplemented!()
        }
    }
    #[tokio::test]
    async fn should_execute_system_step_print(){
        let text = r#"print fillable text `Hello World`;"#;
        let (i,step)=SystemStep::parser(text).unwrap();
        let context= Context::new(Arc::new(Mutex::new(DummyUser::new())));
        step.execute(&context).await;
        unsafe {
            assert_eq!(MESSAGES.get(0).unwrap(),"Hello World")
        }

    }
    // #[tokio::test]
    // async fn should_execute_system_step_for(){
    //     let step=SystemStep::For(Variable{
    //         name:"temp".to_string(),
    //         data_type:Option::Some(DataType::PositiveInteger),
    //     },
    //      Variable{
    //          name:"on".to_string(),
    //          data_type:Option::Some(DataType::String),
    //      },
    //         Box::new(Step::System(SystemStep::Print(Text{
    //         blocks:vec![Block::Final("Hello World".to_string())]
    //     }))),
    //         Option::None
    //     );
    //     let context= Context::new(Arc::new(Mutex::new(DummyUser::new())));
    //     context.store.set("on".to_string(),Arc::new(Mutex::new(HeapObject::List(vec![Arc::new(Mutex::new(HeapObject::Final(Value::PositiveInteger(1)))),Arc::new(Mutex::new(HeapObject::Final(Value::PositiveInteger(2))))])))).await;
    //     step.execute(&context).await;
    //     unsafe {
    //         assert_eq!(MESSAGES.get(0).unwrap(),"Hello World");
    //         assert_eq!(MESSAGES.get(1).unwrap(),"Hello World");
    //     }
    //
    // }
}
