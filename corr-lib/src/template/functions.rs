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
use captcha::Captcha;
use captcha::filters::{Noise, Wave, Dots};
use crate::journey::step::rest::CorrRequest;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Concat;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Contains;

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct CorrCaptcha;

#[async_trait]
impl Function for CorrCaptcha{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        let chars = if let  Some(exp)= args.get(0){
            exp.evaluate(context).await.parse::<u32>().unwrap_or(5)
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
        Value::Map(retval)
    }
}

#[async_trait]
impl Function for Concat{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        let mut buffer = "".to_string();
        for arg in args {
            buffer.push_str(arg.evaluate(context).await.to_string().as_str());
        }
        Value::String(buffer)
    }
}
#[async_trait]
impl Function for Contains{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        let mut tof = args.get(0).unwrap_or(&Expression::Constant(Value::String("".to_string()))).evaluate(context).await.to_string();
        let mut i = 0;
        for arg in args {
            if i!=0{
                if !tof.contains(&arg.evaluate(context).await.to_string()) {
                    return Value::Boolean(false)
                }
            }
            i = i+1
        }
        Value::Boolean(true)
    }
}


//Add Function
#[derive(Debug,Clone,PartialEq)]
pub struct Add;
#[derive(Debug,Clone,PartialEq)]
pub struct Equal;

//Random Element Function
#[derive(Debug,Clone,PartialEq)]
pub struct RandomElement;

//Random Element Function
#[derive(Debug,Clone,PartialEq)]
pub struct Random;

#[async_trait]
impl Function for Add{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        let mut number= Number::PositiveInteger(0);
        for arg in args {
            if let Some(res)=arg.evaluate(context).await.to_number(){
                number=number.add(res)
            }
        }
        number.to_value()
    }
}
#[async_trait]
impl Function for Equal{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        let mut ret = true;
        let first = if let  Some(exp)= args.get(0){
            exp.evaluate(context).await
        } else {
            return Value::Boolean(true)
        };
        for arg in args {
            let res=arg.evaluate(context).await;
            ret = ret && first.eq(&res);
        }
        Value::Boolean(ret)
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
impl Function for Multiply{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        let mut number= Number::PositiveInteger(1);
        for arg in args {
            if let Some(res)=arg.evaluate(context).await.to_number(){
                number=number.multiply(res)
            }
        }
        number.to_value()
    }
}
#[async_trait]
impl Function for Mod{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await.to_number(){
                if let Some(arg) = args.get(1){
                    if let Some(second) = arg.evaluate(context).await.to_number(){
                        first.remainder(second).to_value()
                    } else {
                        first.to_value()
                    }
                } else {
                    first.to_value()
                }
            } else {
                Value::Null
            }
        } else {
            Value::Null
        }
    }
}

#[async_trait]
impl Function for Uuid{
    async fn evaluate(&self, _args: Vec<Expression>, _context: &Context) -> Value {
        let val = uuid::Uuid::new_v4();
        Value::String(val.to_string())
    }
}
#[async_trait]
impl Function for TimeStamp{
    async fn evaluate(&self, _args: Vec<Expression>, _context: &Context) -> Value {
        let val = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        Value::PositiveInteger(val)
    }
}

#[async_trait]
impl Function for Now{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        let  value = if args.len() == 1 {
            let format = args.get(0).unwrap().evaluate(context).await.to_string();
            chrono::Utc::now().format(format.as_str()).to_string()
        } else {
            chrono::Utc::now().to_rfc3339().to_string()
        };
        Value::String(value)
    }
}

// #[async_trait]
// impl Function for Now{
//     async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
//         let  value = if args.len() == 1 {
//             let format = args.get(0).unwrap().evaluate(context).await.to_string();
//             chrono::Utc::now().format(format.as_str()).to_string()
//         } else {
//             chrono::Utc::now().to_rfc3339().to_string()
//         };
//         Value::String(value)
//     }
// }

//Subtarct Function
#[derive(Debug,Clone,PartialEq)]
pub struct Subtract;

//Fake Function
#[derive(Debug,Clone,PartialEq)]
pub struct FakeValue;

//Fake Function
#[derive(Debug,Clone,PartialEq)]
pub struct Increment;

//Fake Function
#[derive(Debug,Clone,PartialEq)]
pub struct Decrement;

#[async_trait]
impl Function for FakeValue{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        if let Value::String(arg) = args.get(0).unwrap().fill(context).await {
            get_fake(arg)
        } else {
            Value::Null
        }

    }
}

#[async_trait]
impl Function for Increment{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await.to_number(){
                first.add(Number::PositiveInteger(1)).to_value()
            } else {
                Value::Null
            }
        } else {
            Value::Null
        }

    }
}

