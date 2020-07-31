use crate::journey::step::Step;
use async_trait::async_trait;
use crate::journey::{Executable};
use crate::core::{IO, Context, Variable};
use crate::template::text::{Text, Fillable};

#[derive(Debug, Clone)]
pub enum SystemStep{
    Print(Text),
    For(Variable,Variable,Box<Step>),
    Collection(Vec<Step>)
}
#[async_trait]
impl Executable for SystemStep{
    async fn execute(&self,context: &Context) {
        match self {
            SystemStep::Print(txt)=>{
                context.write(txt.fill(context).await).await;
            },
            SystemStep::Collection(steps)=>{
                for step in steps {
                    step.execute(context).await
                }
            }
            SystemStep::For(temp,on,inner)=>{
                context.iterate(on.name.clone(),temp.name.clone(),async move |context|{
                    inner.execute(&context).await;
                    context
                }).await;

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
    use crate::core::{Client, Context, Variable, DataType, HeapObject, Value};
    use crate::journey::Executable;
    use crate::journey::step::Step;
    use crate::template::text::{Text, Block};

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
        let step=SystemStep::Print(Text{
            blocks:vec![Block::Final("Hello World".to_string())]
        });
        let context= Context::new(Arc::new(Mutex::new(DummyUser::new())));
        step.execute(&context).await;
        unsafe {
            assert_eq!(MESSAGES.get(0).unwrap(),"Hello World")
        }

    }
    #[tokio::test]
    async fn should_execute_system_step_for(){
        let step=SystemStep::For(Variable{
            name:"temp".to_string(),
            data_type:Option::Some(DataType::Long),
        },
         Variable{
             name:"on".to_string(),
             data_type:Option::Some(DataType::String),
         },
            Box::new(Step::System(SystemStep::Print(Text{
            blocks:vec![Block::Final("Hello World".to_string())]
        }))));
        let context= Context::new(Arc::new(Mutex::new(DummyUser::new())));
        context.store.set("on".to_string(),Arc::new(Mutex::new(HeapObject::List(vec![Arc::new(Mutex::new(HeapObject::Final(Value::Long(1)))),Arc::new(Mutex::new(HeapObject::Final(Value::Long(2))))])))).await;
        step.execute(&context).await;
        unsafe {
            assert_eq!(MESSAGES.get(0).unwrap(),"Hello World");
            assert_eq!(MESSAGES.get(1).unwrap(),"Hello World");
        }

    }
}
