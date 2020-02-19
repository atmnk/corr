use corr_templates::json::Fillable;
use corr_core::runtime::{Variable, ValueProvider, Environment};
use corr_core::runtime::Value;
use corr_core::runtime::VarType;


#[derive(Debug,PartialEq,Clone)]
pub struct JourneyStore{
    pub journeys:Vec<Journey>
}
pub trait Interactable<T>  where T:ValueProvider {
    fn start_with(&self,filter:String,runtime:Environment<T>);
}
impl<T> Interactable<T> for JourneyStore where T:ValueProvider{
    fn start_with(&self, _filter: String, mut runtime: Environment<T>) {
        runtime.write(format!("Choose from following"));
        let mut counter = 1;
        for journey in &self.journeys{
            runtime.write(format!("{}. {}",counter,journey.name));
            counter+=1;
        }
        runtime.write(format!("Enter your choice"));
        let num = runtime.read(Variable{
            name:format!("choice"),
            data_type:Option::Some(VarType::Long)
        });
        match num {
            Value::Long(val)=>{
                let selected = self.journeys[(val-1) as usize].clone();
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
pub trait Executable<T> where T:ValueProvider{
    fn execute(&self,runtime:Environment<T>);
}
impl<T> Executable<T> for Journey where T:ValueProvider{
    fn execute(&self,runtime: Environment<T>) {
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
    use corr_core::runtime::Variable;
use corr_core::runtime::{ValueProvider, Value,Environment};
    use crate::{JourneyStore, Journey, Interactable, Executable};
    use std::rc::Rc;
    use std::cell::RefCell;

    struct MockChannel;
    impl ValueProvider for MockChannel{

        fn read(&mut self, _variable: Variable) -> Value {
            Value::Long(1)
        }


        fn write(&mut self, _text: String) {

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
        js.start_with(format!("hello"),Environment{ channel:Rc::new(RefCell::new(MockChannel))})
    }
    #[test]
    fn should_execute_journey(){
        let jn=Journey{
                name:format!("Hello")
            };
        jn.execute(Environment{ channel:Rc::new(RefCell::new(MockChannel))})
    }
}