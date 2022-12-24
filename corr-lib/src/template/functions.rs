use crate::template::{Function, Expression, Fillable};
use crate::core::{runtime::Context, Value, Number};
use async_trait::async_trait;
use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use fake::faker::name::raw::*;
use fake::faker::company::raw::*;
use fake::faker::address::raw::*;
use fake::locales::*;
use base64::encode;
use fake::Fake;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use strfmt::{ Formatter, strfmt_map};
use std::collections::HashMap;
use std::env;
use anyhow::Result;
use captcha::Captcha;
use num_traits::ToPrimitive;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Concat;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Ceil;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Floor;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Array;
//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct UniqueRandomElements;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct CInt;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Round;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct LPad;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct RPad;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Mid;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Left;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Right;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Contains;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct CorrCaptcha;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct EnvVar;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct At;

#[async_trait]
impl Function for CorrCaptcha{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let chars = if let  Some(exp)= args.get(0){
            exp.evaluate(context).await?.parse::<u32>().unwrap_or(5)
        } else {
            5
        };
        let mut cp = Captcha::new();
        let mut cb = cp.add_chars(chars);
        let g = cb.text_area().clone();
        cb = cb.extract(g);
        let mut retval = HashMap::new();
        retval.insert("image".to_string(),Value::String(cb.as_base64().unwrap()));
        retval.insert("value".to_string(),Value::String(cb.chars_as_string()));
        Ok(Value::Map(retval))
    }
}
#[async_trait]
impl Function for At{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let val = if let  Some(exp)= args.get(0){
            exp.evaluate(context).await?
        } else {
            Value::Array(vec![])
        };
        let index = if let Some(exp)= args.get(1){
            exp.evaluate(context).await?.parse::<usize>().unwrap_or(0)
        } else {
            0 as usize
        };
        match val {
            Value::Array(res)=>{
                Ok(res.get(index).map(|val|val.clone()).unwrap_or(Value::Null))
            },
            Value::Buffer(res)=>{
                Ok(res.get(index).map(|val|Value::PositiveInteger(val.to_u128().unwrap_or(0))).unwrap_or(Value::Null))
            },
            _=>Ok(Value::Null)
        }

    }
}
#[async_trait]
impl Function for Array{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            if let Some(Some(length)) = arg.evaluate(context).await?.to_number().map(|num|num.as_usize()){
                let exp = if let Some(arg) = args.get(1){
                    arg.clone()
                } else {
                    Expression::Constant(Value::Null)
                };
                let mut vals=vec![];
                for _ in 0..length{
                    vals.push(exp.evaluate(context).await?);
                }
                let val = Value::Array(vals);
                Ok(val)
            } else {
                Ok(Value::Array(vec![]))
            }
        } else {
            Ok(Value::Array(vec![]))
        }
    }
}
#[async_trait]
impl Function for UniqueRandomElements{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(total_e) = args.get(0) {
            if let Some(array_e) = args.get(1) {
                let total = if let Some(num) = total_e.evaluate(context).await?.to_number().map(|num| num.as_usize()).flatten(){
                    num
                } else {
                    0
                };
                let array = if let Value::Array(array) = array_e.evaluate(context).await? {
                    array
                } else {
                    vec![]
                };
                let mut vals = vec![];
                let mut to_add;
                let mut rng = rand::thread_rng();
                for i in 0..total {
                    if i>=array.len(){
                        break
                    }
                    let index = rng.gen_range(0,array.len() -1 );
                    to_add =  array[index].clone();
                    while vals.contains(&to_add) {
                        let index = rng.gen_range(0,array.len() -1 );
                        to_add =  array[index].clone()
                    }
                    vals.push(to_add.clone());
                }
                Ok(Value::Array(vals))
            } else {
                Ok(Value::Array(vec![]))
            }
        } else {
            Ok(Value::Array(vec![]))
        }
    }
}
#[async_trait]
impl Function for Concat{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let mut buffer = "".to_string();
        for arg in args {
            buffer.push_str(arg.evaluate(context).await?.to_string().as_str());
        }
        Ok(Value::String(buffer))
    }
}
#[async_trait]
impl Function for LPad{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let base = args.get(0).unwrap_or(&Expression::Constant(Value::String("".to_string()))).evaluate(context).await?.to_string();
        let pad = args.get(1).unwrap_or(&Expression::Constant(Value::String("".to_string()))).evaluate(context).await?.to_string();
        let till:usize = args.get(2).unwrap_or(&Expression::Constant(Value::String(base.len().to_string()))).evaluate(context).await?.parse().unwrap_or(base.len());
        let mut new_str = base.clone();
        while new_str.len()<till {
            new_str = format!("{}{}",pad,new_str)
        }
        Ok(Value::String(new_str))
    }
}
#[async_trait]
impl Function for RPad{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let base = args.get(0).unwrap_or(&Expression::Constant(Value::String("".to_string()))).evaluate(context).await?.to_string();
        let pad = args.get(1).unwrap_or(&Expression::Constant(Value::String("".to_string()))).evaluate(context).await?.to_string();
        let till:usize = args.get(2).unwrap_or(&Expression::Constant(Value::String(base.len().to_string()))).evaluate(context).await?.parse().unwrap_or(base.len());
        let mut new_str = base.clone();
        while new_str.len()<till {
            new_str = format!("{}{}",new_str,pad)
        }
        Ok(Value::String(new_str))
    }
}
#[async_trait]
impl Function for Mid{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let base = args.get(0).unwrap_or(&Expression::Constant(Value::String("".to_string()))).evaluate(context).await?.to_string();
        let length:usize = args.get(2).unwrap_or(&Expression::Constant(Value::String("1".to_string()))).evaluate(context).await?.parse().unwrap_or(1);
        let start = args.get(1).unwrap_or(&Expression::Constant(Value::String("0".to_string()))).evaluate(context).await?.parse().unwrap_or(0);
        if start+length<base.len() {
            let sub_str = format!("{}",&base[start..start+length]);
            Ok(Value::String(sub_str))
        } else if start < base.len(){
            Ok(Value::String(format!("{}",base)))
        } else {
            Ok(Value::String(format!("")))
        }
    }
}
#[async_trait]
impl Function for Left{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let base = args.get(0).unwrap_or(&Expression::Constant(Value::String("".to_string()))).evaluate(context).await?.to_string();
        let length:usize = args.get(1).unwrap_or(&Expression::Constant(Value::String("1".to_string()))).evaluate(context).await?.parse().unwrap_or(1);
        if length<base.len() {
            let sub_str = format!("{}",&base[0..length]);
            Ok(Value::String(sub_str))
        } else {
            Ok(Value::String(format!("{}",base)))
        }
    }
}
#[async_trait]
impl Function for Right{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let base = args.get(0).unwrap_or(&Expression::Constant(Value::String("".to_string()))).evaluate(context).await?.to_string();
        let length:usize = args.get(1).unwrap_or(&Expression::Constant(Value::String("1".to_string()))).evaluate(context).await?.parse().unwrap_or(1);
        if length > base.len() {
            Ok(Value::String(format!("{}", base)))
        } else {
            let start = base.len() - length;
            let sub_str = format!("{}",&base[start..start+length]);
            Ok(Value::String(sub_str))
        }
    }
}
#[async_trait]
impl Function for Contains{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let tof = args.get(0).unwrap_or(&Expression::Constant(Value::String("".to_string()))).evaluate(context).await?.to_string();
        let mut i = 0;
        for arg in args {
            if i!=0{
                if !tof.contains(&arg.evaluate(context).await?.to_string()) {
                    return Ok(Value::Boolean(false))
                }
            }
            i = i+1
        }
        Ok(Value::Boolean(true))
    }
}

