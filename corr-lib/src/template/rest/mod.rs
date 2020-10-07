pub mod extractable;
pub mod parser;
use crate::template::{Expression, Fillable};
use crate::template::text::Text;
use crate::core::runtime::Context;
use async_trait::async_trait;
use crate::template::object::FillableObject;
use crate::core::Value;
use crate::journey::step::rest::CorrRequest;

#[derive(Debug, Clone,PartialEq)]
pub enum RestVerb{
    GET,
    POST,
    PUT,
    PATCH,
    DELETE
}
#[derive(Debug, Clone,PartialEq)]
pub struct FillableRequest {
    pub verb:RestVerb,
    pub url: URL,
    pub body:Option<FillableRequestBody>,
    pub headers:Option<FillableRequestHeaders>,
}
#[derive(Debug, Clone,PartialEq)]
pub struct FillableRequestHeaders{
    pub headers:Vec<FillableRequestHeaderPair>
}
#[derive(Debug, Clone,PartialEq)]
pub struct RequestHeaders{
    pub headers:Vec<RequestHeader>
}
#[derive(Debug, Clone,PartialEq)]
pub struct FillableRequestHeaderPair{
    pub key:String,
    pub value:FillableRequestHeaderValue
}
#[derive(Debug, Clone,PartialEq)]

pub enum FillableRequestHeaderValue{
    WithExpression(Expression),
    WithText(Text)
}
#[derive(Debug, Clone,PartialEq)]
pub enum FillableRequestBody{
    WithObject(FillableObject)
}
#[derive(Debug, Clone,PartialEq)]
pub enum RequestBody{
    JSON(Value)
}
#[derive(Debug, Clone,PartialEq)]
pub enum URL{
    WithExpression(Expression),
    WithText(Text)
}
impl RequestBody{
    pub fn to_string_body(&self)->String{
        match self {
            RequestBody::JSON(val)=>{
                if let Ok(body) = serde_json::to_string(&val.to_json_value()){
                    body
                } else {
                    "".to_string()
                }
            }
        }
    }
}
#[async_trait]
impl Fillable<CorrRequest> for FillableRequest{
    async fn fill(&self, context: &Context) -> CorrRequest {
        let body = if let Some(bd)=&self.body{
            Option::Some(bd.fill(context).await)
        } else {
            Option::None
        };
        let headers = if let Some(frh)=&self.headers{
            Option::Some(frh.fill(context).await)
        } else {
            Option::None
        };
        CorrRequest{
            method:self.verb.clone(),
            body,
            url:self.url.fill(context).await,
            headers
        }
    }
}
#[async_trait]
impl Fillable<String> for URL{
    async fn fill(&self, context: &Context) -> String {
        match self {
            URL::WithText(txt)=>txt.fill(context).await,
            URL::WithExpression(expr)=>expr.fill(context).await
        }
    }
}
#[async_trait]
impl Fillable<RequestBody> for FillableRequestBody{
    async fn fill(&self, context: &Context) -> RequestBody {
        match self{
            FillableRequestBody::WithObject(obj)=>{
                RequestBody::JSON(obj.fill(context).await)
            }
        }
    }
}

#[async_trait]
impl Fillable<String> for FillableRequestHeaderValue{
    async fn fill(&self, context: &Context) -> String {
        match self {
            FillableRequestHeaderValue::WithExpression(expr)=>expr.fill(context).await,
            FillableRequestHeaderValue::WithText(txt)=>txt.fill(context).await
        }
    }
}
#[derive(Debug, Clone,PartialEq)]
pub struct RequestHeader{
    pub key:String,
    pub value:String
}
#[async_trait]
impl Fillable<RequestHeader> for FillableRequestHeaderPair{
    async fn fill(&self, context: &Context) -> RequestHeader {
        RequestHeader{
            key:self.key.clone(),
            value:self.value.fill(context).await
        }
    }
}
#[async_trait]
impl Fillable<RequestHeaders> for FillableRequestHeaders{
    async fn fill(&self, context: &Context) -> RequestHeaders {
        let mut vec_val=vec![];
        for header in &self.headers {
            vec_val.push(header.fill(context).await);
        }
        RequestHeaders{
            headers:vec_val
        }
    }
}
#[cfg(test)]
mod tests{
    use crate::core::proto::{Input, ContinueInput, Output};
    use std::sync::{Arc, Mutex};
    use crate::core::runtime::Context;
    use crate::parser::Parsable;
    use crate::template::Fillable;
    use crate::template::rest::{FillableRequest, RestVerb, RequestBody, RequestHeaders, RequestHeader, URL, FillableRequestBody, FillableRequestHeaderValue};
    use crate::journey::step::rest::CorrRequest;
    use crate::core::{DataType, Value};
    use nom::lib::std::collections::HashMap;

