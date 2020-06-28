extern crate reqwest;
extern crate serde_json;
use corr_core::runtime::{Environment, Value, Variable, VarType};
use corr_templates::json::{Json};
use corr_journeys::Executable;
use corr_templates::json::extractable::{ExtractableJson};
use reqwest::header::CONTENT_TYPE;
use corr_templates::text::Text;
use std::collections::HashMap;
use corr_templates::{Fillable, Extractable};
use ureq::{Request, Response};
use tokio::runtime::Runtime;
use std::collections::hash_map::RandomState;

pub struct RestData{
    pub url:Text,
    pub response:Option<ExtractableJson>,
    pub headers:HashMap<String,Text>,
    pub responseHeaders:HashMap<String,Variable>,
}
pub trait ExecutableRestStep{
    fn method(&self,runtime:&Environment)->Request;
    fn get_body(&self)->Option<&BodyData>;
    fn get_resp_template(&self)->&Option<ExtractableJson>;
    fn get_headers(&self)->&HashMap<String,Text>;
    fn get_response_headers(&self)->&HashMap<String,Variable>;
    fn perform(&self, runtime: &Environment) {
        let mut req=self.method(runtime);
        let resp;
        let headers=self.get_headers();
        for (key,value) in headers {
            req.set(key.as_str(),value.fill(runtime).to_string().as_str());
        }
        if let Some(body)=self.get_body() {
            resp= req.send_bytes(body.fill(runtime).as_bytes());
        } else {
            resp=req.call();
        }

        if resp.ok() {
            println!("success: {:?}", resp.headers_names());
            let headers=self.get_response_headers();
            for (key,value) in headers {
                if let Some(header_value) = resp.header(key){
                    let mut var=value.clone();
                    var.data_type=Option::Some(VarType::String);
                    var.extract(Value::String(header_value.to_string()),runtime);
                } else {
                    runtime.error(format!("Header {:?} not found in response",key))
                }

            }

            let resp_string = resp.into_string().unwrap_or("".to_string());

            if let Option::Some(resp_t)=&self.get_resp_template(){
                let val=serde_json::from_str(resp_string.as_str()).unwrap();
                let corr_val=Value::from(&val);
                resp_t.extract(corr_val,runtime);
            }


        } else {
            // This can include errors like failure to parse URL or connect timeout.
            // They are treated as synthetic HTTP-level error statuses.
            println!("error {}: {}", resp.status(), resp.into_string().unwrap());
        }
    }
}
impl Executable for PostStep{
    fn execute(&self, runtime: &Environment) {
        self.perform(runtime)
    }
}
impl Executable for PutStep{
    fn execute(&self, runtime: &Environment) {
        self.perform(runtime)
    }
}
impl Executable for PatchStep{
    fn execute(&self, runtime: &Environment) {
        self.perform(runtime)
    }
}
impl Executable for GetStep{
    fn execute(&self, runtime: &Environment) {
        self.perform(runtime)
    }
}
impl Executable for DeleteStep{
    fn execute(&self, runtime: &Environment) {
        self.perform(runtime)
    }
}
impl ExecutableRestStep for PostStep{
    fn method(&self, runtime: &Environment) -> Request {
        ureq::post(self.rest.url.fill(runtime).to_string().as_str())
    }


    fn get_body(&self) -> Option<&BodyData> {
        Option::Some(&self.body)
    }

    fn get_resp_template(&self) -> &Option<ExtractableJson> {
        return &self.rest.response
    }

    fn get_headers(&self) -> &HashMap<String, Text, RandomState> {
        return &self.rest.headers
    }