#[async_trait]
impl Function for EnvVar{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        Ok(env::var(args.get(0).unwrap().evaluate(context).await?.to_string()).map(|e|Value::String(e)).unwrap_or(Value::Null))
    }
}
//Add Function
#[derive(Debug,Clone,PartialEq)]
pub struct Add;
#[derive(Debug,Clone,PartialEq)]
pub struct Equal;

#[derive(Debug,Clone,PartialEq)]
pub struct LogicalAnd;

#[derive(Debug,Clone,PartialEq)]
pub struct LogicalOr;

#[derive(Debug,Clone,PartialEq)]
pub struct LogicalNot;

#[derive(Debug,Clone,PartialEq)]
pub struct GreaterThanEqual;
#[derive(Debug,Clone,PartialEq)]
pub struct GreaterThan;
#[derive(Debug,Clone,PartialEq)]
pub struct LessThanEqual;
#[derive(Debug,Clone,PartialEq)]
pub struct LessThan;


#[derive(Debug,Clone,PartialEq)]
pub struct NotEqual;

#[derive(Debug,Clone,PartialEq)]
pub struct Chunked;
// #[derive(Debug,Clone,PartialEq)]
// pub struct GreaterThan;
//
// #[derive(Debug,Clone,PartialEq)]
// pub struct GreaterThanEqual;
//
// #[derive(Debug,Clone,PartialEq)]
// pub struct LessThan;
//
// #[derive(Debug,Clone,PartialEq)]
// pub struct LessThanEqual;

