use crate::template::Expression;
use crate::core::{Context, IO, Variable};
use async_trait::async_trait;
#[derive(Clone,Debug)]
pub struct Text{
    pub blocks:Vec<Block>
}
#[derive(Clone,Debug)]
pub enum Block{
    Final(String),
    Expression(ExpressionBlock),
    Loop(LoopBlock)
}
#[derive(Clone,Debug)]
pub struct LoopBlock{
    on:String,
    with:String,
    inner:Vec<Block>
}
#[derive(Clone,Debug)]
pub struct ExpressionBlock{
    expression:Expression
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
        match self {
            Expression::Variable(name,data_type)=>{
                let vv=context.read(Variable{
                    name:name.clone(),
                    data_type:data_type.clone()
                }).await;
                vv.value.to_string()
            },
            Expression::Function(func,args)=>{
                func.evaluate(args.clone(),context).to_string()
            }
        }
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