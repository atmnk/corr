pub mod text;
use crate::core::{DataType, Context, Value, IO, Variable};
use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Clone,Debug)]
pub enum Expression{
    Variable(String,Option<DataType>),
    Function(Arc<dyn Function>,Vec<Expression>),
    Constant(Value)
}
#[async_trait]
pub trait Function:Debug+Sync+Send{
    async fn evaluate(&self,args:Vec<Expression>,context:&Context)->Value;
}
impl Expression{
    async fn evaluate(&self, context: &Context) -> Value {
        match self {
            Expression::Variable(name,data_type)=>{
                let vv=context.read(Variable{
                    name:name.clone(),
                    data_type:data_type.clone()
                }).await;
                vv.value
            },
            Expression::Function(func,args)=>{
                func.evaluate(args.clone(),context).await
            },
            Expression::Constant(val)=>{
                val.clone()
            }
        }
    }
}
//Functions
#[derive(Debug)]
struct Concat;
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
#[cfg(test)]
mod tests{
    use crate::template::{Concat, Function, Expression};
    use crate::core::{DataType, Value, Context};
    use crate::core::proto::{Input, ContinueInput, Output, TellMeOutput};
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn should_concat(){
        let a=Concat{};
        let input=vec![Input::Continue(ContinueInput{name:"one".to_string(),value:"123".to_string(),data_type:DataType::Long})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=a.evaluate(vec![Expression::Variable("one".to_string(),Option::Some(DataType::Long)),Expression::Constant(Value::String("hello".to_string()))],&context).await;
        assert_eq!(result,Value::String("123hello".to_string()));
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"one".to_string(),data_type:DataType::Long}));
    }

    #[tokio::test]
    async fn should_evaluate_expression_when_variable(){
        let expr=Expression::Variable("name".to_string(),Option::Some(DataType::String));
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=expr.evaluate(&context).await;
        assert_eq!(result,Value::String("Atmaram".to_string()));
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));
    }
    #[tokio::test]
    async fn should_evaluate_expression_when_function(){
        let expr=Expression::Function(Arc::new(Concat{}),vec![Expression::Constant(Value::String("Hello".to_string()))]);
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=expr.evaluate(&context).await;
        assert_eq!(result,Value::String("Hello".to_string()));
    }
    #[tokio::test]
    async fn should_evaluate_expression_when_constant(){
        let expr=Expression::Constant(Value::Long(12));
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=expr.evaluate(&context).await;
        assert_eq!(result,Value::Long(12));
    }
}