//Random Element Function
#[derive(Debug,Clone,PartialEq)]
pub struct RandomElement;

//Random Element Function
#[derive(Debug,Clone,PartialEq)]
pub struct Random;

#[async_trait]
impl Function for Add{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let mut number= Number::PositiveInteger(0);
        for arg in args {
            if let Some(res)=arg.evaluate(context).await?.to_number(){
                number=number.add(res)
            }
        }
        Ok(number.to_value())
    }
}
#[async_trait]
impl Function for Equal{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let mut ret = true;
        let first = if let  Some(exp)= args.get(0){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        for arg in args {
            let res=arg.evaluate(context).await?;
            ret = ret && first.eq(&res);
        }
        Ok(Value::Boolean(ret))
    }
}

#[async_trait]
impl Function for LogicalAnd{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let mut ret = true;
        let mut next = Value::Boolean(true);
        for arg in args {
            let res=arg.evaluate(context).await?;
            ret = next.and(&res).to_bool();
            next  = Value::Boolean(ret)
        }
        Ok(Value::Boolean(ret))
    }
}
#[async_trait]
impl Function for LogicalOr{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let mut ret = true;
        let mut next = Value::Boolean(false);
        for arg in args {
            let res=arg.evaluate(context).await?;
            ret =  next.or(&res).to_bool();
            next  = Value::Boolean(ret)
        }
        Ok(Value::Boolean(ret))
    }
}
#[async_trait]
impl Function for LogicalNot{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let  Some(exp)= args.get(0){
            Ok(exp.evaluate(context).await?.not())
        } else {
            return Ok(Value::Boolean(false))
        }
    }
}
#[async_trait]
impl Function for GreaterThanEqual{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let first = if let  Some(exp)= args.get(0){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        let second = if let  Some(exp)= args.get(1){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        return Ok(first.ge(second));
    }
}
#[async_trait]
impl Function for LessThanEqual{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let first = if let  Some(exp)= args.get(0){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        let second = if let  Some(exp)= args.get(1){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        return Ok(first.le(second));
    }
}
#[async_trait]
impl Function for GreaterThan{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let first = if let  Some(exp)= args.get(0){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        let second = if let  Some(exp)= args.get(1){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        return Ok(first.gt(second));
    }
}
#[async_trait]
impl Function for LessThan{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let first = if let  Some(exp)= args.get(0){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        let second = if let  Some(exp)= args.get(1){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        return Ok(first.lt(second));
    }
}
#[async_trait]
impl Function for NotEqual{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let first = if let  Some(exp)= args.get(0){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        let second = if let  Some(exp)= args.get(1){
            exp.evaluate(context).await?
        } else {
            return Ok(Value::Boolean(true))
        };
        Ok(Value::Boolean(!first.eq(&second)))
    }
}

//Multiply Function
#[derive(Debug,Clone,PartialEq)]
pub struct Multiply;

//Multiply Function
#[derive(Debug,Clone,PartialEq)]
pub struct Formated;

//Multiply Function
#[derive(Debug,Clone,PartialEq)]
pub struct Mod;

//Uuid Function
#[derive(Debug,Clone,PartialEq)]
pub struct Uuid;

//Uuid Function
#[derive(Debug,Clone,PartialEq)]
pub struct TimeStamp;

//Get Current Date With Now and optional format
#[derive(Debug,Clone,PartialEq)]
pub struct Now;
#[async_trait]
impl Function for Chunked{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            let data = arg.evaluate(context).await?;
            match data {
                Value::Array(items)=>{
                    let default = Expression::Constant(Value::PositiveInteger(10));
                    if let Value::PositiveInteger(size) = args.get(1).unwrap_or(&default).evaluate(context).await?{
                        Ok(Value::Array(items.chunks(size.to_usize().unwrap_or(10)).map(|c| Value::Array(c.iter().map(|i| i.clone()).collect())).collect()))
                    } else {
                        Ok(Value::Array(items.chunks(10).map(|c| Value::Array(c.iter().map(|i| i.clone()).collect())).collect()))
                    }
                },
                Value::Buffer(items)=>{
                    let default = Expression::Constant(Value::PositiveInteger(10));
                    if let Value::PositiveInteger(size) = args.get(1).unwrap_or(&default).evaluate(context).await?{
                        Ok(Value::Array(items.chunks(size.to_usize().unwrap_or(10)).map(|c| Value::Buffer(c.iter().map(|i| i.clone()).collect())).collect()))
                    } else {
                        Ok(Value::Array(items.chunks(10).map(|c| Value::Buffer(c.iter().map(|i| i.clone()).collect())).collect()))
                    }
                },
                _=>Ok(Value::Array(vec![arg.evaluate(context).await?]))
            }
        } else {
            Ok(Value::Null)
        }
    }
}
#[async_trait]
impl Function for Multiply{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let mut number= Number::PositiveInteger(1);
        for arg in args {
            if let Some(res)=arg.evaluate(context).await?.to_number(){
                number=number.multiply(res)
            }
        }
        Ok(number.to_value())
    }
}
#[async_trait]
impl Function for Mod{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await?.to_number(){
                if let Some(arg) = args.get(1){
                    if let Some(second) = arg.evaluate(context).await?.to_number(){
                        Ok(first.remainder(second).to_value())
                    } else {
                        Ok(first.to_value())
                    }
                } else {
                    Ok(first.to_value())
                }
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }
    }
}

