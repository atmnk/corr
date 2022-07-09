pub mod parser;


use std::collections::HashMap;
use std::sync::Arc;
use crate::template::object::extractable::{Extractable};
use crate::template::rest::{ RequestBody, RequestHeaders, RestVerb, FillableRequest};
use crate::template::rest::extractable::{ExtractableRestData, CorrResponse};
use crate::journey::{Executable, Journey};
use crate::core::runtime::Context;
use crate::template::Fillable;
use async_trait::async_trait;
use tokio::task::JoinHandle;
use hyper::{Request,Body};
use hyper::body::Bytes;
use hyper_tls::HttpsConnector;
use hyper::Client;
use hyper::client::HttpConnector;
use lazy_static::lazy_static;
use std::time::{Instant};

lazy_static! {
    static ref HTTPCLIENT: Client<HttpConnector> = Client::builder().build::<_, hyper::Body>(HttpConnector::new());
}


#[derive(Debug, Clone,PartialEq)]
pub struct RestSetp{
    is_async:bool,
    request: FillableRequest,
    response:Option<ExtractableRestData>
}
#[derive(Debug, Clone,PartialEq)]
pub struct CorrRequest {
    pub method:RestVerb,
    pub url:String,
    pub body:Option<RequestBody>,
    pub headers:Option<RequestHeaders>
}
#[async_trait]
impl Executable for RestSetp{

    async fn execute(&self, context: &Context) ->Vec<JoinHandle<bool>>{
        let start = Instant::now();
        let req = self.request.fill(context).await;
        rest(req.clone(),self.response.clone(),context,self.is_async).await;
        let duration = start.elapsed();
        context.scrapper.ingest("response_time",duration.as_millis() as f64,vec![("method".to_string(),req.method.clone().as_str().to_string()),("url".to_string(),req.url.clone())]).await;
        context.rest_stats_store.push_stat((req.method,req.url,duration.as_millis())).await;
        return vec![]
    }

    fn get_deps(&self) -> Vec<String> {
        vec![]
    }
}
pub async fn rest(request: CorrRequest, response:Option<ExtractableRestData>, context:&Context, is_async:bool) {
    let mut builder = match request.method {
        RestVerb::GET => Request::get(request.url.clone()),
        RestVerb::POST => Request::post(request.url.clone()),
        RestVerb::PATCH => Request::patch(request.url.clone()),
        RestVerb::PUT => Request::put(request.url.clone()),
        RestVerb::DELETE => Request::delete(request.url.clone())
    };
    if let Some(headers) = request.headers.clone() {
        for header in headers.headers {
            builder = builder.header(header.key.as_str(), header.value.as_str())
        }
    }
    match &request.body {
        Option::Some(RequestBody::JSON(_)) => {
            builder = builder.header("Content-Type", "application/json")
        },
        _ => {}
    };
    if let Ok(i_req) = builder.body(Body::from(request.body.clone().map(|bd| { bd.to_string_body() }).unwrap_or("".to_string()))) {
        let context = context.clone();
        let step = async move|| {
            let i_response ={
                let uri = i_req.uri().to_string();
                if uri.starts_with("https") {
                    let https = HttpsConnector::new();
                    let client = Client::builder().build::<_, hyper::Body>(https);
                    client.request(i_req).await
                } else {
                    // let http = HttpConnector::new();
                    // let client = Client::builder().build::<_, hyper::Body>(http);
                    HTTPCLIENT.request(i_req).await
                }
            };

            if let Some(er) = response {
                match i_response {
                    Ok(rb)=>{
                        if rb.status().as_u16() < 399 {
                            let (parts,body) = rb.into_parts();
                            let body_bytes = hyper::body::to_bytes(body).await.unwrap_or(Bytes::from(""));

                            er.extract_from(&context, CorrResponse {
                                body: String::from_utf8(body_bytes.to_vec()).unwrap_or("".to_string()), //text_async().await.unwrap().to_string(),
                                headers: parts.headers,
                                status:parts.status.as_u16()
                            }).await
                        } else {
                            context.scrapper.ingest("errors",1.0,vec![("api".to_string(),request.url.clone()),("message".to_string(),format!("{}",rb.status().as_str()))]).await;
                            eprintln!("Rest api {} Failed with code {}", request.url, rb.status())
                        }
                    },
                    Err(e)=>{
                        context.scrapper.ingest("errors",1.0,vec![("api".to_string(),request.url.clone()),("message".to_string(),format!("{}",e.to_string()))]).await;
                        eprintln!("Error Response for api {} {:?}", request.url,e)
                    }
                }

            } else {
                match i_response {
                    Ok(rb)=>{
                        if rb.status().as_u16() > 399 {
                            context.scrapper.ingest("errors",1.0,vec![(format!("status"),format!("{}",rb.status())),(format!("api"),format!("{}",request.url))]).await;
                            eprintln!("Rest api {} with body {} Failed with code {}", request.url, request.body.map(|b|b.to_string_body()).unwrap_or(format!("")), rb.status())
                        }
                    },
                    Err(e)=>{
                        context.scrapper.ingest("errors",1.0,vec![(format!("api"),format!("{}",request.url))]).await;
                        eprintln!("Error Response for api {} {:?}", request.url,e)
                    }
                }
            }
        };
        if is_async {
            tokio::spawn(step());
        } else {
            step().await;
        }

    }

}
#[cfg(test)]
mod tests {
    use crate::core::proto::{Input};
    use std::sync::{Arc, Mutex};
    use crate::journey::{Executable};
    use crate::core::runtime::{Context};
    use crate::parser::Parsable;
    use crate::journey::step::rest::RestSetp;
    use crate::core::{DataType, Value};
    use mockito::mock;

