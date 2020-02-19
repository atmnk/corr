extern crate corr_core;
use corr_core::runtime::Variable;
use corr_core::runtime::Value;
use corr_core::runtime::ValueProvider;
use corr_core::runtime::Environment;

#[derive(Clone,PartialEq,Debug)]
pub enum Json {
    Value(Value),
    Variable(Variable)
}
pub trait Fillable<T,C> where C:ValueProvider{
    fn fill(&self,runtime:&Environment<C>)->T;
}
impl<C> Fillable<Value,C> for Value where C:ValueProvider{
    fn fill(&self, _runtime: &Environment<C>) -> Value {
        self.clone()
    }
}
impl<C> Fillable<Json,C> for Json where C:ValueProvider{
    fn fill(&self, runtime:&Environment<C>) ->Json {
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

