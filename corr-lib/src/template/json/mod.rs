pub mod parser;
pub mod extractable;
use crate::core::{Variable, Value};
use crate::template::Expression;
use crate::core::runtime::{Context, IO};
use async_trait::async_trait;
#[derive(Clone,Debug,PartialEq)]
pub enum Json{
    Expression(Expression),
    StaticArray(Vec<Json>),
    DynamicArray(Variable,Variable,Box<Json>,Option<Variable>),
    Object(Vec<Pair>)
}
#[derive(Clone,Debug,PartialEq)]
pub struct Pair{
    pub key:String,
    pub value:Json
}
#[async_trait]
pub trait FillableJson{
    async fn json_fill(&self,context:&Context)->serde_json::Value;
}

#[async_trait]
impl FillableJson for Json{
    async fn json_fill(&self, context: &Context) -> serde_json::Value {
        match self {
            Json::Expression(expr)=>{
                expr.json_fill(context).await
            },
            Json::StaticArray(arr)=>{
                let mut new_vec=Vec::new();
                for val in arr {
                    new_vec.push(val.json_fill(context).await)
                }
                serde_json::Value::Array(new_vec)
            },
            Json::DynamicArray(with,on,inner,index_var)=>{
                let res=context.iterate(on.name.clone(),with.name.clone(),async move |context,i|{
                    if let Some(iv)=index_var.clone(){
                        context.define(iv.name,Value::PositiveInteger(i)).await
                    }
                    inner.json_fill(&context).await
                }).await;
                serde_json::Value::Array(res)
            },
            Json::Object(vec)=>{
                let mut mp=serde_json::Map::new();
                for pair in vec {
                    mp.insert(pair.key.clone(),pair.value.json_fill(context).await);
                }
                serde_json::Value::Object(mp)
            }
        }
    }
}
#[async_trait]
impl FillableJson for Expression{
    async fn json_fill(&self, context: &Context) -> serde_json::Value {
        match self {
            Expression::Constant(val)=>{
                val.to_json_value()
            },
            Expression::Variable(var,dt)=>{
                let vv=context.read(Variable{
                    name:var.clone(),
                    data_type:dt.clone()
                }).await;
                vv.value.to_json_value()
            },
            Expression::Function(_,_)=>{
                self.evaluate(context).await.to_json_value()
            }
        }
    }
}