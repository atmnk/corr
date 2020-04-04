use corr_templates::Fillable;
use corr_core::runtime::{Variable, ValueProvider, Environment};
use corr_core::runtime::Value;
use corr_core::runtime::VarType;
use corr_templates::text::Text;

pub struct PrintStep{
    pub text:Text
}
impl Executable for PrintStep {
    fn execute(&self, runtime: &Environment)  {
        let val=format!("{}",self.text.fill(runtime).to_string());
        (*runtime.channel).borrow_mut().write(val);
    }
}
pub struct LoopStep{
    pub as_var:Variable,
    pub in_var:Variable,
    pub inner_steps:Vec<Box<dyn Executable>>
}
pub struct TimesStep{
    pub as_var:Variable,
    pub in_var:Variable,
    pub counter_var:Variable,
    pub times:usize,
    pub inner_steps:Vec<Box<dyn Executable>>
}
impl Executable for LoopStep {
    fn execute(&self, runtime: &Environment) {
        runtime.iterate(self.as_var.clone(),self.in_var.clone(),|_i|{
            for step in &self.inner_steps{
                step.execute(runtime);
            }
        })
    }
}
impl Executable for TimesStep {
    fn execute(&self, runtime: &Environment) {
        runtime.build_iterate_outside_building_inside(self.as_var.clone(),self.in_var.clone(),self.times,|i|{
            let val=Value::Long(i as i64);
            (*runtime.channel).borrow_mut().load_value_as(self.counter_var.clone() ,val);
            for step in &self.inner_steps{
                step.execute(runtime);
            }
        });
    }
}
pub struct JourneyStore{
    pub journeys:Vec<Journey>
}
pub trait Interactable{
    fn start_with(&self,filter:String,runtime:Environment) ;
}
impl Interactable for JourneyStore{
    fn start_with(&self, _filter: String, mut runtime: Environment)  {
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
                runtime.close();
            }
            _=>{return}
        }
    }
}
pub struct Journey{
    pub name:String,
    pub steps:Vec<Box<dyn Executable>>,
}
pub trait Executable{
    fn execute(&self,runtime:&Environment) ;
}
impl Executable for Journey {
    fn execute(&self,runtime: &Environment) {
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

        fn load_ith_as(&mut self, _i: usize, _index_ref_var: Variable, _list_ref_var: Variable) {
            unimplemented!()
        }

        fn save(&self, _var: Variable, _value: Value) {
            unimplemented!()
        }

        fn load_value_as(&mut self, _ref_var: Variable, _val: Value) {
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
        js.start_with(format!("hello"),Environment::new_rc(MockChannel))
    }
    struct Post;
    impl  Executable for Post where{
        fn execute(&self, _runtime: &Environment) {
            println!("Hello");
        }
    }
    #[test]
    fn should_execute_journey(){

        let jn=Journey{
                name:format!("Hello"),
                steps:vec![Box::new(Post{})],
            };
        jn.execute(& Environment::new_rc(MockChannel))
    }
}