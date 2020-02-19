use std::rc::Rc;
use std::cell::RefCell;
use corr_core::{Channel, Variable, VarType, Value, Runtime};
use corr_templates::json::Fillable;
use std::borrow::BorrowMut;
#[derive(Debug,PartialEq,Clone)]
pub struct JourneyStore{
    pub journeys:Vec<Journey>
}
pub trait Interactable<T>  where T:Channel {
    fn start_with(&self,filter:String,runtime:Runtime<T>);
}
impl<T> Interactable<T> for JourneyStore where T:Channel{
    fn start_with(&self, filter: String, mut runtime: Runtime<T>) {
        runtime.write(format!("Choose from following"));
        let mut counter = 1;
        for journey in &self.journeys{
            runtime.write(format!("{}. {}",counter,journey.name));
            counter+=1;
        }
        runtime.write(format!("Enter your choice"));
        let mut num = runtime.read(Variable{
            name:format!("choice"),
            data_type:Option::Some(VarType::Long)
        });
        match num {
            Value::Long(val)=>{
                let mut selected = self.journeys[(val-1) as usize].clone();
                selected.execute(runtime);
            }
            _=>{return}
        }
    }
}
#[derive(Debug,PartialEq,Clone)]
pub struct Journey{
    pub name:String
}
pub trait Executable<T> where T:Channel{
    fn execute(&self,runtime:Runtime<T>);
}
impl<T> Executable<T> for Journey where T:Channel{
    fn execute(&self,runtime: Runtime<T>) {
        let tmp=corr_templates::json::Json::Variable(Variable{
            name:format!("name {:?}",self.name),
            data_type:Option::Some(VarType::String)
        });
        tmp.fill(&runtime);
        (*runtime.channel).borrow_mut().close();
    }
}
#[cfg(test)]
mod tests{
    use corr_core::{Channel, Variable, Value, Runtime};
    use crate::{JourneyStore, Journey, Interactable, Executable};
    use std::rc::Rc;
    use std::cell::RefCell;

    struct MockChannel;
    impl Channel for MockChannel{

        fn read(&mut self, variable: Variable) -> Value {
            Value::Long(1)
        }


        fn write(&mut self, text: String) {

        }

        fn close(&mut self) {

        }
    }
    #[test]
    fn should_start_journey_store(){
        let js=JourneyStore {
            journeys:vec![Journey{
                name:format!("Hello")
            }]
        };
        js.start_with(format!("hello"),Runtime{ channel:Rc::new(RefCell::new(MockChannel))})
    }
    #[test]
    fn should_execute_journey(){
        let jn=Journey{
                name:format!("Hello")
            };
        jn.execute(Runtime{ channel:Rc::new(RefCell::new(MockChannel))})
    }
}