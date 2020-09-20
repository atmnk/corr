pub mod parser;
use crate::template::{Expression, VariableReferenceName};
use crate::core::runtime::{Context};
use async_trait::async_trait;
use crate::core::Value;

#[derive(Clone,Debug,PartialEq)]
pub struct Text{
    pub blocks:Vec<Block>
}
#[derive(Clone,Debug,PartialEq)]
pub enum Block{
    Scriplet(Scriplet),
    Text(String),
}
#[derive(Clone,Debug,PartialEq)]
pub enum Scriplet{
    Expression(Expression),
    ForLoop(TextForLoop)
}
#[derive(Clone,Debug,PartialEq)]
pub enum TextForLoop{
    WithVariableReference(VariableReferenceName,Option<VariableReferenceName>,Option<VariableReferenceName>,Box<TextLoopInnerTemplate>)
}
#[derive(Clone,Debug,PartialEq)]
pub enum TextLoopInnerTemplate {
    Expression(Expression),
    ForLoop(TextForLoop),
    Blocks(Vec<Block>)
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
            },
            Scriplet::ForLoop(tfl)=>{
                tfl.fill(context).await
            }
        }
    }
}
#[async_trait]
impl Fillable for TextForLoop{
    async fn fill(&self, context: &Context) -> String {
        match self {
            TextForLoop::WithVariableReference(on,with,opt_index,inner)=>{
                context.iterate(on.to_string(),with.clone().map(|val|val.to_string()),
                                async move |context,i|{
                                    if let Some(index) = opt_index {
                                        context.define(index.to_string(),Value::PositiveInteger(i)).await;
                                    }
                                    inner.fill(&context).await
                }).await.join("")
            }
        }
    }
}
#[async_trait]
impl Fillable for TextLoopInnerTemplate {
    async fn fill(&self, context: &Context) -> String {
        match self {
            TextLoopInnerTemplate::ForLoop(tfl)=>tfl.fill(context).await,
            TextLoopInnerTemplate::Expression(expr)=>expr.fill(context).await,
            TextLoopInnerTemplate::Blocks(blocks)=> {
                let mut values = vec![];
                for block in blocks {
                    values.push(block.fill(context).await)
                }
                values.join("")
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

#[cfg(test)]
mod tests{
    use crate::template::text::{Scriplet, Fillable, Block, Text, TextForLoop, TextLoopInnerTemplate};
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
    async fn should_fill_scriplet_with_for_loop(){
        let txt = r#"<%names.for(name)=>name%>"#;
        let (_,scriplet)=Scriplet::parser(txt).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"names::length".to_string(),value:"2".to_string(),data_type:DataType::PositiveInteger}),
            Input::new_continue(format!("name"),format!("Atmaram"),DataType::String),
            Input::new_continue(format!("name"),format!("Yogesh"),DataType::String)
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=scriplet.fill(&context).await;
        assert_eq!(result,"AtmaramYogesh".to_string());
    }

    #[tokio::test]
    async fn should_fill_textforloop(){
        let text=r#"names.for %>Twise<%"#;
        let (_,tfl)=TextForLoop::parser(text).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"names::length".to_string(),value:"2".to_string(),data_type:DataType::PositiveInteger}),
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=tfl.fill(&context).await;
        assert_eq!(result,"TwiseTwise".to_string());
    }

    #[tokio::test]
    async fn should_fill_textloopinnertemplate_when_for_loop(){
        let text=r#"names.for %>Twise<%"#;
        let (_,tlit)=TextLoopInnerTemplate::parser(text).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"names::length".to_string(),value:"2".to_string(),data_type:DataType::PositiveInteger}),
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=tlit.fill(&context).await;
        assert_eq!(result,"TwiseTwise".to_string());
    }

    #[tokio::test]
    async fn should_fill_textloopinnertemplate_when_expression(){
        let text=r#"name%"#;
        let (_,tlit)=TextLoopInnerTemplate::parser(text).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String}),
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=tlit.fill(&context).await;
        assert_eq!(result,"Atmaram".to_string());
    }

    #[tokio::test]
    async fn should_fill_textloopinnertemplate_when_blocks(){
        let text=r#"%>Hello <%name%><%"#;
        let (_,tlit)=TextLoopInnerTemplate::parser(text).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String}),
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let result=tlit.fill(&context).await;
        assert_eq!(result,"Hello Atmaram".to_string());
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
}