#[async_trait]
impl Function for Decrement{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await.to_number(){
                first.subtract(Number::PositiveInteger(1)).to_value()
            } else {
                Value::Null
            }
        } else {
            Value::Null
        }

    }
}
#[async_trait]
impl Function for Formated{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        if let Some(arg) = args.get(0){
            let first = arg.evaluate(context).await.to_string();
            let mut vars=HashMap::new();
            let mut index = 0;
            for arg in args{
                if index !=0 {
                    let num = arg.evaluate(context).await.to_number().unwrap();
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
            Value::String(fstr)
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
            Value::Null
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
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await.to_number(){
                if let Some(arg) = args.get(1){
                    if let Some(second) = arg.evaluate(context).await.to_number(){
                        first.subtract(second).to_value()
                    } else {
                        first.to_value()
                    }
                } else {
                    first.to_value()
                }
            } else {
                Value::Null
            }
        } else {
            Value::Null
        }
    }
}
#[async_trait]
impl Function for Random{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {

        if let Some(val1) = args.get(0){
            if let Some(val2) = args.get(1){

                let value1:Value = val1.fill(context).await;
                let value2:Value = val2.fill(context).await;
                let mut rng = rand::thread_rng();
                if let Value::PositiveInteger(uv1) = value1{
                    if let Value::PositiveInteger(uv2)=value2{

                        Value::PositiveInteger(rng.gen_range(uv1,uv2))
                    } else {
                        Value::Null
                    }
                } else if let Value::Integer(uv1) = value1{
                    if let Value::Integer(uv2)=value2{

                        Value::Integer(rng.gen_range(uv1,uv2))
                    } else {
                        Value::Null
                    }
                } else if let Value::Double(uv1) = value1{
                    if let Value::Double(uv2)=value2{
                        Value::Double(rng.gen_range(uv1,uv2))
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Null
                }
            } else {
                Value::Null
            }

        }else {
            Value::Null
        }
    }
}
#[async_trait]
impl Function for RandomElement{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        if let Some(arg) = args.get(0) {
            let value:Value = arg.fill(context).await;
            if let Value::Array(val)=value{
                if val.len()==0{
                    Value::Null
                } else if val.len() == 1 {
                    val.get(0).unwrap().clone()
                } else {
                    let mut rng = rand::thread_rng();
                    let index = rng.gen_range(0,val.len() -1 );
                    if let Some(ret_val) = val.get(index){
                        ret_val.clone()
                    } else {
                        Value::Null
                    }
                }

            } else {
                value.clone()
            }
        } else {
            Value::Null
        }
    }
}
//Divide Function
#[derive(Debug,Clone,PartialEq)]
pub struct Divide;


#[async_trait]
impl Function for Divide{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        if let Some(arg) = args.get(0){
            if let Some(first) = arg.evaluate(context).await.to_number(){
                if let Some(arg) = args.get(1){
                    if let Some(second) = arg.evaluate(context).await.to_number(){
                        first.divide(second).to_value()
                    } else {
                        first.to_value()
                    }
                } else {
                    first.to_value()
                }
            } else {
                Value::Null
            }
        } else {
            Value::Null
        }
    }
}

//Divide Function
#[derive(Debug,Clone,PartialEq)]
pub struct Encode;

#[async_trait]
impl Function for Encode{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        if let Some(arg) = args.get(0){
            let filled:Value = arg.fill(context).await;
            Value::String(encode(filled.to_string()))
        } else {
            Value::Null
        }
    }
}

//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct FromJson;

#[async_trait]
impl Function for FromJson{
    async fn evaluate(&self, args: Vec<Expression>, context: &Context) -> Value {
        let path:String = args.get(0).unwrap().fill(context).await;
        if let Ok(file) = File::open(path){
            let reader = BufReader::new(file);
            let file_contents= serde_json::from_reader(reader);
            // Read the JSON contents of the file as an instance of `User`.
            if let Ok(value) = file_contents{
                Value::from_json_value(value)
            } else {
                Value::Null
            }

        } else {
            Value::Null
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
        ("sub",Arc::new(Subtract{})),
        ("mul",Arc::new(Multiply{})),
        ("div",Arc::new(Divide{})),
        ("concat",Arc::new(Concat{})),
        ("from_json",Arc::new(FromJson{})),
        ("fake",Arc::new(FakeValue{})),
        ("encode",Arc::new(Encode{})),
        ("random",Arc::new(Random{})),
        ("contains",Arc::new(Contains{})),
        ("random_element",Arc::new(RandomElement{}))
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
    use crate::template::functions::{Concat, Add, Subtract, Multiply, Divide, Formated, get_fake};
    use crate::core::runtime::Context;
    use crate::template::{Expression, Function};

    #[tokio::test]
    async fn should_concat(){
        let a=Concat{};
        let input=vec![Input::Continue(ContinueInput{name:"one".to_string(),value:"123".to_string(),data_type:DataType::PositiveInteger})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![Expression::Variable("one".to_string(),Option::Some(DataType::PositiveInteger)),Expression::Constant(Value::String("hello".to_string()))],&context).await;
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
        ],&context).await;
        assert_eq!(result,Value::PositiveInteger(5));
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
        ],&context).await;
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
        ],&context).await;
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
        ],&context).await;
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
        ],&context).await;
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
        ],&context).await;
        assert_eq!(result,Value::PositiveInteger(2));
    }
}
