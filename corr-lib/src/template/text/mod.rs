pub mod parser;
use crate::template::{Expression, ExpressionBlock};
use crate::core::runtime::{Context};
use async_trait::async_trait;

#[derive(Clone,Debug,PartialEq)]
pub struct Text{
    pub blocks:Vec<Block>
}
#[derive(Clone,Debug,PartialEq)]
pub enum Block{
    Final(String),
    Expression(ExpressionBlock),
    Loop(LoopBlock)
}
#[derive(Clone,Debug,PartialEq)]
pub struct LoopBlock{
    on:String,
    with:String,
    inner:Vec<Block>
}

#[async_trait]
pub trait Fillable{
    async fn fill(&self,context:&Context)->String;
}
#[async_trait]
impl Fillable for Text{
    async fn fill(&self, context: &Context) -> String {
        let mut ret="".to_string();
        for block in self.blocks.iter() {
            ret.push_str(block.fill(context).await.as_str());
        }
        ret
    }
}
#[async_trait]
impl Fillable for Block{
    async fn fill(&self, context: &Context) -> String {
        match self {
            Block::Final(val)=>val.clone(),
            Block::Expression(expr)=>expr.fill(context).await,
            Block::Loop(lb)=>lb.fill(context).await,
        }
    }
}
#[async_trait]
impl Fillable for Expression{
    async fn fill(&self, context: &Context) -> String {
        self.evaluate(context).await.to_string()
    }
}
#[async_trait]
impl Fillable for ExpressionBlock{
    async fn fill(&self, context: &Context) -> String {
        self.expression.fill(context).await
    }
}
#[async_trait]
impl Fillable for LoopBlock{
    async fn fill(&self, context: &Context) -> String {
        context.iterate(self.on.clone(),self.with.clone(),async move|context|{
            let mut buffer="".to_string();
            for block in &self.inner{
                buffer.push_str(block.fill(&context).await.as_str())
            }
            buffer
        }).await.join("")
    }
}
#[cfg(test)]
mod tests{
    use crate::template::text::{LoopBlock, Block, Fillable, ExpressionBlock, Text};
    use crate::core::proto::{Input, ContinueInput, Output, TellMeOutput};
    use crate::core::{DataType, Value};
    use std::sync::{Arc, Mutex};
    use crate::template::Expression;
    use crate::core::runtime::Context;

    #[tokio::test]
    async fn should_fill_loop_block(){
        let lb=LoopBlock{
            on:"names".to_string(),
            with:"name".to_string(),
            inner:vec![Block::Final("hello".to_string())]
        };
        let input=vec![Input::Continue(ContinueInput{name:"names::length".to_string(),value:"3".to_string(),data_type:DataType::PositiveInteger})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=lb.fill(&context).await;
        assert_eq!(result,"hellohellohello".to_string());
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"names::length".to_string(),data_type:DataType::PositiveInteger}));
    }
    #[tokio::test]
    async fn should_fill_expression_block(){
        let eb=ExpressionBlock{
            expression:Expression::Variable("name".to_string(),Option::Some(DataType::String))
        };
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=eb.fill(&context).await;
        assert_eq!(result,"Atmaram".to_string());
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));
    }
    #[tokio::test]
    async fn should_fill_block(){
        let b=Block::Final("hello".to_string());
        let input=vec![];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=b.fill(&context).await;
        assert_eq!(result,"hello".to_string());
    }
    #[tokio::test]
    async fn should_fill_text(){
        let txt=Text{
            blocks:vec![
                Block::Final("Hello".to_string()),
                Block::Expression(
                    ExpressionBlock{
                        expression:Expression::Constant(Value::String("World".to_string()))
                    }
                ),
                Block::Loop(
                    LoopBlock{
                        on:"names".to_string(),
                        with:"name".to_string(),
                        inner:vec![Block::Expression(ExpressionBlock{ expression: Expression::Variable("name".to_string(),Option::Some(DataType::String))})]
                    }
                )
            ]
        };
        let input=vec![
            Input::Continue(ContinueInput{
                name:"names::length".to_string(),
                value:"2".to_string(),
                data_type:DataType::PositiveInteger
            }),
            Input::Continue(ContinueInput{
                name:"name".to_string(),
                value:"Atmaram".to_string(),
                data_type:DataType::String
            }),
            Input::Continue(ContinueInput{
                name:"name".to_string(),
                value:"Atiksh".to_string(),
                data_type:DataType::String
            })
        ];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=txt.fill(&context).await;
        assert_eq!(result,"HelloWorldAtmaramAtiksh".to_string());
        assert_eq!(buffer.lock().unwrap().len(),3);
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"names::length".to_string(),data_type:DataType::PositiveInteger}));
        assert_eq!(buffer.lock().unwrap().get(1).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));
        assert_eq!(buffer.lock().unwrap().get(2).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));
    }
}
