
use async_trait::async_trait;
use crate::template::VariableReferenceName;
use crate::core::runtime::Context;
use crate::core::Value;
use rdbc_async::sql::ResultSet;
// use formdata::FormData;
// use std::collections::HashMap;

pub mod parser;
#[derive(Clone,Debug,PartialEq)]
pub enum ExtractableObject{
    WithVariableReference(VariableReferenceName),
    WithForLoop(Box<ExtractableForLoop>),
    WithMapObject(ExtractableMapObject),
    WithFixedArray(Vec<ExtractableObject>)
}
#[derive(Clone,Debug,PartialEq)]
pub enum ExtractableMapObject{
    WithPairs(Vec<ExtractablePair>)
}

#[derive(Clone,Debug,PartialEq)]
pub struct ExtractableForLoop{
    on:VariableReferenceName,
    with: VariableReferenceName,
    inner: ExtractableObject
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
impl Extractable<Box<dyn rdbc_async::sql::ResultSet>> for ExtractableObject{
    async fn extract_from(&self, _context: &Context, _value: Box<dyn ResultSet>) {
        todo!()
    }
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
            },
            ExtractableObject::WithForLoop(efl)=>{
                efl.extract_from(context,value).await
            }
            ExtractableObject::WithFixedArray(vec_templates)=>{
                vec_templates.extract_from(context,value).await
            }
        }
    }
}
// #[async_trait]
// impl Extractable<FormData> for ExtractableObject{
//     async fn extract_from(&self, context: &Context, value: FormData) {
//         match self {
//             ExtractableObject::WithVariableReference(vrn)=>{
//                 let mut hm = HashMap::new();
//                 for (name,dv) in value.fields{
//                     hm.insert(name.clone(),Value::String(dv.clone()));
//                 }
//                 context.define(vrn.to_string(),Value::Map(hm)).await;
//             },
//             ExtractableObject::WithMapObject(mpobj)=>{
//                 mpobj.extract_from(context,value).await;
//                 // mpobj.extract_from(context,value).await
//             },
//             _=>{
//                 panic!("Not Supported")
//             }
//             // ExtractableObject::WithForLoop(efl)=>{
//             //     efl.extract_from(context,value).await
//             // }
//             // ExtractableObject::WithFixedArray(vec_templates)=>{
//             //     vec_templates.extract_from(context,value).await
//             // }
//         }
//     }
// }
#[async_trait]
impl Extractable<serde_json::Value> for Vec<ExtractableObject>{
    async fn extract_from(&self, context: &Context, value: serde_json::Value) {
        if let serde_json::Value::Array(vals)= value {
            let mut index = 0;
            for inner in self {
                if let Some(val) = vals.get(index){
                    inner.extract_from(context,val.clone()).await
                } else {
                    inner.extract_from(context,serde_json::Value::Null).await
                }
                index =  index + 1
            }
        } else {
            for inner in self {
                inner.extract_from(context,serde_json::Value::Null).await
            }
        }
    }
}
#[async_trait]
impl Extractable<serde_json::Value> for ExtractableForLoop{
    async fn extract_from(&self, context: &Context, value: serde_json::Value) {
        if let serde_json::Value::Array(vec_val) = value{
            context.iterate_like(vec_val,self.on.clone().to_string(),self.with.clone().to_string(),async move |context,_,value|{
                self.inner.clone().extract_from(&context,value).await
            }).await;
        } else {
            context.define(self.on.to_string(),Value::Array(vec![])).await
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
// #[async_trait]
// impl Extractable<FormData> for ExtractableMapObject{
//     async fn extract_from(&self, context: &Context, value: FormData) {
//         match self {
//             ExtractableMapObject::WithPairs(pairs)=>{
//                 for pair in pairs {
//                     let val = value.clone();
//                     pair.extract_from(context,val).await
//                 }
//             }
//         }
//     }
// }
#[async_trait]
impl Extractable<serde_json::Value> for ExtractablePair{
    async fn extract_from(&self, context: &Context, value: serde_json::Value) {
        match self {
            ExtractablePair::WithKeyValue(key,value_template)=>{
                if let serde_json::Value::Object(mp)=value{
                    if let Some(property_value) = mp.get(key) {
                        value_template.extract_from(context,property_value.clone()).await
                    } else {
                        value_template.extract_from(context,serde_json::Value::Null).await
                    }
                } else {
                    value_template.extract_from(context,serde_json::Value::Null).await
                }
            }
        }
    }
}
// #[async_trait]
// impl Extractable<FormData> for ExtractablePair{
//     async fn extract_from(&self, context: &Context, value: FormData) {
//         match self {
//             ExtractablePair::WithKeyValue(key,value_template)=>{
//                 if let Some((_,val)) = value.fields.iter().find(|(index,_)|{index.eq(key)}){
//                     if let ExtractableObject::WithVariableReference(vrn) = value_template{
//                         context.define(vrn.to_string(),Value::String(val.clone())).await;
//                     }
//                 }
//             }
//         }
//     }
// }
#[cfg(test)]
mod tests{
    use crate::template::object::extractable::{ExtractablePair, Extractable, ExtractableMapObject, ExtractableObject, ExtractableForLoop};
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
    async fn should_extract_extractableforloop(){
        let text=r#"names.for (name)=>name"#;
        let (_,efl) = ExtractableForLoop::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let mut vec_val = vec![];
        vec_val.push(serde_json::Value::String(format!("Atmaram")));
        vec_val.push(serde_json::Value::String(format!("Yogesh")));
        efl.extract_from(&context,serde_json::Value::Array(vec_val.clone())).await;
        assert_eq!(context.get_var_from_store(format!("names")).await,Option::Some(Value::from_json_value(serde_json::Value::Array(vec_val))));
    }
    #[tokio::test]
    async fn should_extract_vec_extractableobject(){
        let text=r#"[ name , place ]"#;
        let (_,efl) = Vec::<ExtractableObject>::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let mut vec_val = vec![];
        vec_val.push(serde_json::Value::String(format!("Atmaram")));
        vec_val.push(serde_json::Value::String(format!("Pune")));
        efl.extract_from(&context,serde_json::Value::Array(vec_val.clone())).await;
        assert_eq!(context.get_var_from_store(format!("name")).await,Option::Some(Value::String(format!("Atmaram"))));
        assert_eq!(context.get_var_from_store(format!("place")).await,Option::Some(Value::String(format!("Pune"))));

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
    #[tokio::test]
    async fn should_extract_extractableobject_when_extractableforloop(){
        let text=r#"object names.for (name)=>name"#;
        let (_,eo) = ExtractableObject::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let mut vec_val = vec![];
        vec_val.push(serde_json::Value::String(format!("Atmaram")));
        vec_val.push(serde_json::Value::String(format!("Yogesh")));
        eo.extract_from(&context,serde_json::Value::Array(vec_val.clone())).await;
        assert_eq!(context.get_var_from_store(format!("names")).await,Option::Some(Value::from_json_value(serde_json::Value::Array(vec_val))));

    }
    #[tokio::test]
    async fn should_extract_extractableobject_when_fixedarray(){
        let text=r#"object [ name , place ]"#;
        let (_,eo) = ExtractableObject::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let mut vec_val = vec![];
        vec_val.push(serde_json::Value::String(format!("Atmaram")));
        vec_val.push(serde_json::Value::String(format!("Pune")));
        eo.extract_from(&context,serde_json::Value::Array(vec_val.clone())).await;
        assert_eq!(context.get_var_from_store(format!("name")).await,Option::Some(Value::String(format!("Atmaram"))));
        assert_eq!(context.get_var_from_store(format!("place")).await,Option::Some(Value::String(format!("Pune"))));

    }
}