    #[tokio::test]
    async fn should_execute_get_rest_step() {
        let mock = mock("GET", "/hello")
            .with_status(200)
            .with_body(r#"{"id" : 1 }"#)
            .with_header("A", "Hello")
            .match_header("Hello", "hello")
            .create();

        let text = r#"get request {
            url: text `<%base_url%>/hello`,
            headers: { "Hello": "hello" }
        } matching body object { "id": id } and headers { "A": a }"#;
        let (_, step) = RestSetp::parser(text).unwrap();
        let input = vec![
            Input::new_continue("base_url".to_string(), mockito::server_url(), DataType::String)
        ];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context = Context::mock(input, buffer.clone());
        step.execute(&context).await;
        mock.assert();
        assert_eq!(context.get_var_from_store(format!("id")).await, Option::Some(Value::PositiveInteger(1)));
        assert_eq!(context.get_var_from_store(format!("a")).await, Option::Some(Value::String("Hello".to_string())))
    }

    #[tokio::test]
    async fn should_execute_get_rest_step_onhttps() {

        let text = r#"get request {
            url: text `<%base_url%>/todos/1`,
            headers: { "Hello": "hello" }
        } matching body object { "title": title }"#;
        let (_, step) = RestSetp::parser(text).unwrap();
        let input = vec![
            Input::new_continue("base_url".to_string(), "https://jsonplaceholder.typicode.com".to_string(), DataType::String)
        ];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context = Context::mock(input, buffer.clone());
        step.execute(&context).await;
        assert_eq!(context.get_var_from_store(format!("title")).await, Option::Some(Value::String("delectus aut autem".to_string())));
    }

    #[tokio::test]
    async fn should_execute_post_rest_step() {
        let mock = mock("POST", "/hello")
            .with_status(200)
            .with_body(r#"{"id" : 1 }"#)
            .with_header("A", "Hello")
            .match_header("Hello", "3")
            .create();

        let text = r#"post request {
            url: text `<%base_url%>/hello`,
            body: object { "name" : name },
            headers: { "Hello": add(1,2) }
        } matching body object { "id": id } and headers { "A": a }"#;
        let (_, step) = RestSetp::parser(text).unwrap();
        let input = vec![
            Input::new_continue("name".to_string(), "Atmaram".to_string(), DataType::String),
            Input::new_continue("base_url".to_string(), mockito::server_url(), DataType::String)
        ];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context = Context::mock(input, buffer.clone());
        step.execute(&context).await;
        mock.assert();
        assert_eq!(context.get_var_from_store(format!("id")).await, Option::Some(Value::PositiveInteger(1)));
        assert_eq!(context.get_var_from_store(format!("a")).await, Option::Some(Value::String("Hello".to_string())))
    }

    #[tokio::test]
    async fn should_execute_put_rest_step() {
        let mock = mock("PUT", "/hello")
            .with_status(200)
            .with_body(r#"{"id" : 1 }"#)
            .with_header("A", "Hello")
            .match_header("Hello", "AB")
            .create();

        let text = r#"put request {
            url: text `<%base_url%>/hello`,
            body: object { "name" : name },
            headers: { "Hello": concat("A","B") }
        } matching body object { "id": id } and headers { "A": a }"#;
        let (_, step) = RestSetp::parser(text).unwrap();
        let input = vec![
            Input::new_continue("name".to_string(), "Atmaram".to_string(), DataType::String),
            Input::new_continue("base_url".to_string(), mockito::server_url(), DataType::String)
        ];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context = Context::mock(input, buffer.clone());
        step.execute(&context).await;
        mock.assert();
        assert_eq!(context.get_var_from_store(format!("id")).await, Option::Some(Value::PositiveInteger(1)));
        assert_eq!(context.get_var_from_store(format!("a")).await, Option::Some(Value::String("Hello".to_string())))
    }

    #[tokio::test]
    async fn should_execute_patch_rest_step() {
        let mock = mock("PATCH", "/hello")
            .with_status(200)
            .with_body(r#"{"id" : 1 }"#)
            .with_header("A", "Hello")
            .create();

        let text = r#"patch request {
            url: text `<%base_url%>/hello`,
            body: object { "name" : name }
        } matching body object { "id": id } and headers { "A": a }"#;
        let (_, step) = RestSetp::parser(text).unwrap();
        let input = vec![
            Input::new_continue("name".to_string(), "Atmaram".to_string(), DataType::String),
            Input::new_continue("base_url".to_string(), mockito::server_url(), DataType::String)
        ];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context = Context::mock(input, buffer.clone());
        step.execute(&context).await;
        mock.assert();
        assert_eq!(context.get_var_from_store(format!("id")).await, Option::Some(Value::PositiveInteger(1)));
        assert_eq!(context.get_var_from_store(format!("a")).await, Option::Some(Value::String("Hello".to_string())))
    }

    #[tokio::test]
    async fn should_execute_delete_rest_step() {
        let mock = mock("DELETE", "/1")
            .with_status(200)
            .with_body(r#"{"id" : 1 }"#)
            .with_header("A", "Hello")
            .match_header("Hello", "hello")
            .create();

        let text = r#"delete request {
            url: text `<%base_url%>/1`,
            headers: { "Hello": "hello" }
        } matching body object { "id": id } and headers { "A": a }"#;
        let (_, step) = RestSetp::parser(text).unwrap();
        let input = vec![
            Input::new_continue("base_url".to_string(), mockito::server_url(), DataType::String)
        ];
        let buffer = Arc::new(Mutex::new(vec![]));
        let context = Context::mock(input, buffer.clone());
        step.execute(&context).await;
        mock.assert();
        assert_eq!(context.get_var_from_store(format!("id")).await, Option::Some(Value::PositiveInteger(1)));
        assert_eq!(context.get_var_from_store(format!("a")).await, Option::Some(Value::String("Hello".to_string())))
    }
}