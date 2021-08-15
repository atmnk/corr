
use async_trait::async_trait;
use crate::template::VariableReferenceName;
use crate::core::runtime::Context;
use crate::core::{Value};
use crate::template::object::extractable::Extractable;
use crate::template::rest::MultipartField;

pub mod parser;
#[derive(Clone,Debug,PartialEq)]
pub enum ExtractableForm{
    WithFields(Vec<(String,VariableReferenceName)>)
}
// #[derive(Clone,Debug,PartialEq)]
// pub enum FormField{
//     File(VariableReferenceName),
//     Text(VariableReferenceName)
// }
#[async_trait]
impl Extractable<Vec<MultipartField>> for ExtractableForm{
    async fn extract_from(&self, context: &Context, value: Vec<MultipartField>) {
        match self{
            ExtractableForm::WithFields(ef)=>{
                for (key,value_t) in ef {
                    let val:Vec<Value> = value.iter().filter(|mf| mf.name.clone().map(|n|n.eq(key)).unwrap_or(false)).map(|mpf|mpf.to_value()).collect();
                    if val.len()==0 {
                        context.define(value_t.to_string(),Value::Null).await;
                    } else if val.len() == 1 {
                        context.define(value_t.to_string(),val.get(0).unwrap().clone()).await;
                    } else {
                        context.define(value_t.to_string(),Value::Array(val)).await;
                    }
                }
            }
        }
    }
}