    fn get_response_headers(&self) -> &HashMap<String, Variable, RandomState> {
        return &self.rest.responseHeaders;
    }
}
impl ExecutableRestStep for PutStep{
    fn method(&self, runtime: &Environment) -> Request {
        ureq::put(self.rest.url.fill(runtime).to_string().as_str())
    }
    fn get_resp_template(&self) -> &Option<ExtractableJson> {
        return &self.rest.response
    }
    fn get_body(&self) -> Option<&BodyData> {
        Option::Some(&self.body)
    }
    fn get_headers(&self) -> &HashMap<String, Text, RandomState> {
        return &self.rest.headers
    }
    fn get_response_headers(&self) -> &HashMap<String, Variable, RandomState> {
        return &self.rest.responseHeaders;
    }
}
impl ExecutableRestStep for PatchStep{
    fn method(&self, runtime: &Environment) -> Request {
        ureq::patch(self.rest.url.fill(runtime).to_string().as_str())
    }
    fn get_resp_template(&self) -> &Option<ExtractableJson> {
        return &self.rest.response
    }
    fn get_body(&self) -> Option<&BodyData> {
        Option::Some(&self.body)
    }
    fn get_headers(&self) -> &HashMap<String, Text, RandomState> {
        return &self.rest.headers
    }
    fn get_response_headers(&self) -> &HashMap<String, Variable, RandomState> {
        return &self.rest.responseHeaders;
    }
}
impl ExecutableRestStep for GetStep{
    fn method(&self, runtime: &Environment) -> Request {
        ureq::get(self.rest.url.fill(runtime).to_string().as_str())
    }
    fn get_resp_template(&self) -> &Option<ExtractableJson> {
        return &self.rest.response
    }
    fn get_body(&self) -> Option<&BodyData> {
        Option::None
    }
    fn get_headers(&self) -> &HashMap<String, Text, RandomState> {
        return &self.rest.headers
    }
    fn get_response_headers(&self) -> &HashMap<String, Variable, RandomState> {
        return &self.rest.responseHeaders;
    }
}
impl ExecutableRestStep for DeleteStep{
    fn method(&self, runtime: &Environment) -> Request {
        ureq::delete(self.rest.url.fill(runtime).to_string().as_str())
    }
    fn get_resp_template(&self) -> &Option<ExtractableJson> {
        return &self.rest.response
    }
    fn get_body(&self) -> Option<&BodyData> {
        Option::None
    }
    fn get_headers(&self) -> &HashMap<String, Text, RandomState> {
        return &self.rest.headers
    }
    fn get_response_headers(&self) -> &HashMap<String, Variable, RandomState> {
        return &self.rest.responseHeaders;
    }
}
pub struct PostStep{
    pub rest:RestData,
    pub body:BodyData,
}
#[derive(Clone)]
pub enum BodyData{
    Json(Json),
    Text(Text)
}
impl BodyData {
    fn fill(&self,runtime: &Environment)->String{
        match self {
            BodyData::Json(json)=>{
                serde_json::to_string(&json.fill(runtime)).unwrap()
            },
            BodyData::Text(text)=>{
                text.fill(runtime).to_string()
            }
        }
    }
}

    // impl Executable for PostStep{
    //     fn execute(&self, runtime: &Environment) {
    //         let req=ureq::post(filled.path.clone());
    //         let body = match &self.body {
    //             BodyData::Json(json)=>{
    //                 serde_json::to_string(&json.fill(runtime)).unwrap()
    //             },
    //             BodyData::Text(text)=>{
    //                 text.fill(runtime).to_string()
    //             }
    //         };
    //         req.send_bytes(body.as_bytes())
    //     }
    // }
// impl Executable for PostStep {
//     fn execute(&self, runtime: &Environment) {
//         let client = reqwest::blocking::Client::new();
//         let request_body=self.body.fill(runtime);
//         let mut initial= client
//             .post(&self.rest.url.fill(runtime).to_string());
//
//         for key in self.rest.headers.keys() {
//             initial = initial.header(key,self.rest.headers.get(key).unwrap().fill(runtime).to_string())
//         }
//
//         let body=initial.body(request_body).send().unwrap().text().unwrap();
//
//
//         if let Option::Some(resp)=&self.rest.response{
//             let val=serde_json::from_str(body.as_str()).unwrap();
//             let corr_val=Value::from(&val);
//             resp.extract(corr_val,runtime);
//         }
//     }
// }
pub struct PutStep{
    pub rest:RestData,
    pub body:BodyData,
}
// impl Executable for PutStep {
//     fn execute(&self, runtime: &Environment) {
//         let client = reqwest::blocking::Client::new();
//         let request_body=self.body.fill(runtime);
//         let mut initial= client
//             .put(&self.rest.url.fill(runtime).to_string());
//
//         for key in self.rest.headers.keys() {
//             initial = initial.header(key,self.rest.headers.get(key).unwrap().fill(runtime).to_string())
//         }
//
//         let body=initial.body(request_body).send().unwrap().text().unwrap();
//
//
//         if let Option::Some(resp)=&self.rest.response{
//             let val=serde_json::from_str(body.as_str()).unwrap();
//             let corr_val=Value::from(&val);
//             resp.extract(corr_val,runtime);
//         }
//     }
// }
pub struct PatchStep{
    pub rest:RestData,
    pub body:BodyData,
}
// impl Executable for PatchStep {
//     fn execute(&self, runtime: &Environment) {
//         let client = reqwest::blocking::Client::new();
//         let request_body=self.body.fill(runtime);
//         let mut initial= client
//             .patch(&self.rest.url.fill(runtime).to_string());
//
//         for key in self.rest.headers.keys() {
//             initial = initial.header(key,self.rest.headers.get(key).unwrap().fill(runtime).to_string())
//         }
//
//         let body=initial.body(request_body).send().unwrap().text().unwrap();
//
//
//         if let Option::Some(resp)=&self.rest.response{
//             let val=serde_json::from_str(body.as_str()).unwrap();
//             let corr_val=Value::from(&val);
//             resp.extract(corr_val,runtime);
//         }
//     }
// }
pub struct GetStep{
    pub rest:RestData
}
// fn toValue(resp:&reqwest::blocking::Response)->Value{
//     for header in resp.headers().keys() {
//         println!("{:?}",header)
//     }
//     return Value::Null;
// }