#[async_trait]
impl Function for Ceil{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await?.to_number(){
                Ok(first.ceil().to_value())
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }
    }
}
#[async_trait]
impl Function for CInt{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await?.to_number(){
                Ok(first.cint().to_value())
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }
    }
}
#[async_trait]
impl Function for Floor{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await?.to_number(){
                Ok(first.floor().to_value())
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }
    }
}

#[async_trait]
impl Function for Round{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await?.to_number(){
                Ok(first.round().to_value())
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }
    }
}

#[async_trait]
impl Function for Uuid{
    async fn evaluate(&self, _args: Vec<Expression>, _context: &Context) -> Result<Value> {
        let val = uuid::Uuid::new_v4();
        Ok(Value::String(val.to_string()))
    }
}
#[async_trait]
impl Function for TimeStamp{
    async fn evaluate(&self, _args: Vec<Expression>, _context: &Context) -> Result<Value> {
        let val = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        Ok(Value::PositiveInteger(val))
    }
}

#[async_trait]
impl Function for Now{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let  value = if args.len() == 1 {
            let format = args.get(0).unwrap().evaluate(context).await?.to_string();
            chrono::Utc::now().format(format.as_str()).to_string()
        } else {
            chrono::Utc::now().to_rfc3339().to_string()
        };
        Ok(Value::String(value))
    }
}


//Subtarct Function
#[derive(Debug,Clone,PartialEq)]
pub struct Subtract;

//Fake Function
#[derive(Debug,Clone,PartialEq)]
pub struct FakeValue;

#[derive(Debug,Clone,PartialEq)]
pub struct Increment;

#[derive(Debug,Clone,PartialEq)]
pub struct Decrement;

#[derive(Debug,Clone,PartialEq)]
pub struct Length;

#[derive(Debug,Clone,PartialEq)]
pub struct IndexOf;

#[async_trait]
impl Function for Length{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let val = args.get(0).unwrap().fill(context).await?;
        match val {
            Value::String(str)=>{
                Ok(Value::PositiveInteger(str.len() as u128))
            },
            Value::Array(arr)=>{
                Ok(Value::PositiveInteger(arr.len() as u128))
            },
            _=>{
                Ok(Value::PositiveInteger(1 as u128))
            }
        }
    }
}
#[async_trait]
impl Function for IndexOf{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let val:Value = args.get(0).unwrap().fill(context).await?;
        let of_value:Value = args.get(1).unwrap().fill(context).await?;
        match val {
            Value::String(str)=>{
                let index =  str.find(&of_value.to_string());
                if let Some(ind)= index {
                    Ok(Value::Integer(ind as i128))
                } else {
                    Ok(Value::Integer(-1))
                }
            },
            Value::Array(arr)=>{
                let index = arr.iter().position(|el| el.eq(&of_value));
                if let Some(ind)= index {
                    Ok(Value::Integer(ind as i128))
                } else {
                    Ok(Value::Integer(-1))
                }
            },
            _=>{
                Ok(Value::Integer(if val.eq(&of_value){0} else {-1}))
            }
        }
    }
}
#[async_trait]
impl Function for FakeValue{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Value::String(arg) = args.get(0).unwrap().fill(context).await? {
            Ok(get_fake(arg))
        } else {
            Ok(Value::Null)
        }

    }
}

#[async_trait]
impl Function for Increment{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await?.to_number(){
                Ok(first.add(Number::PositiveInteger(1)).to_value())
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }

    }
}

