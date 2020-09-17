pub mod parser;
use crate::core::{Variable, Value};
use crate::template::Expression;
use crate::core::runtime::{Context, IO};
use async_trait::async_trait;
#[derive(Clone,Debug,PartialEq)]
pub enum EJson{
    Variable(Variable),
    StaticArray(Vec<EJson>),
    DynamicArray(Variable,Variable,Box<EJson>),
    Object(Vec<EPair>)
}
#[derive(Clone,Debug,PartialEq)]
pub struct EPair{
    pub key:String,
    pub value:EJson
}
#[async_trait]
pub trait Extractable{
    async fn extract_from(&self,context:&Context,value:serde_json::Value);
}
#[async_trait]
impl Extractable for EJson{
    async fn extract_from(&self, context: &Context, value: serde_json::Value) {
        match self {
            EJson::Variable(var)=>{
                context.define(var.name.clone(),Value::from_json_value(value)).await;
            },
            EJson::Object(pairs)=>{
                if let serde_json::Value::Object(obj)=value{
                    for pair in  pairs {
                        if let Some(val)=obj.get(pair.key.clone().as_str()) {
                            pair.value.extract_from(context,val.clone()).await
                        }
                    }
                }
            },
            EJson::StaticArray(arr)=>{
                if let serde_json::Value::Array(vec_val)=value{
                    let mut i = 0 ;
                    for value in arr {
                        if let Some(val)=vec_val.get(i){
                            value.extract_from(context,val.clone()).await
                        }
                    }
                };

            },
            EJson::DynamicArray(with,on,inner)=>{
                if let serde_json::Value::Array(vec_val)=value{
                    context.iterate_like(vec_val, on.name.clone(), with.name.clone(), async move |context, i, val|{
                        inner.extract_from(&context,val).await;
                    }).await;
                };
            }
        }
    }
}
