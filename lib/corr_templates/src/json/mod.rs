extern crate corr_core;

use self::corr_core::{Value, VarType, Channel, Variable, Runtime};
use std::cell::{RefMut, RefCell};
use std::rc::Rc;

#[derive(Clone,PartialEq,Debug)]
pub enum Json {
    Value(Value),
    Variable(Variable)
}
pub trait Fillable<T,C> where C:Channel{
    fn fill(&self,runtime:&Runtime<C>)->T;
}
impl<C> Fillable<Value,C> for Value where C:Channel{
    fn fill(&self, runtime: &Runtime<C>) -> Value {
        self.clone()
    }
}
impl<C> Fillable<Json,C> for Json where C:Channel{
    fn fill(&self, runtime:&Runtime<C>) ->Json {
        match self {
            Json::Value(val)=>{
                Json::Value(val.fill(runtime))
            },
            Json::Variable(var)=>{
                let value= runtime.channel.borrow_mut().read(var.clone());
                Json::Value(value)
            }
        }
    }
}
#[cfg(test)]
mod tests{
    use super::corr_core::{Channel, Variable, Value, VarType, Runtime};
    use crate::json::{Json, Fillable};
    use std::cell::{RefMut, RefCell};
    use std::rc::Rc;

    struct MockChannel{
        pointer:usize,
        values:Vec<Value>
    }
    impl Channel for MockChannel{
        fn read(&mut self, variable: Variable) -> Value {
            let val= self.values[self.pointer].clone();
            self.pointer += 1;
            return val;
        }
        fn write(&mut self, text: String) {
            unimplemented!()
        }

        fn close(&mut self) {
            unimplemented!()
        }
    }
    #[test]
    fn should_fill_variable(){
        let val = Json::Variable(Variable{
            name:format!("name"),
            data_type:Option::Some(VarType::String)
        });
        let rc = Rc::new(RefCell::new(MockChannel{
            pointer:0,
            values:vec![Value::Long(5),Value::String(format!("Atmaram"))
                        ,Value::String(format!("Naik"))
                        ,Value::String(format!("Naik"))
                        ,Value::String(format!("Naik"))
                        ,Value::String(format!("Naik"))
                        ,Value::String(format!("Naik"))
            ]
        }));
        let rt=Runtime{
            channel:rc
        };
        rt.iterate(Variable{
            name:format!("hobby"),
            data_type:Option::Some(VarType::String)
        },Variable{
            name:format!("hobbies"),
            data_type:Option::None
        },||{
            println!("Hello");
            val.fill(&rt);
        });
    }

}