    #[tokio::test]
    async fn should_fill_fillablerequest_when_only_url(){
        let txt = r#"get request {
            url: text `http://localhost/<%id:PositiveInteger%>`
        }"#;
        let (_,fr) = FillableRequest::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"id".to_string(),value:"3".to_string(),data_type:DataType::PositiveInteger})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = fr.fill(&context).await;
        assert_eq!(filled,CorrRequest{
            method:RestVerb::GET,
            url:format!("http://localhost/3"),
            body:Option::None,
            headers:Option::None
        })
    }
    #[tokio::test]
    async fn should_fill_fillablerequest_when_with_body_and_no_headers(){
        let txt = r#"post request {
            url: text `http://localhost/<%id:PositiveInteger%>`,
            body: object {"name":name }
        }"#;
        let (_,fr) = FillableRequest::parser(txt).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String}),
            Input::Continue(ContinueInput{name:"id".to_string(),value:"3".to_string(),data_type:DataType::PositiveInteger})
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = fr.fill(&context).await;
        let mut mp = HashMap::new();
        mp.insert(format!("name"),Value::String(format!("Atmaram")));
        assert_eq!(filled,CorrRequest{
            method:RestVerb::POST,
            url:format!("http://localhost/3"),
            body:Option::Some(RequestBody::JSON(Value::Map(mp))),
            headers:Option::None
        })
    }
    #[tokio::test]
    async fn should_fill_fillablerequest_when_with_headers_and_no_body(){
        let txt = r#"post request {
            url: text `http://localhost/<%id:PositiveInteger%>`,
            headers: {
                "X-API-KEY": x_api_key
            }
        }"#;
        let (_,fr) = FillableRequest::parser(txt).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"x_api_key".to_string(),value:"Something".to_string(),data_type:DataType::String}),
            Input::Continue(ContinueInput{name:"id".to_string(),value:"3".to_string(),data_type:DataType::PositiveInteger})
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = fr.fill(&context).await;
        assert_eq!(filled,CorrRequest{
            method:RestVerb::POST,
            url:format!("http://localhost/3"),
            body:Option::None,
            headers:Option::Some(RequestHeaders{
                headers:vec![RequestHeader{
                    key:format!("X-API-KEY"),
                    value:format!("Something")
                }]
            })
        })
    }
    #[tokio::test]
    async fn should_fill_fillablerequest_when_with_body_and_headers(){
        let txt = r#"post request {
            url: text `http://localhost/<%id:PositiveInteger%>`,
            body: object {"name":name },
            headers: {
                "X-API-KEY": x_api_key
            }
        }"#;
        let (_,fr) = FillableRequest::parser(txt).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String}),
            Input::Continue(ContinueInput{name:"x_api_key".to_string(),value:"Something".to_string(),data_type:DataType::String}),
            Input::Continue(ContinueInput{name:"id".to_string(),value:"3".to_string(),data_type:DataType::PositiveInteger})
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = fr.fill(&context).await;
        let mut mp = HashMap::new();
        mp.insert(format!("name"),Value::String(format!("Atmaram")));
        assert_eq!(filled,CorrRequest{
            method:RestVerb::POST,
            url:format!("http://localhost/3"),
            body:Option::Some(RequestBody::JSON(Value::Map(mp))),
            headers:Option::Some(RequestHeaders{
                headers:vec![RequestHeader{
                    key:format!("X-API-KEY"),
                    value:format!("Something")
                }]
            })
        })
    }
    #[tokio::test]
    async fn should_fill_url_when_text(){
        let txt = r#"text `http://localhost/<%id:PositiveInteger%>`"#;
        let (_,url) = URL::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"id".to_string(),value:"3".to_string(),data_type:DataType::PositiveInteger})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = url.fill(&context).await;
        assert_eq!(filled,format!("http://localhost/3"))
    }
    #[tokio::test]
    async fn should_fill_url_when_expression(){
        let txt = r#"concat("http://localhost/",id:PositiveInteger)"#;
        let (_,url) = URL::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"id".to_string(),value:"3".to_string(),data_type:DataType::PositiveInteger})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = url.fill(&context).await;
        assert_eq!(filled,format!("http://localhost/3"))
    }
    #[tokio::test]
    async fn should_fill_fillablerequestbody(){
        let txt = r#"object {"name":name }"#;
        let (_,frb) = FillableRequestBody::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = frb.fill(&context).await;
        let mut mp = HashMap::new();
        mp.insert(format!("name"),Value::String(format!("Atmaram")));
        assert_eq!(filled,RequestBody::JSON(Value::Map(mp)))
    }
    #[tokio::test]
    async fn should_fill_fillablerequestheadervalue(){
        let txt = r#"concat("ABC-",name)"#;
        let (_,frhv) = FillableRequestHeaderValue::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = frhv.fill(&context).await;
        assert_eq!(filled,format!("ABC-Atmaram"))
    }
}