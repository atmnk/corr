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
//Concat Function
#[derive(Debug,Clone,PartialEq)]
pub struct Concat;

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


//Add Function
#[derive(Debug,Clone,PartialEq)]
pub struct Add;

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

//Multiply Function
#[derive(Debug,Clone,PartialEq)]
pub struct Multiply;

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


//Subtarct Function
#[derive(Debug,Clone,PartialEq)]
pub struct Subtract;

//Fake Function
#[derive(Debug,Clone,PartialEq)]
pub struct FakeValue;

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
pub fn get_function(name:&str)->Arc<dyn Function>{
    match name {
        "add"=>{
            Arc::new(Add{})
        },
        "sub"=>{
            Arc::new(Subtract{})
        },
        "mul"=>{
            Arc::new(Multiply{})
        },
        "div"=>{
            Arc::new(Divide{})
        },
        "concat"=>{
            Arc::new(Concat{})
        },
        "from_json"=>{
            Arc::new(FromJson{})
        },
        "fake"=>{
            Arc::new(FakeValue{})
        },
        "encode"=>{
            Arc::new(Encode{})
        }
        _=>Arc::new(Concat{})
    }
}
#[cfg(test)]

mod tests{
    use crate::core::{DataType, Value};
    use crate::core::proto::{Input, ContinueInput, Output, TellMeOutput};
    use std::sync::{Arc, Mutex};
    use crate::template::functions::{Concat, Add, Subtract, Multiply, Divide, get_fake};
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
