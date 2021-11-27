// pub mod json;
pub mod object;
pub mod text;
pub mod functions;
pub mod parser;
pub mod rest;
pub mod form;
use crate::core::{DataType, runtime::Context, Value, runtime::IO, Variable};
use std::fmt::Debug;
use async_trait::async_trait;
use crate::template::functions::*;
use crate::template::text::Text;
use crate::template::object::FillableObject;
use std::sync::Arc;

#[derive(Debug, Clone,PartialEq)]
pub enum Assignable{
    Expression(Expression),
    FillableText(Text)
}
#[async_trait]
impl Fillable<Value> for Assignable{
    async fn fill(&self, context: &Context) -> Value {
        match self {
            Assignable::Expression(expr)=>expr.fill(context).await,
            Assignable::FillableText(txt)=>Value::String(txt.fill(context).await),
        }
    }
}
#[async_trait]
pub trait Fillable<T>{
    async fn fill(&self,context:&Context)->T;
}
#[derive(Clone,Debug,PartialEq)]
pub enum BinaryOperator {
    And,
    Or,
    Add,
    Subtract,
    Divide,
    Multiply,
    Mod,
    Equal,
    GreaterThanEqual,
    LessThanEqual,
    LessThan,
    GreaterThan,
    NotEqual,
    // Range,
    // Increment,
    // Decrement
}
#[derive(Clone,Debug,PartialEq)]
pub enum UnaryPostOperator {
    Increment,
    Decrement
}
#[derive(Clone,Debug,PartialEq)]
pub enum UnaryPreOperator {
    Not
    // Range,
    // Increment,
    // Decrement
}
impl BinaryOperator {
    pub fn get_function(&self)->Arc<dyn Function>{
        match self {
            BinaryOperator::And=>{
                Arc::new(LogicalAnd{})
            },
            BinaryOperator::Or=>{
                Arc::new(LogicalOr{})
            },
            BinaryOperator::Add=>{
                Arc::new(Add{})
            },
            BinaryOperator::Subtract=>{
                Arc::new(Subtract{})
            },
            BinaryOperator::Multiply=>{
                Arc::new(Multiply{})
            },
            BinaryOperator::Divide=>{
                Arc::new(Divide{})
            },
            BinaryOperator::Mod=>{
                Arc::new(Mod{})
            },
            BinaryOperator::Equal=>{
                Arc::new(Equal{})
            },
            BinaryOperator::GreaterThanEqual=>{
                Arc::new(GreaterThanEqual{})
            },BinaryOperator::LessThanEqual=>{
                Arc::new(LessThanEqual{})
            },BinaryOperator::GreaterThan=>{
                Arc::new(GreaterThan{})
            },BinaryOperator::LessThan=>{
                Arc::new(LessThan{})
            },
            BinaryOperator::NotEqual=>{
                Arc::new(NotEqual{})
            },
        }
    }
}
impl UnaryPostOperator {
    pub fn get_function(&self)->Arc<dyn Function>{
        match self {
            UnaryPostOperator::Increment=>{
                Arc::new(Increment{})
            },
            UnaryPostOperator::Decrement=>{
                Arc::new(Decrement{})
            }
        }
    }
}
impl UnaryPreOperator {
    pub fn get_function(&self)->Arc<dyn Function>{
        match self {
            UnaryPreOperator::Not=>{
                Arc::new(LogicalNot{})
            }
        }
    }
}
#[derive(Clone,Debug,PartialEq)]
pub enum Operator{
    Binary(BinaryOperator),
    UnaryPost(UnaryPostOperator),
    UnaryPre(UnaryPreOperator)
}
impl Operator {
    pub fn get_function(&self)->Arc<dyn Function>{
        match self {
            Operator::Binary(bo)=>{
                bo.get_function()
            },
            Operator::UnaryPost(uo)=>{
                uo.get_function()
            },
            Operator::UnaryPre(uo)=>{
                uo.get_function()
            }
        }
    }
}
#[derive(Clone,Debug,PartialEq)]
pub enum Expression{
    FillableObject(Box<FillableObject>),
    Function(String,Vec<Expression>),
    Variable(String,Option<DataType>),
    DotFunction(Box<Expression>,String,Vec<Expression>),
    Constant(Value),
    Operator(Operator, Vec<Expression>)
}
#[derive(Clone,Debug,PartialEq)]
pub struct VariableReferenceName {
    pub parts:Vec<String>
}
#[derive(Clone,Debug,PartialEq)]
struct FunctionCallChain {
    left:Expression,
    function_chain:Vec<(String, Vec<Expression>)>
}
impl FunctionCallChain {
    // pub fn from(vrn:VariableReferenceName)->Self{
    //     let mut parts = vrn.parts.clone();
    //     let opt_last = parts.pop();
    //     if let Some(last) = opt_last{
    //         if parts.len()>0 {
    //             Self{
    //                 left :Option::Some(Expression::Variable(parts.join("."),Option::None)),
    //                 function:last
    //             }
    //
    //         } else {
    //             Self{
    //                 left :Option::None,
    //                 function:last
    //             }
    //         }
    //     } else {
    //         panic!("Impposiible VRN")
    //     }
    // }
    // pub fn from_expr(expr:Expression,func:String)->Self{
    //     Self{
    //         left:Option::Some(expr),
    //         function:func
    //     }
    // }
}

impl VariableReferenceName {
    pub fn to_string(&self)->String{
        return self.parts.join(".")
    }
    pub fn from(str:&str)->VariableReferenceName{
        let mut parts=vec![];
        for part in str.split("."){
            parts.push(part.to_string());
        }
        VariableReferenceName{
            parts
        }
    }
}
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
            Expression::DotFunction(on,func,args)=>{
                let mut cloned = args.clone();
                cloned.insert(0,on.as_ref().clone());
                get_function(func.as_str()).evaluate(cloned,context).await
            },
            Expression::Constant(val)=>{
                val.clone()
            },
            Expression::Operator(op,args)=>{
                op.get_function().evaluate(args.clone(),context).await
            }
            Expression::FillableObject(fo)=>{
                fo.fill(context).await
            }
        }
    }
}
//Functions

#[cfg(test)]
mod tests{
    use crate::template::{Expression, Assignable, Fillable};
    use crate::core::{DataType, Value};
    use crate::core::proto::{Input, ContinueInput, Output, TellMeOutput};
    use std::sync::{Arc, Mutex};
    use crate::core::runtime::Context;
    use crate::parser::Parsable;

    #[tokio::test]
    async fn should_fill_assignable_when_expression(){
        let txt = r#"name"#;
        let (_,assbl) = Assignable::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=assbl.fill(&context).await;
        assert_eq!(result,Value::String("Atmaram".to_string()));
    }

    #[tokio::test]
    async fn should_fill_assignable_when_fillabletext(){
        let txt = r#"text `Hello <%name%>`"#;
        let (_,assbl) = Assignable::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=assbl.fill(&context).await;
        assert_eq!(result,Value::String("Hello Atmaram".to_string()));
    }

    #[tokio::test]
    async fn should_fill_assignable_when_fillableobject(){
        let txt = r#"object name"#;
        let (_,assbl) = Assignable::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=assbl.fill(&context).await;
        assert_eq!(result,Value::String("Atmaram".to_string()));
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
