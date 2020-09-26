// pub mod parser;
// use async_trait::async_trait;
// use crate::journey::{Executable};
// use crate::template::text::{Text};
// use crate::core::runtime::{Context};
// use crate::template::json::{Json, FillableJson};
// use isahc::prelude::*;
// use crate::template::json::extractable::{EJson, Extractable};
// use crate::template::Fillable;
//
// #[derive(Debug, Clone,PartialEq)]
// pub enum RestVerb{
//     GET,
//     POST,
//     PUT,
//     PATCH,
//     DELETE
// }
// #[derive(Debug, Clone,PartialEq)]
// pub struct RestStep {
//     pub verb:RestVerb,
//     pub url:Text,
//     pub body:Option<Body>,
//     pub response:Option<EJson>
// }
// impl RestStep {
//     pub async fn text_body(&self, context:&Context) ->String{
//         match &self.body {
//             Option::Some(Body::Json(val))=>{
//                 if let Ok(res)=serde_json::to_string(&val.json_fill(context).await){
//                     res
//                 } else {
//                     "".to_string()
//                 }
//             },
//             Option::Some(Body::Text(txt))=>{
//                 txt.fill(context).await
//             },
//             _=>"".to_string()
//         }
//     }
//     async fn to_request(&self,context:&Context)->Option<Request<String>>{
//         let mut builder = match self.verb {
//             RestVerb::GET=> {Request::get(self.url.fill(context).await)}
//             RestVerb::POST=>{Request::post(self.url.fill(context).await)}
//             RestVerb::PATCH=>{Request::patch(self.url.fill(context).await)}
//             RestVerb::PUT=>{Request::put(self.url.fill(context).await)}
//             RestVerb::DELETE=>{Request::delete(self.url.fill(context).await)}
//         };
//         match self.body {
//             Option::Some(Body::Text(_))=>{
//                 builder = builder.header("Content-Type","application/text")
//             },
//             Option::Some(Body::Json(_))=>{
//                 builder = builder.header("Content-Type","application/json")
//             },
//             _=>{}
//         }
//         if let Ok(request) = builder
//             .body(self.text_body(context).await){
//             Some(request)
//         } else {
//             None
//         }
//
//     }
// }
// #[derive(Debug, Clone,PartialEq)]
// pub enum Body{
//     Json(Json),
//     Text(Text)
// }
//
// #[async_trait]
// impl Executable for RestStep {
//     async fn execute(&self, context: &Context) {
//         if let Some(request)=self.to_request(context).await{
//             if let Ok(mut response)=Request::send_async(request).await{
//                 if let Ok(body)=response.text(){
//                     let value: serde_json::Value = serde_json::from_str(body.as_str()).unwrap_or(serde_json::Value::Null);
//                     println!("{}",value);
//                     if let Some(response_template) = &self.response{
//                         response_template.extract_from(context,value).await;
//                     }
//                 }
//             }
//         }
//     }
// }
