pub mod parser;
use crate::core::runtime::Context;
use crate::template::VariableReferenceName;
use crate::template::object::extractable::{ExtractableObject, Extractable};
use async_trait::async_trait;
use warp::hyper::http::HeaderValue;
use crate::core::Value;
use isahc::prelude::Response;
use isahc::Body;

#[derive(Debug, Clone,PartialEq)]
pub struct ExtractableResponseHeaders{
    pub headers:Vec<ExtractableResponseHeaderPair>
}

#[derive(Debug, Clone,PartialEq)]
pub struct ExtractableResponseHeaderPair{
    pub key:String,
    pub value:ExtractableResponseHeaderValue
}
#[derive(Debug, Clone,PartialEq)]

pub enum ExtractableResponseHeaderValue{
    WithVariableReference(VariableReferenceName)
}
pub struct CorrResponse{
    pub body:String,
    pub original_response:Response<Body>
}
pub struct Header{
    pub key:&'static str,
    pub value:&'static str,
}
impl Header{
    pub fn new(key:&'static str,value:&'static str)->Self{
        Header{
            key,
            value
        }
    }
}
impl CorrResponse {
    pub fn new(body:&str,headers:Vec<Header>,status:u16)->Self{
        let mut resp = Response::builder()
            .status(status);
        for header in headers {
            resp= resp.header(header.key,header.value)
        }

        CorrResponse{
            body:body.to_string(),
            original_response:resp.body(Body::from_bytes(body.as_bytes())).unwrap()
        }
    }
}
#[derive(Debug, Clone,PartialEq)]
pub enum ExtractableResponseBody{
    WithObject(ExtractableObject)
}
pub enum ResponseBody{
    JSON(serde_json::Value)
}
#[derive(Debug, Clone,PartialEq)]
pub struct ExtractableResponse {
    pub body:Option<ExtractableResponseBody>,
    pub headers:Option<ExtractableResponseHeaders>
}
#[async_trait]
impl Extractable<ResponseBody> for ExtractableResponseBody{
    async fn extract_from(&self, context: &Context, value: ResponseBody) {
        match self {
            ExtractableResponseBody::WithObject(eb)=>{
                match value {
                    ResponseBody::JSON(body)=>{
                        eb.extract_from(context,body).await
                    }
                }
            }
        }
    }
}
#[async_trait]
impl Extractable<CorrResponse> for ExtractableResponse{
    async fn extract_from(&self, context: &Context, value: CorrResponse) {
            if let Some(eb) = &self.body{
                match eb {
                    ExtractableResponseBody::WithObject(_)=>{
                        eb.extract_from(context,ResponseBody::JSON(serde_json::from_str::<serde_json::Value>(value.body.as_str()).unwrap())).await;
                    }
                }
            }
            if let Some(eh) = &self.headers{
                eh.extract_from(context,value).await
            }
    }
}
#[async_trait]
impl Extractable<CorrResponse> for ExtractableResponseHeaders{
    async fn extract_from(&self, context: &Context, value: CorrResponse) {
        for header in &self.headers {
            if let Some(hv) = value.original_response.headers().get(header.key.clone()){
                header.value.extract_from(context,hv.clone()).await
            }
        }
    }
}
#[async_trait]
impl Extractable<HeaderValue> for ExtractableResponseHeaderValue{
    async fn extract_from(&self, context: &Context, value: HeaderValue) {
        match self {
            ExtractableResponseHeaderValue::WithVariableReference(var)=>{
                if let Ok(hv)=value.to_str(){
                    context.define(var.to_string(),Value::String(hv.to_string())).await
                }
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::core::{Value};
    use crate::core::proto::{Output};
    use std::sync::{Arc, Mutex};
    use crate::core::runtime::{Context};
    use crate::parser::Parsable;
    use crate::template::rest::extractable::{ExtractableResponseHeaderValue, ExtractableResponseHeaders, CorrResponse, Header, ExtractableResponse, ExtractableResponseBody, ResponseBody};
    use crate::template::object::extractable::Extractable;
    use warp::hyper::http::HeaderValue;

    #[tokio::test]
    async fn should_extract_extractableresponseheadervalue(){
        let text=r#"token"#;
        let (_,ep) = ExtractableResponseHeaderValue::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        ep.extract_from(&context,HeaderValue::from_static("XYZABC")).await;
        assert_eq!(context.get_var_from_store(format!("token")).await,Option::Some(Value::String(format!("XYZABC"))))
    }
    #[tokio::test]
    async fn should_extract_extractableresponseheaderheaders(){
        let text=r#"{"Authorization":token,"X-API-KEY": api_key }"#;
        let (_,ep) = ExtractableResponseHeaders::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let cr= CorrResponse::new("",vec![Header::new("Authorization","ABCDXYZ"),
          Header::new("X-API-KEY","SomethingIsBetterThanNothing")
        ],200);
        ep.extract_from(&context,cr).await;
        assert_eq!(context.get_var_from_store(format!("token")).await,Option::Some(Value::String(format!("ABCDXYZ"))));
        assert_eq!(context.get_var_from_store(format!("api_key")).await,Option::Some(Value::String(format!("SomethingIsBetterThanNothing"))));
    }
    #[tokio::test]
    async fn should_extract_extractableresponse(){
        let text=r#"body object {"name":name } and headers {"Authorization":token,"X-API-KEY": api_key }"#;
        let (_,ep) = ExtractableResponse::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let cr= CorrResponse::new(r#"{"name":"Atmaram"}"#,vec![Header::new("Authorization","ABCDXYZ"),
                                          Header::new("X-API-KEY","SomethingIsBetterThanNothing")
        ],200);
        ep.extract_from(&context,cr).await;
        assert_eq!(context.get_var_from_store(format!("name")).await,Option::Some(Value::String(format!("Atmaram"))));
        assert_eq!(context.get_var_from_store(format!("token")).await,Option::Some(Value::String(format!("ABCDXYZ"))));
        assert_eq!(context.get_var_from_store(format!("api_key")).await,Option::Some(Value::String(format!("SomethingIsBetterThanNothing"))));
    }
    #[tokio::test]
    async fn should_extract_extractableresponsebody(){
        let text=r#"object {"place":place }"#;
        let (_,ep) = ExtractableResponseBody::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        ep.extract_from(&context,ResponseBody::JSON(serde_json::from_str::<serde_json::Value>(r#"{"place":"Pune"}"#).unwrap())).await;
        assert_eq!(context.get_var_from_store(format!("place")).await,Option::Some(Value::String(format!("Pune"))));
    }
}