// impl Executable for GetStep{
//     fn execute(&self, runtime: &Environment) {
//         let client = reqwest::blocking::Client::new();
//         let mut initial= client
//             .get(&self.rest.url.fill(runtime).to_string());
//
//         for key in self.rest.headers.keys() {
//             initial = initial.header(key,self.rest.headers.get(key).unwrap().fill(runtime).to_string())
//         }
//
//         let resp = initial.send().unwrap();
//
//
//
//         let rh = toValue(&resp);
//         let body= resp.text().unwrap();
//
//
//         if let Option::Some(resp)=&self.rest.response{
//             let val=serde_json::from_str(&body).unwrap();
//             let corr_val=Value::from(&val);
//             resp.extract(corr_val,runtime);
//         }
//         if let Option::Some(resp)=&self.rest.responseHeaders{
//             let val=serde_json::from_str(&body).unwrap();
//             let corr_val=Value::from(&val);
//             resp.extract(corr_val,runtime);
//         }
//
//     }
// }
pub struct DeleteStep{
    pub rest:RestData
}
// impl Executable for DeleteStep{
//     fn execute(&self, runtime: &Environment) {
//         let client = reqwest::blocking::Client::new();
//         let mut initial= client
//             .delete(&self.rest.url.fill(runtime).to_string());
//
//         for key in self.rest.headers.keys() {
//             initial = initial.header(key,self.rest.headers.get(key).unwrap().fill(runtime).to_string())
//         }
//
//         let body= initial.send().unwrap().text().unwrap();
//
//         if let Option::Some(resp)=&self.rest.response{
//             let val=serde_json::from_str(&body).unwrap();
//             let corr_val=Value::from(&val);
//             resp.extract(corr_val,runtime);
//         }
//
//     }
// }
#[cfg(test)]
mod tests{
    use crate::{PostStep, RestData, GetStep, BodyData, ExecutableRestStep};
    use corr_templates;
    use corr_templates::json::parser::parse;
    use corr_core::runtime::{Environment, ValueProvider, Variable, Value, VarType};
    use corr_journeys::Executable;
    use std::collections::HashMap;
    use corr_templates::text::Text;
    use reqwest::header::CONTENT_TYPE;
    use std::net::Shutdown::Read;

    struct MockChannel;
    impl ValueProvider for MockChannel{

        fn save(&self, _var: Variable, _value: Value) {
            unimplemented!()
        }

        fn read(&mut self, _variable: Variable) -> Value {
            unimplemented!()
        }

        fn write(&mut self, _text: String) {
            unimplemented!()
        }

        fn set_index_ref(&mut self, _index_ref_var: Variable, _list_ref_var: Variable) {
            unimplemented!()
        }

        fn drop(&mut self, _str: String) {
            unimplemented!()
        }

        fn load_ith_as(&mut self, _i: usize, _index_ref_var: Variable, _list_ref_var: Variable) {
            unimplemented!()
        }

        fn load_value_as(&mut self, _ref_var: Variable, _val: Value) {
            unimplemented!()
        }

        fn close(&mut self) {
            unimplemented!()
        }
    }
    #[test]
    fn should_do_post() {
        let body=BodyData::Json(parse(r#"{"name":"PQR"}"#).unwrap());
        let response=corr_templates::json::extractable::parser::parse(r#"{"id":{{id}}}"#);
        let mut headers:HashMap<String,Text>=HashMap::new();
        headers.insert(format!("{}",CONTENT_TYPE),corr_templates::text::parser::parse("application/json").unwrap());
        let step=PostStep{
            rest:RestData{
                url:corr_templates::text::parser::parse("http://localhost:8080/api/category").unwrap(),
                response,
                headers,
                responseHeaders:HashMap::new()
            },
            body
        ,

        };
        let runtime=Environment::new_rc(MockChannel);
        step.execute(&runtime);
        println!("{:?}",(*runtime.channel).borrow().reference_store)
    }
    #[test]
    fn should_do_get() {
        let response=corr_templates::json::extractable::parser::parse(r#"[ <% for (id:Long in ids){%>
                                        {
                                            "id": {{id}}
                                        }
                                    <%}%>]"#);
        let mut headers:HashMap<String,Text>=HashMap::new();
        let step=GetStep{
            rest:RestData{
                url:corr_templates::text::parser::parse("http://localhost:8080/api/category").unwrap(),
                response,
                headers,
                responseHeaders:HashMap::new()
            }
        };
        let runtime=Environment::new_rc(MockChannel);
        step.execute(&runtime);
        println!("{:?}",(*runtime.channel).borrow().reference_store)
    }
    #[test]
    fn should_do_get_capturing_headers() {
        let response=Option::None;
        let mut headers:HashMap<String,Variable>=HashMap::new();
        headers.insert(format!("token"),Variable{
            name:format!("token"),
            data_type:Option::Some(VarType::String)
        });
        let step=GetStep{
            rest:RestData{
                url:corr_templates::text::parser::parse("http://localhost:9000/api/test").unwrap(),
                response,
                headers:HashMap::new(),
                responseHeaders:headers
            }
        };
        let runtime=Environment::new_rc(MockChannel);
        step.execute(&runtime);
        println!("{:?}",(*runtime.channel).borrow().reference_store)
    }
}