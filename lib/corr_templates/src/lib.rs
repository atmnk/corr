pub mod json;
pub mod text;
pub mod parser;
extern crate rand;
#[macro_use]
extern crate nom;
use corr_core::runtime::{Value, Environment, ValueProvider, Variable};
use uuid::Uuid;
use rand::Rng;
pub trait Func{
    fn eval(&self,args:Vec<Value>)->Value;
}
pub struct Round;
pub struct Random;
pub struct UUID;
pub struct Concat;
pub struct Multiply;
pub struct Add;
impl Func for Multiply{
    fn eval(&self,args: Vec<Value>)->Value {
        let mut res=1.0;
        let mut double=false;
        for val in args {
            match val {
                Value::Long(l)=>{
                    res=res*l as f64;
                },
                Value::Double(d)=>{
                    double=true;
                    res=res*d;
                },
                _=>{continue}
            }
        }
        if double{
            return Value::Double(res)
        }
        return Value::Long(res as i64)

    }
}
impl Func for Add{
    fn eval(&self,args: Vec<Value>)->Value {
        let mut res=1.0;
        let mut double=false;
        for val in args {
            match val {
                Value::Long(l)=>{
                    res=res+l as f64;
                },
                Value::Double(d)=>{
                    double=true;
                    res=res+d;
                },
                _=>{continue}
            }
        }
        if double {
            return Value::Double(res)
        }
        return Value::Long(res as i64)

    }
}
impl Func for Concat{
    fn eval(&self,args: Vec<Value>)->Value {
        Value::String(Value::Array(args).to_string())
    }
}
impl Func for UUID{
    fn eval(&self, _args: Vec<Value>) -> Value {
        return Value::String(Uuid::new_v4().to_string());
    }
}
impl Func for Random{
    fn eval(&self, args: Vec<Value>) -> Value {
        let val1=args.get(0).unwrap();
        let val2=args.get(1).unwrap();
        let mut rng = rand::thread_rng();
        match val1{
            Value::Long(l1)=>{
                match val2 {
                    Value::Long(l2)=>{
                        Value::Long(rng.gen_range(l1,l2))
                    },
                    _=>{
                        unimplemented!()
                    }
                }
            },
            Value::Double(d1)=>{
                match val2 {
                    Value::Double(d2)=>{
                        Value::Double(rng.gen_range(d1,d2))
                    },
                    _=>{unimplemented!()}
                }
            },
            _=>{unimplemented!()}
        }
    }
}
impl Func for Round{
    fn eval(&self, args: Vec<Value>) -> Value {
        let val1=args.get(0).unwrap();
        let val2=args.get(1).unwrap();
        match val1{
            Value::Double(d)=>{
                match val2 {
                    Value::Long(places)=>{
                        Value::Double((d.clone() * (10_f64.powi(places.clone() as i32))).round()/100.0)
                    },
                    _=>{unimplemented!()}
                }
            },
            _=>unimplemented!()
        }
    }
}
#[derive(Clone,PartialEq,Debug)]
pub enum Argument{
    Variable(Variable),
    Function(Function),
    Final(Value)
}
impl Fillable<Value> for Argument {
    fn fill(&self, runtime:&Environment) ->Value {
        match self {
            Argument::Function(fun)=>{
                fun.fill(runtime)
            },
            Argument::Variable(var)=>{
                runtime.channel.borrow_mut().read(var.clone())
            },
            Argument::Final(val)=>{
                val.clone()
            }
        }
    }
}
#[derive(Clone,PartialEq,Debug)]
pub struct Function{
    name:String,
    args:Vec<Argument>
}
pub trait Fillable<T> {
    fn fill(&self,runtime:&Environment)->T;
}
impl Fillable<Value> for Value {
    fn fill(&self, _runtime: &Environment) -> Value {
        self.clone()
    }
}
impl Fillable<Value> for Function {
    fn fill(&self, runtime:&Environment) ->Value {
        let mut args=Vec::new();
        for arg in &self.args{
            args.push(arg.fill(runtime))
        }
        get_function(self.name.clone()).eval(args)
    }
}
pub fn get_function(name:String)->Box<dyn Func>{
    match &name[..] {
        "concat" => Box::new(Concat),
        "mul"=>Box::new(Multiply),
        "add"=>Box::new(Add),
        "uuid"=>Box::new(UUID),
        "random"=>Box::new(Random),
        "round"=>Box::new(Round),
        _=>{unimplemented!()}
    }
}
