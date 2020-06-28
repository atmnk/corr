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
            let url=resp.get_url().to_string();
            let status=resp.status();
            let body=resp.into_string().unwrap();
            runtime.error(format!("Error reported by api {} with status {} and response {}",url,status,body));
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

pub struct PutStep{
    pub rest:RestData,
    pub body:BodyData,
}

pub struct PatchStep{
    pub rest:RestData,
    pub body:BodyData,
}

pub struct GetStep{
    pub rest:RestData
}

pub struct DeleteStep{
    pub rest:RestData
}

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

        fn write(&mut self, text: String) {

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

        }
    }
    #[test]
    fn should_do_post() {
        let body=BodyData::Json(parse(r#"{"username":"admin","password":"test123$"}"#).unwrap());
        let response=corr_templates::json::extractable::parser::parse(r#"{"token":{{token}},}"#);
        let mut headers:HashMap<String,Text>=HashMap::new();
        headers.insert(format!("{}",CONTENT_TYPE),corr_templates::text::parser::parse("application/json").unwrap());
        let step=PostStep{
            rest:RestData{
                url:corr_templates::text::parser::parse("https://atmnk-swapi.herokuapp.com/api/login").unwrap(),
                response,
                headers,
                responseHeaders:HashMap::new()
            },
            body,

        };
        let mut runtime=Environment::new_rc(MockChannel);
        step.execute(&runtime);
        assert_ne!(runtime.read(Variable{
            name:format!("token"),
            data_type:Option::None
        }),Value::Null);
    }
    #[test]
    fn should_do_get() {
        let response=corr_templates::json::extractable::parser::parse(r#"{
                       	"results": [<% for (person:Object in persons){%>{
                       		"id": {{person.id}},
                       		"name": {{person.name}},
                       		"gender": {{person.gender}},
                       		}<%}%>]}"#);
        let mut headers:HashMap<String,Variable>=HashMap::new();
        headers.insert(format!("Content-Type"),Variable{
            name:format!("ct"),
            data_type:Option::Some(VarType::String)
        });
        let step=GetStep{
            rest:RestData{
                url:corr_templates::text::parser::parse("https://atmnk-swapi.herokuapp.com/api/people").unwrap(),
                response,
                headers:HashMap::new(),
                responseHeaders:headers
            }
        };
        let mut runtime=Environment::new_rc(MockChannel);
        step.execute(&runtime);
        assert_eq!(runtime.read(Variable{
            name:format!("persons.size"),
            data_type:Option::None
        }),Value::Long(82));
    }
    #[test]
    fn should_do_get_capturing_headers() {
        let response=Option::None;
        let mut headers:HashMap<String,Variable>=HashMap::new();
        headers.insert(format!("Content-Type"),Variable{
            name:format!("ct"),
            data_type:Option::Some(VarType::String)
        });
        let step=GetStep{
            rest:RestData{
                url:corr_templates::text::parser::parse("https://atmnk-swapi.herokuapp.com/api/people").unwrap(),
                response,
                headers:HashMap::new(),
                responseHeaders:headers
            }
        };
        let mut runtime=Environment::new_rc(MockChannel);
        step.execute(&runtime);
        assert_eq!(runtime.read(Variable{
            name:format!("ct"),
            data_type:Option::None
        }),Value::String("application/json; charset=utf-8".to_string()));
    }
}