#[async_trait]
impl Function for Decrement{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await?.to_number(){
                Ok(first.subtract(Number::PositiveInteger(1)).to_value())
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }

    }
}
#[async_trait]
impl Function for Formated{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            let first = arg.evaluate(context).await?.to_string();
            let mut vars=HashMap::new();
            let mut index = 0;
            for arg in args{
                if index !=0 {
                    let num = arg.evaluate(context).await?.to_number().unwrap();
                    match num{
                        Number::Double(d)=>{
                            vars.insert(format!("{0}",index-1),d);
                        },
                        _=>{}
                    }
                }

                index = index + 1;

            }

            let f = |mut fmt: Formatter| {
                // print!("{0}",fmt.key);
                fmt.f64(*vars.get(fmt.key).unwrap())
            };

            let fstr = strfmt_map(first.as_str(),&f).unwrap();
            Ok(Value::String(fstr))
                // if let Some(arg) = args.get(1){
                //     if let Some(second) = arg.evaluate(context).await.to_number(){
                //         first.subtract(second).to_value()
                //     } else {
                //         first.to_value()
                //     }
                // } else {
                //     first.to_value()
                // }
        } else {
            Ok(Value::Null)
        }
    }
}
fn get_fake(fake_type:String)->Value{
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
        "StateAbbr"=>Value::String(StateAbbr(EN).fake()),
        "ZipCode"=>Value::String(format!("{:05}",ZipCode(EN).fake::<String>().trim())),
        _=>Value::Null
    }
}

#[async_trait]
impl Function for Subtract{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await?.to_number(){
                if let Some(arg) = args.get(1){
                    if let Some(second) = arg.evaluate(context).await?.to_number(){
                        Ok(first.subtract(second).to_value())
                    } else {
                        Ok(first.to_value())
                    }
                } else {
                    Ok(first.to_value())
                }
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }
    }
}
#[async_trait]
impl Function for Random{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {

        if let Some(val1) = args.get(0){
            if let Some(val2) = args.get(1){

                let value1:Value = val1.fill(context).await?;
                let value2:Value = val2.fill(context).await?;
                let mut rng = rand::thread_rng();
                if let Value::PositiveInteger(uv1) = value1{
                    if let Value::PositiveInteger(uv2)=value2{

                        Ok(Value::PositiveInteger(rng.gen_range(uv1,uv2)))
                    } else {
                        Ok(Value::Null)
                    }
                } else if let Value::Integer(uv1) = value1{
                    if let Value::Integer(uv2)=value2{

                        Ok(Value::Integer(rng.gen_range(uv1,uv2)))
                    } else {
                        Ok(Value::Null)
                    }
                } else if let Value::Double(uv1) = value1{
                    if let Value::Double(uv2)=value2{
                        Ok(Value::Double(rng.gen_range(uv1,uv2)))
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    Ok(Value::Null)
                }
            } else {
                Ok(Value::Null)
            }

        }else {
            Ok(Value::Null)
        }
    }
}
#[async_trait]
impl Function for RandomElement{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0) {
            let value:Value = arg.fill(context).await?;
            if let Value::Array(val)=value{
                if val.len()==0{
                    Ok(Value::Null)
                } else if val.len() == 1 {
                    Ok(val.get(0).unwrap().clone())
                } else {
                    let mut rng = rand::thread_rng();
                    let index = rng.gen_range(0,val.len() -1 );
                    if let Some(ret_val) = val.get(index){
                        Ok(ret_val.clone())
                    } else {
                        Ok(Value::Null)
                    }
                }

            } else {
                Ok(value.clone())
            }
        } else {
            Ok(Value::Null)
        }
    }
}
//Divide Function
#[derive(Debug,Clone,PartialEq)]
pub struct Divide;


#[async_trait]
impl Function for Divide{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await?.to_number(){
                if let Some(arg) = args.get(1){
                    if let Some(second) = arg.evaluate(context).await?.to_number(){
                        Ok(first.divide(second).to_value())
                    } else {
                        Ok(first.to_value())
                    }
                } else {
                    Ok(first.to_value())
                }
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }
    }
}

//Divide Function
#[derive(Debug,Clone,PartialEq)]
pub struct Encode;

#[async_trait]
impl Function for Encode{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        if let Some(arg) = args.get(0){
            let filled:Value = arg.fill(context).await?;
            Ok(Value::String(encode(filled.to_string())))
        } else {
            Ok(Value::Null)
        }
    }
}

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct FromJson;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct ReadWavSamples;

