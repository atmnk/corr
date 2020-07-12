pub mod json;
pub mod text;
pub mod parser;
extern crate rand;
#[macro_use]
extern crate nom;
use corr_core::runtime::{Value, Environment, ValueProvider, Variable};
use uuid::Uuid;
use rand::Rng;
use rand::seq::SliceRandom;

pub trait Func{
    fn eval(&self,args:Vec<Value>)->Value;
}
pub trait Extractable{
    fn extract(&self,val:Value,runtime:&Environment);
}
impl Extractable for Variable{
    fn extract(&self, val: Value, runtime: &Environment) {
        runtime.save(self.clone(),val)
    }
}
fn luhn_from(num:i64,bias:i64)->String{
    let mut pos_num = num;
    let mut take = pos_num % 10;
    let mut odd=true;
    let mut sum = 0;
    while(pos_num!=0 || take!=0){

        let mut add=if(odd){
            (((take*2)/10) + ((take*2)%10))
        } else {
            take
        };
        sum=sum+add;
        odd = !odd;

        pos_num = pos_num / 10;
        take = pos_num % 10;
    }
    sum = 10-((sum+bias) % 10);
    if(sum==10){
        sum = 0
    }
    sum= sum + (num*10);
    format!("{:010}", sum)
}

pub struct Round;
pub struct Random;
pub struct UUID;
pub struct Concat;
pub struct Multiply;
pub struct Add;
pub struct Either;
pub struct LPad;
pub struct Env;
pub struct Luhn;
pub struct FakeValues;
use fake::faker::name::raw::*;
use fake::faker::lorem::raw::*;
use fake::faker::company::raw::*;
use fake::faker::address::raw::*;
use fake::locales::*;
use fake::Fake;

fn fake(fake_type:String)->Value{
    match fake_type.as_str() {
        "Name"=> Value::String(Name(EN).fake()),
        "FirstName"=>Value::String(FirstName(EN).fake()),
        "LastName"=>Value::String(LastName(EN).fake()),
        "Title"=>Value::String(Title(EN).fake()),
        "Suffix"=>Value::String(Suffix(EN).fake()),
        "NameWithTitle"=>Value::String(NameWithTitle(EN).fake()),
        "CompanySuffix"=>Value::String(CompanySuffix(EN).fake()),
        "CompanyName"=>Value::String(CompanyName(EN).fake()),
        "Profession"=>Value::String(Profession(EN).fake()),
        "CityName"=>Value::String(CityName(EN).fake()),
        "StreetName"=>Value::String(StreetName(EN).fake()),
        "StateName"=>Value::String(StateName(EN).fake()),
        "ZipCode"=>Value::String(ZipCode(EN).fake()),
        _=>Value::Null
    }
}
impl  Func for FakeValues {
    fn eval(&self, args: Vec<Value>) -> Value {
        let fake_type=if let Some(Value::String(value))=args.get(0){
            value.clone()
        } else {
            "Name".to_string()
        };
        fake(fake_type)
    }
}
impl Func for Luhn{
    fn eval(&self, args: Vec<Value>) -> Value {
        let from=if let Some(Value::Long(value))=args.get(0){
            value.clone()
        } else {
            0
        };
        let bias=if let Some(Value::Long(value))=args.get(1){
            value.clone()
        } else {
            0
        };
        Value::String(luhn_from(from,bias))

    }
}
impl Func for Env{
    fn eval(&self, args: Vec<Value>) -> Value {
        let alt_value=if let Some(Value::String(value))=args.get(1){
            Value::String(value.clone())
        } else {
            Value::Null
        };
        if let Some(Value::String(var_name))=args.get(0){
            if let Ok(var_value)=std::env::var(var_name.as_str()) {
                Value::String(var_value)
            } else {
                alt_value
            }
        } else {
            Value::Null
        }

    }
}
impl Func for Either{
    fn eval(&self,args: Vec<Value>)->Value {
        args.choose(&mut rand::thread_rng()).unwrap().clone()
    }
}

impl Func for LPad{
    fn eval(&self,args: Vec<Value>)->Value {
        let val1=args.get(0).unwrap();
        let val2=args.get(1).unwrap();
        let val3=if let Some(val3)=args.get(2){
            val3.clone()
        } else {
            Value::String("0".to_string())
        };
        let padd_this=match val3 {
            Value::String(str_val)=>str_val,
            _=>"0".to_string()
        };
        let mut buffer="".to_string();
        match val2 {
            Value::Long(val)=>{
                let initial= val1.to_string();
                let size = (val - initial.len() as i64) as usize;
                for i in 0..size {
                    buffer.push_str(padd_this.as_str())
                }
                buffer.push_str(initial.as_str());
                Value::String(buffer)
            },
            _=> val1.clone()
        }
    }
}
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
        "either"=>Box::new(Either),
        "lpad"=>Box::new(LPad),
        "env"=>Box::new(Env),
        "luhn"=>Box::new(Luhn),
        "fake"=>Box::new(FakeValues),
        _=>{unimplemented!()}
    }
}
#[cfg(test)]
mod tests{
    use crate::{luhn_from};
    #[test]
    fn check_luhn(){
        assert_eq!(luhn_from(113429602,24),"1134296023");
    }

}
