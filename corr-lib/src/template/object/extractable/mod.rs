
use async_trait::async_trait;
use crate::template::VariableReferenceName;
use crate::core::runtime::Context;
use crate::core::Value;

pub mod parser;
#[derive(Clone,Debug,PartialEq)]
pub enum ExtractableObject{
    WithVariableReference(VariableReferenceName),
    WithMapObject(ExtractableMapObject)
}
#[derive(Clone,Debug,PartialEq)]
pub enum ExtractableMapObject{
    WithPairs(Vec<ExtractablePair>)
}
#[derive(Clone,Debug,PartialEq)]
pub enum ExtractablePair{
    WithKeyValue(String,ExtractableObject)
}
#[async_trait]
pub trait Extractable<T>{
    async fn extract_from(&self,context:&Context,value:T);
}
#[async_trait]
impl Extractable<serde_json::Value> for ExtractableObject{
    async fn extract_from(&self, context: &Context, value: serde_json::Value) {
        match self {
            ExtractableObject::WithVariableReference(vrn)=>{
                context.define(vrn.to_string(),Value::from_json_value(value)).await
            },
            ExtractableObject::WithMapObject(mpobj)=>{
                mpobj.extract_from(context,value).await
            }
        }
    }
}
#[async_trait]
impl Extractable<serde_json::Value> for ExtractableMapObject{
    async fn extract_from(&self, context: &Context, value: serde_json::Value) {
        match self {
            ExtractableMapObject::WithPairs(pairs)=>{
                    for pair in pairs {
                        pair.extract_from(context,value.clone()).await
                    }
            }
        }
    }
}
#[async_trait]
impl Extractable<serde_json::Value> for ExtractablePair{
    async fn extract_from(&self, context: &Context, value: serde_json::Value) {
        match self {
            ExtractablePair::WithKeyValue(key,value_template)=>{
                if let serde_json::Value::Object(mp)=value{
                    if let Some(property_value) = mp.get(key) {
                        value_template.extract_from(context,property_value.clone()).await
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests{
    use crate::template::object::extractable::{ExtractablePair, Extractable, ExtractableMapObject, ExtractableObject};
    use crate::core::{Value};
    use crate::core::proto::{Output};
    use std::sync::{Arc, Mutex};
    use crate::core::runtime::{Context};
    use crate::parser::Parsable;
    use serde_json::Number;

    #[tokio::test]
    async fn should_extract_extractablepair(){
        let text=r#""name":name"#;
        let (_,ep) = ExtractablePair::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let mut hm = serde_json::Map::new();
        hm.insert(format!("name"),serde_json::Value::String(format!("Hello")));
        ep.extract_from(&context,serde_json::Value::Object(hm)).await;
        assert_eq!(context.get_var_from_store(format!("name")).await,Option::Some(Value::String(format!("Hello"))))
    }
    #[tokio::test]
    async fn should_extract_extractablemapobject(){
        let text=r#"{"name":name,"place":place}"#;
        let (_,ep) = ExtractableMapObject::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let mut hm = serde_json::Map::new();
        hm.insert(format!("name"),serde_json::Value::String(format!("Atmaram")));
        hm.insert(format!("place"),serde_json::Value::String(format!("Mumbai")));
        ep.extract_from(&context,serde_json::Value::Object(hm)).await;
        assert_eq!(context.get_var_from_store(format!("name")).await,Option::Some(Value::String(format!("Atmaram"))));
        assert_eq!(context.get_var_from_store(format!("place")).await,Option::Some(Value::String(format!("Mumbai"))));
    }
    #[tokio::test]
    async fn should_extract_extractableobject_when_variablereference(){
        let text=r#"object name"#;
        let (_,ep) = ExtractableObject::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        ep.extract_from(&context,serde_json::Value::String(format!("Atmaram"))).await;
        assert_eq!(context.get_var_from_store(format!("name")).await,Option::Some(Value::String(format!("Atmaram"))))
    }
    #[tokio::test]
    async fn should_extract_extractableobject_when_extractablemapobject(){
        let text=r#"object {"name":name,"age":age}"#;
        let (_,ep) = ExtractableObject::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let mut hm = serde_json::Map::new();
        hm.insert(format!("name"),serde_json::Value::String(format!("Atmaram")));
        hm.insert(format!("age"),serde_json::Value::Number(Number::from(34)));
        ep.extract_from(&context,serde_json::Value::Object(hm)).await;
        assert_eq!(context.get_var_from_store(format!("name")).await,Option::Some(Value::String(format!("Atmaram"))));
        assert_eq!(context.get_var_from_store(format!("age")).await,Option::Some(Value::PositiveInteger(34)));

    }
}