#[derive(Debug,Clone,PartialEq)]
pub struct ReadFileBinary;

#[async_trait]
impl Function for FromJson{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let path:String = args.get(0).unwrap().fill(context).await?;
        if let Ok(file) = File::open(path){
            let reader = BufReader::new(file);
            let file_contents= serde_json::from_reader(reader);
            // Read the JSON contents of the file as an instance of `User`.
            if let Ok(value) = file_contents{
                Ok(Value::from_json_value(value))
            } else {
                Ok(Value::Null)
            }

        } else {
            Ok(Value::Null)
        }
    }
}
#[async_trait]
impl Function for ReadFileBinary {
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let path:String = args.get(0).unwrap().fill(context).await?;
        let res = tokio::fs::read(path.clone()).await;
        match res {
            Ok(data)=>{
                Ok(Value::Buffer(data))
            },
            Err(e)=>{
                eprintln!("Error {:?} while opening file {}",e,path);
                context.exit(-1).await;
                Ok(Value::Null)
            }
        }
    }
}
#[async_trait]
impl Function for ReadWavSamples{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Result<Value> {
        let path:String = args.get(0).unwrap().fill(context).await?;
        let res = hound::WavReader::open(path.clone());
        match res {
            Ok(mut reader)=>{
                let d={
                    let data:Vec<u8> = reader.samples().map(|s: hound::Result<i16>| {s.unwrap().to_le_bytes()}).flat_map(|b| b).collect();
                    Ok(Value::Buffer(data))
                };
                d
            },
            Err(e)=>{
                eprintln!("Error {:?} while opening file {}",e,path);
                context.exit(-1).await;
                Ok(Value::Null)
            }
        }

    }
}
pub fn functions()->Vec<(&'static str,Arc<dyn Function>)>{
    return vec![
        ("timestamp",Arc::new(TimeStamp{})),
        ("now",Arc::new(Now{})),
        ("uuid",Arc::new(Uuid{})),
        ("add",Arc::new(Add{})),
        ("captcha",Arc::new(CorrCaptcha{})),
        ("mod",Arc::new(Mod{})),
        ("ceil",Arc::new(Ceil{})),
        ("floor",Arc::new(Floor{})),
        ("len",Arc::new(Length{})),
        ("indexOf",Arc::new(IndexOf{})),
        ("round",Arc::new(Round{})),
        ("cint",Arc::new(CInt{})),
        ("sub",Arc::new(Subtract{})),
        ("mul",Arc::new(Multiply{})),
        ("div",Arc::new(Divide{})),
        ("concat",Arc::new(Concat{})),
        ("lpad",Arc::new(LPad{})),
        ("at",Arc::new(At{})),
        ("rpad",Arc::new(RPad{})),
        ("mid",Arc::new(Mid{})),
        ("left",Arc::new(Left{})),
        ("right",Arc::new(Right{})),
        ("from_json",Arc::new(FromJson{})),
        ("read_wav",Arc::new(ReadWavSamples{})),
        ("read_binary",Arc::new(ReadFileBinary{})),
        ("chunked",Arc::new(Chunked{})),
        ("fake",Arc::new(FakeValue{})),
        ("encode",Arc::new(Encode{})),
        ("random",Arc::new(Random{})),
        ("array",Arc::new(Array{})),
        ("unique_random_elements",Arc::new(UniqueRandomElements{})),
        ("contains",Arc::new(Contains{})),
        ("random_element",Arc::new(RandomElement{})),
        ("env",Arc::new(EnvVar{}))
    ]
}
pub fn function_names()->Vec<&'static str>{
    let mut names =vec![];
    for (name,_) in functions() {
        names.push(name);
    }
    names
}
pub fn get_function(name:&str)->Arc<dyn Function>{
    for (reserved_name,value) in functions() {
        if reserved_name.eq(name){
            return value;
        }
    }
    return Arc::new(Concat{})
}
#[cfg(test)]

mod tests{
    use crate::core::{DataType, Value};
    use crate::core::proto::{Input, ContinueInput, Output, TellMeOutput};
    use std::sync::{Arc, Mutex};
    use crate::template::functions::*;
    use crate::core::runtime::Context;
    use crate::template::{Expression, Function};

