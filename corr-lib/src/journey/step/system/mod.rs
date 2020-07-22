use crate::journey::step::Step;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::journey::{Executable};
use crate::core::{IO, Context, Variable};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemStep{
    Print,
    For(Variable,Variable,Box<SystemStep>),
    Collection(Vec<Step>)
}
#[async_trait]
impl Executable for SystemStep{
    async fn execute(&self,context: &Context) {
        match self {
            SystemStep::Print=>{
                context.write(format!("Hello World")).await;
            },
            SystemStep::Collection(steps)=>{
                for step in steps {
                    step.execute(context).await
                }
            }
            SystemStep::For(temp,on,inner)=>{
                context.iterate(on.name.clone(),temp.name.clone(),async move |context|{
                    inner.execute(&context).await
                }).await

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
    use crate::core::{Client, Context, ReferenceStore};
    use crate::journey::Executable;

    struct DummyUser;

    impl DummyUser{
        pub fn new()->Self{
            return DummyUser{};
        }
    }


    #[tokio::test]
    async fn should_execute_system_step(){
        static mut message:Vec<String> = vec![];
        #[async_trait]
        impl Client for DummyUser{
            fn send(&self, output: Output) {
                if let Output::KnowThat(kto)=output{
                    unsafe {
                        message.push(kto.message);
                    }
                }
            }

            async fn get_message(&mut self) -> Input {
                unimplemented!()
            }
        }
        let step=SystemStep::Print;
        let context= Context {
            user:Arc::new(Mutex::new(DummyUser::new())),
            store:ReferenceStore::new()
        };
        step.execute(&context).await;
        unsafe {
            assert_eq!(message.get(0).unwrap(),"Hello World")
        }

    }
}
