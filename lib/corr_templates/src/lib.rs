pub mod json;
pub mod text;
extern crate rand;
#[macro_use]
extern crate nom;
use corr_core::runtime::Value;
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
        Value::String(Uuid::new_v4().to_string())
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