    #[tokio::test]
    async fn should_find_index_of_array(){
        let a=IndexOf{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![Expression::Constant(Value::Array(vec![
            Value::PositiveInteger(2),
            Value::String("0".to_string()),
            Value::String("3".to_string())
        ])),Expression::Constant(Value::String("0".to_string()))],&context).await.unwrap();
        assert_eq!(result,Value::Integer(1));
    }
    #[tokio::test]
    async fn should_give_negative_when_cannot_find_index_of_array(){
        let a=IndexOf{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![Expression::Constant(Value::Array(vec![
            Value::PositiveInteger(2),
            Value::String("0".to_string()),
            Value::String("3".to_string())
        ])),Expression::Constant(Value::String("5".to_string()))],&context).await.unwrap();
        assert_eq!(result,Value::Integer(-1));
    }
    #[tokio::test]
    async fn should_find_index_of_string(){
        let a=IndexOf{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![ Expression::Constant(Value::String("Atmaram".to_string())),Expression::Constant(Value::String("ma".to_string()))],&context).await.unwrap();
        assert_eq!(result,Value::Integer(2));
    }
    #[tokio::test]
    async fn should_give_negative_when_cannot_find_index_of_string(){
        let a=IndexOf{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![ Expression::Constant(Value::String("Atmaram".to_string())),Expression::Constant(Value::String("Z".to_string()))],&context).await.unwrap();
        assert_eq!(result,Value::Integer(-1));
    }
    #[tokio::test]
    async fn should_find_length_of_array(){
        let a=Length{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![Expression::Constant(Value::Array(vec![
            Value::PositiveInteger(2),
            Value::String("0".to_string()),
            Value::String("3".to_string())
        ]))],&context).await.unwrap();
        assert_eq!(result,Value::PositiveInteger(3));
    }
    #[tokio::test]
    async fn should_find_length_of_string(){
        let a=Length{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![ Expression::Constant(Value::String("Atmaram".to_string()))],&context).await.unwrap();
        assert_eq!(result,Value::PositiveInteger(7));
    }
    #[tokio::test]
    async fn should_lpad(){
        let a=LPad{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::PositiveInteger(2)),
            Expression::Constant(Value::String("0".to_string())),
            Expression::Constant(Value::String("3".to_string()))
        ],&context).await.unwrap();
        assert_eq!(result,Value::String(format!("002")));
    }
    #[tokio::test]
    async fn should_generate_array(){
        let a=Array{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::PositiveInteger(2)),
            Expression::Constant(Value::Array(vec![Value::String("3".to_string())])),
        ],&context).await.unwrap();
        if let Value::Array(vals)=result{
            assert_eq!(vals.len(),2);
        } else {
            panic!("Not even array")
        }

    }
    #[tokio::test]
    async fn should_generate_array_of_unique_random_elements(){
        let a=UniqueRandomElements{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::PositiveInteger(2)),
            Expression::Constant(Value::Array(vec![
                Value::String("1".to_string()),
                Value::String("2".to_string()),
                Value::String("3".to_string()),
                Value::String("4".to_string()),
                Value::String("5".to_string()),
            ])),
        ],&context).await.unwrap();
        if let Value::Array(vals)=result{
            assert_eq!(vals.len(),2);
            println!("{:?}",vals)
        } else {
            panic!("Not even array")
        }

    }
    #[tokio::test]
    async fn should_or(){
        let a=LogicalOr{};
        let tests = vec![
            (true,true,true),
            (true,false,true),
            (false,true,true),
            (false,false,false),
        ];
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        for (first,second,op) in tests {
            let result=a.evaluate(vec![
                Expression::Constant(Value::Boolean(first)),
                Expression::Constant(Value::Boolean(second)),
            ],&context).await.unwrap();
            assert_eq!(result,Value::Boolean(op));
        }

    }
    #[tokio::test]
    async fn should_and(){
        let a=LogicalAnd{};
        let tests = vec![
            (true,true,true),
            (true,false,false),
            (false,true,false),
            (false,false,false),
        ];
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        for (first,second,op) in tests {
            let result=a.evaluate(vec![
                Expression::Constant(Value::Boolean(first)),
                Expression::Constant(Value::Boolean(second)),
            ],&context).await.unwrap();
            assert_eq!(result,Value::Boolean(op));
        }

    }
    #[tokio::test]
    async fn should_not(){
        let a=LogicalNot{};
        let tests = vec![
            (true,false),
            (false,true),
        ];
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        for (first,op) in tests {
            let result=a.evaluate(vec![
                Expression::Constant(Value::Boolean(first)),
            ],&context).await.unwrap();
            assert_eq!(result,Value::Boolean(op));
        }

    }
    #[tokio::test]
    async fn should_round_above(){
        let a=Round{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::Double(15.5)),
        ],&context).await.unwrap();
        assert_eq!(result,Value::Double(16.0));
    }
    #[tokio::test]
    async fn should_round_below(){
        let a=Round{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::Double(15.4)),
        ],&context).await.unwrap();
        assert_eq!(result,Value::Double(15.0));
    }

    #[tokio::test]
    async fn should_ceil(){
        let a=Ceil{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::Double(15.4)),
        ],&context).await.unwrap();
        assert_eq!(result,Value::Double(16.0));
    }

    #[tokio::test]
    async fn should_cint(){
        let a=CInt{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::Double(15.4)),
        ],&context).await.unwrap();
        assert_eq!(result,Value::Integer(15));
    }

    #[tokio::test]
    async fn should_floor(){
        let a=Floor{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::Double(15.4)),
        ],&context).await.unwrap();
        assert_eq!(result,Value::Double(15.0));
    }
    #[tokio::test]
    async fn should_rpad(){
        let a=RPad{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::PositiveInteger(2)),
            Expression::Constant(Value::String("0".to_string())),
            Expression::Constant(Value::String("3".to_string()))
        ],&context).await.unwrap();
        assert_eq!(result,Value::String(format!("200")));
    }

    #[tokio::test]
    async fn should_mid(){
        let a=Mid{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::String("0123456789".to_string())),
            Expression::Constant(Value::String("1".to_string())),
            Expression::Constant(Value::String("3".to_string()))
        ],&context).await.unwrap();
        assert_eq!(result,Value::String(format!("123")));
    }
    #[tokio::test]
    async fn should_left(){
        let a=Left{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::String("0123456789".to_string())),
            Expression::Constant(Value::String("4".to_string())),
        ],&context).await.unwrap();
        assert_eq!(result,Value::String(format!("0123")));
    }

    #[tokio::test]
    async fn should_right(){
        let a=Right{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::String("0123456789".to_string())),
            Expression::Constant(Value::String("4".to_string())),
        ],&context).await.unwrap();
        assert_eq!(result,Value::String(format!("6789")));
    }

    #[tokio::test]
    async fn should_concat(){
        let a=Concat{};
        let input=vec![Input::Continue(ContinueInput{name:"one".to_string(),value:"123".to_string(),data_type:DataType::PositiveInteger})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![Expression::Variable("one".to_string(),Option::Some(DataType::PositiveInteger)),Expression::Constant(Value::String("hello".to_string()))],&context).await.unwrap();
        assert_eq!(result,Value::String("123hello".to_string()));
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"one".to_string(),data_type:DataType::PositiveInteger}));
    }



    #[tokio::test]
    async fn should_add(){
        let a=Add{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::PositiveInteger(2)),
            Expression::Constant(Value::String("3".to_string()))
        ],&context).await.unwrap();
        assert_eq!(result,Value::PositiveInteger(5));
    }
    #[tokio::test]
    async fn should_ge(){
        let a=GreaterThanEqual{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::PositiveInteger(2)),
            Expression::Constant(Value::String("3".to_string()))
        ],&context).await.unwrap();
        assert_eq!(result,Value::Boolean(false));
    }
    #[tokio::test]
    async fn should_get_zipcode(){
        if let Value::String(str)=get_fake("Zipcode".to_string()){
            assert_eq!(str.len(),5);
        }

    }
    #[tokio::test]
    async fn should_subtract(){
        let a=Subtract{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::PositiveInteger(2)),
            Expression::Constant(Value::String("3".to_string()))
        ],&context).await.unwrap();
        assert_eq!(result,Value::Integer(-1));
    }
    #[tokio::test]
    async fn should_multiply(){
        let a=Multiply{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::PositiveInteger(2)),
            Expression::Constant(Value::String("3".to_string()))
        ],&context).await.unwrap();
        assert_eq!(result,Value::PositiveInteger(6));
    }
    #[tokio::test]
    async fn should_format_without_any_args(){
        let a=Formated{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::String("Hello".to_string()))
        ],&context).await.unwrap();
        assert_eq!(result,Value::String("Hello".to_string()));
    }
    #[tokio::test]
    async fn should_format_with_double_args(){
        let a=Formated{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::String("Hello {0}".to_string())),
            Expression::Constant(Value::Double(100.0)),
        ],&context).await.unwrap();
        assert_eq!(result,Value::String("Hello 100".to_string()));
    }
    #[tokio::test]
    async fn should_divide(){
        let a=Divide{};
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![
            Expression::Constant(Value::PositiveInteger(4)),
            Expression::Constant(Value::String("2".to_string()))
        ],&context).await.unwrap();
        assert_eq!(result,Value::PositiveInteger(2));
    }
}
