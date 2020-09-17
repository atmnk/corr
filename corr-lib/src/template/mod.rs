pub mod json;
pub mod text;
pub mod functions;
pub mod parser;
use crate::core::{DataType, runtime::Context, Value, runtime::IO, Variable};
use std::fmt::Debug;
use async_trait::async_trait;
use crate::template::functions::get_function;

#[derive(Clone,Debug,PartialEq)]
pub enum Expression{
    Variable(String,Option<DataType>),
    Function(String,Vec<Expression>),
    Constant(Value)
}
#[derive(Clone,Debug,PartialEq)]
pub struct VariableReferenceName {
    pub parts:Vec<String>
}
// #[derive(Clone,Debug,PartialEq)]
// pub struct ExpressionBlock{
//     expression:Expression
// }
#[async_trait]
pub trait Function:Debug+Sync+Send{
    async fn evaluate(&self,args:Vec<Expression>,context:&Context)->Value;
}
impl Expression{
    pub(crate) async fn evaluate(&self, context: &Context) -> Value {
        match self {
            Expression::Variable(name,data_type)=>{
                let vv=context.read(Variable{
                    name:name.clone(),
                    data_type:data_type.clone()
                }).await;
                vv.value
            },
            Expression::Function(func,args)=>{
                get_function(func.as_str()).evaluate(args.clone(),context).await
            },
            Expression::Constant(val)=>{
                val.clone()
            }
        }
    }
}
//Functions

#[cfg(test)]
mod tests{
    use crate::template::{Expression};
    use crate::core::{DataType, Value};
    use crate::core::proto::{Input, ContinueInput, Output, TellMeOutput};
    use std::sync::{Arc, Mutex};
    use crate::core::runtime::Context;


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
        let expr=Expression::Function("concat".to_string(),vec![Expression::Constant(Value::String("Hello".to_string()))]);
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=expr.evaluate(&context).await;
        assert_eq!(result,Value::String("Hello".to_string()));
    }
    #[tokio::test]
    async fn should_evaluate_expression_when_constant(){
        let expr=Expression::Constant(Value::PositiveInteger(12));
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=expr.evaluate(&context).await;
        assert_eq!(result,Value::PositiveInteger(12));
    }
}
