use corr_templates::json::Fillable;
use corr_core::runtime::{Variable, ValueProvider, Environment};
use corr_core::runtime::Value;
use corr_core::runtime::VarType;
use std::fmt::Debug;
use std::rc::Rc;
use corr_templates::json::parser::parse;
use std::marker::PhantomData;

pub struct JourneyStore<T> where T:ValueProvider{
    pub journeys:Vec<Journey<T>>
}
pub trait Interactable<T>  where T:ValueProvider {
    fn start_with(&self,filter:String,runtime:Environment<T>);
}
impl<T> Interactable<T> for JourneyStore<T> where T:ValueProvider{
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
                let selected = self.journeys.get((val-1) as usize).unwrap();
                runtime.write(format!("Executing journey {}",selected.name));
                selected.execute(&runtime);
            }
            _=>{return}
        }
    }
}
pub struct Journey<T> where T:ValueProvider{
    pub name:String,
    pub steps:Vec<Box<Executable<T>>>,
}
pub trait Executable<T> where T:ValueProvider{
    fn execute(&self,runtime:&Environment<T>);
}
impl<T> Executable<T> for Journey<T> where T:ValueProvider{
    fn execute(&self,runtime: &Environment<T>) {
        for step in &self.steps  {
            step.execute(&runtime);
        }
    }
}
#[cfg(test)]
mod tests{
    use corr_core::runtime::Variable;
use corr_core::runtime::{ValueProvider, Value,Environment};
    use crate::{JourneyStore, Journey, Interactable, Executable};
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::marker::PhantomData;

    #[derive(Debug)]
    struct MockChannel;
    impl ValueProvider for MockChannel{
        
        fn read(&mut self, _variable: Variable) -> Value {
            Value::Long(1)
        }


        fn write(&mut self, _text: String) {

        }

        fn close(&mut self) {

        }
        fn set_index_ref(&mut self, _: corr_core::runtime::Variable, _: corr_core::runtime::Variable) { unimplemented!() }
        fn drop(&mut self, _: std::string::String) { unimplemented!() }

        fn load_ith_as(&mut self, i: usize, index_ref_var: Variable, list_ref_var: Variable) {
            unimplemented!()
        }
    }
    #[test]
    fn should_start_journey_store(){
        let js=JourneyStore {
            journeys:vec![Journey{
                name:format!("Hello"),
                steps:vec![Box::new(Post{})],
            }]
        };
        js.start_with(format!("hello"),Environment{ channel:Rc::new(RefCell::new(MockChannel))})
    }
    struct Post;
    impl<T>  Executable<T> for Post where T:ValueProvider{
        fn execute(&self, runtime: &Environment<T>) {
            println!("Hello");
        }
    }
    #[test]
    fn should_execute_journey(){

        let jn=Journey{
                name:format!("Hello"),
                steps:vec![Box::new(Post{})],
            };
        jn.execute(& Environment{ channel:Rc::new(RefCell::new(MockChannel))})
    }
}