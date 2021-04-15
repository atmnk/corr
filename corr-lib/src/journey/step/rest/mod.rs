pub mod parser;
use crate::template::object::extractable::{Extractable};
use crate::template::rest::{ RequestBody, RequestHeaders, RestVerb, FillableRequest};
use crate::template::rest::extractable::{ ExtractableResponse, CorrResponse};
use crate::journey::Executable;
use crate::core::runtime::Context;
use isahc::prelude::*;
use crate::template::Fillable;
use async_trait::async_trait;
#[derive(Debug, Clone,PartialEq)]
pub struct RestSetp{
    request: FillableRequest,
    response:Option<ExtractableResponse>
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
    async fn execute(&self, context: &Context) {
        rest(self.request.fill(context).await,self.response.clone(),context).await
    }
}
pub async fn rest(request: CorrRequest, response:Option<ExtractableResponse>,context:&Context) {
    let mut builder = match request.method {
        RestVerb::GET => Request::get(request.url.clone()),
        RestVerb::POST => Request::post(request.url.clone()),
        RestVerb::PATCH => Request::patch(request.url.clone()),
        RestVerb::PUT => Request::put(request.url.clone()),
        RestVerb::DELETE => Request::delete(request.url.clone())
    };
    if let Some(headers) = request.headers {
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
    if let Ok(i_req) = builder.body(request.body.clone().map(|bd| { bd.to_string_body() }).unwrap_or("".to_string())) {
        let i_response = i_req.send_async().await;
        if let Some(er) = response {
            if let Ok(mut rb) = i_response {
                if rb.status().as_u16() < 399 {
                    er.extract_from(context, CorrResponse {
                        body: rb.text_async().await.unwrap().to_string(),
                        original_response: rb
                    }).await
                } else {
                    eprintln!("Rest api {} Failed with code {}", request.url, rb.status())
                }
            } else {
                eprintln!("No Response for api {}", request.url)
            }
        } else {
            if let Ok(rb) = i_response {
                if rb.status().as_u16() > 399 {
                    {
                        eprintln!("Rest api {} with body {} Failed with code {}", request.url, request.body.unwrap().to_string_body(), rb.status())
                    }
                } else {
                    eprintln!("No Response for api {}", request.url)
                }
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
}