pub mod parser;
use crate::template::{Expression};
use crate::core::runtime::{Context};
use async_trait::async_trait;

#[derive(Clone,Debug,PartialEq)]
pub struct Text{
    pub blocks:Vec<Block>
}
#[derive(Clone,Debug,PartialEq)]
pub enum Block{
    Scriplet(Scriplet),
    Text(String),
    //Expression(Expression),
    //Final(String),
    //Expression(ExpressionBlock),
    //Loop(LoopBlock)
}
#[derive(Clone,Debug,PartialEq)]
pub enum Scriplet{
    Expression(Expression),
    //For(For)
}
// #[derive(Clone,Debug,PartialEq)]
// pub struct LoopBlock{
//     on:Variable,
//     with:Variable,
//     inner:Vec<Block>,
//     index_var:Option<Variable>
// }

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
            Block::Scriplet(scriplet)=>{
                scriplet.fill(context).await
            },
            Block::Text(str)=>{
                str.clone()
            }
        }
    }
}
#[async_trait]
impl Fillable for Scriplet{
    async fn fill(&self, context: &Context) -> String {
        match self {
            Scriplet::Expression(expr)=>{
                expr.fill(context).await
            }
        }
    }
}
#[async_trait]
impl Fillable for Expression{
    async fn fill(&self, context: &Context) -> String {
        self.evaluate(context).await.to_string()
    }
}
// #[async_trait]
// impl Fillable for ExpressionBlock{
//     async fn fill(&self, context: &Context) -> String {
//         self.expression.fill(context).await
//     }
// }
// #[async_trait]
// impl Fillable for LoopBlock{
//     async fn fill(&self, context: &Context) -> String {
//         context.iterate(self.on.clone().name,self.with.clone().name,async move|context,i|{
//             if let Some(iv)=self.index_var.clone(){
//                 context.define(iv.name,Value::PositiveInteger(i)).await
//             }
//             let mut buffer="".to_string();
//             for block in &self.inner{
//                 buffer.push_str(block.fill(&context).await.as_str())
//             }
//             buffer
//         }).await.join("")
//     }
// }
#[cfg(test)]
mod tests{
    use crate::template::text::{Scriplet, Fillable, Block, Text};
    use crate::template::Expression;
    use crate::core::DataType;
    use crate::core::proto::{Input, ContinueInput, Output, TellMeOutput};
    use std::sync::{Arc, Mutex};
    use crate::core::runtime::Context;
    use crate::parser::Parsable;

    #[tokio::test]
    async fn should_fill_text_with_scriptlet_and_text(){
        let txt = r#"fillable text `Hello <%name%>`"#;
        let (_,fillable)=Text::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=fillable.fill(&context).await;
        assert_eq!(result,"Hello Atmaram".to_string());
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));
    }

    #[tokio::test]
    async fn should_fill_block_with_text(){
        let fillable=Block::Text("hello".to_string());
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=fillable.fill(&context).await;
        assert_eq!(result,"hello".to_string());
        assert_eq!(buffer.lock().unwrap().len(),0);

    }

    #[tokio::test]
    async fn should_fill_block_with_scriplet(){
        let fillable=Block::Scriplet(Scriplet::Expression(Expression::Variable("name".to_string(),Option::Some(DataType::String))));
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=fillable.fill(&context).await;
        assert_eq!(result,"Atmaram".to_string());
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));

    }

    #[tokio::test]
    async fn should_fill_scriplet_with_expression(){
            let scriplet=Scriplet::Expression(Expression::Variable("name".to_string(),Option::Some(DataType::String)));
            let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
            let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
            let context=Context::mock(input,buffer.clone());
            let result=scriplet.fill(&context).await;
            assert_eq!(result,"Atmaram".to_string());
            assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));

    }
    #[tokio::test]
    async fn should_fill_expression(){
        let expression=Expression::Variable("name".to_string(),Option::Some(DataType::String));
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=expression.fill(&context).await;
        assert_eq!(result,"Atmaram".to_string());
        assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));

    }
    // use crate::template::text::{LoopBlock, Block, Fillable, ExpressionBlock, Text};
    // use crate::core::proto::{Input, ContinueInput, Output, TellMeOutput};
    // use crate::core::{DataType, Value, Variable};
    // use std::sync::{Arc, Mutex};
    // use crate::template::Expression;
    // use crate::core::runtime::Context;
    //
    // #[tokio::test]
    // async fn should_fill_loop_block(){
    //     let lb=LoopBlock{
    //         on:Variable{name:"names".to_string(),data_type:Option::None},
    //         with:Variable{name:"name".to_string(),data_type:Option::None},
    //         inner:vec![Block::Final("hello".to_string())],
    //         index_var:Option::None
    //     };
    //     let input=vec![Input::Continue(ContinueInput{name:"names::length".to_string(),value:"3".to_string(),data_type:DataType::PositiveInteger})];
    //     let buffer = Arc::new(Mutex::new(vec![]));
    //     let context=Context::mock(input,buffer.clone());
    //     let result=lb.fill(&context).await;
    //     assert_eq!(result,"hellohellohello".to_string());
    //     assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"names::length".to_string(),data_type:DataType::PositiveInteger}));
    // }
    // #[tokio::test]
    // async fn should_fill_expression_block(){
    //     let eb=ExpressionBlock{
    //         expression:Expression::Variable("name".to_string(),Option::Some(DataType::String))
    //     };
    //     let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
    //     let buffer = Arc::new(Mutex::new(vec![]));
    //     let context=Context::mock(input,buffer.clone());
    //     let result=eb.fill(&context).await;
    //     assert_eq!(result,"Atmaram".to_string());
    //     assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));
    // }
    // #[tokio::test]
    // async fn should_fill_block(){
    //     let b=Block::Final("hello".to_string());
    //     let input=vec![];
    //     let buffer = Arc::new(Mutex::new(vec![]));
    //     let context=Context::mock(input,buffer.clone());
    //     let result=b.fill(&context).await;
    //     assert_eq!(result,"hello".to_string());
    // }
    // #[tokio::test]
    // async fn should_fill_text(){
    //     let txt=Text{
    //         blocks:vec![
    //             Block::Final("Hello".to_string()),
    //             Block::Expression(
    //                 ExpressionBlock{
    //                     expression:Expression::Constant(Value::String("World".to_string()))
    //                 }
    //             ),
    //             Block::Loop(
    //                 LoopBlock{
    //                     on: Variable::new("names"),// "names".to_string(),
    //                     with:Variable::new("name"),
    //                     inner:vec![Block::Expression(ExpressionBlock{ expression: Expression::Variable("name".to_string(),Option::Some(DataType::String))})],
    //                     index_var:Option::None
    //                 }
    //             )
    //         ]
    //     };
    //     let input=vec![
    //         Input::Continue(ContinueInput{
    //             name:"names::length".to_string(),
    //             value:"2".to_string(),
    //             data_type:DataType::PositiveInteger
    //         }),
    //         Input::Continue(ContinueInput{
    //             name:"name".to_string(),
    //             value:"Atmaram".to_string(),
    //             data_type:DataType::String
    //         }),
    //         Input::Continue(ContinueInput{
    //             name:"name".to_string(),
    //             value:"Atiksh".to_string(),
    //             data_type:DataType::String
    //         })
    //     ];
    //     let buffer = Arc::new(Mutex::new(vec![]));
    //     let context=Context::mock(input,buffer.clone());
    //     let result=txt.fill(&context).await;
    //     assert_eq!(result,"HelloWorldAtmaramAtiksh".to_string());
    //     assert_eq!(buffer.lock().unwrap().len(),3);
    //     assert_eq!(buffer.lock().unwrap().get(0).unwrap().clone(),Output::TellMe(TellMeOutput{name:"names::length".to_string(),data_type:DataType::PositiveInteger}));
    //     assert_eq!(buffer.lock().unwrap().get(1).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));
    //     assert_eq!(buffer.lock().unwrap().get(2).unwrap().clone(),Output::TellMe(TellMeOutput{name:"name".to_string(),data_type:DataType::String}));
    // }
}
