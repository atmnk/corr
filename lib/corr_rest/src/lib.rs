extern crate reqwest;
extern crate serde_json;
use corr_core::runtime::{Environment, Value};
use corr_templates::json::{Json};
use corr_journeys::Executable;
use corr_templates::json::extractable::{ExtractableJson, Extractable};
use reqwest::header::CONTENT_TYPE;
use corr_templates::text::Text;
use std::collections::HashMap;
use corr_templates::Fillable;

pub struct RestData{
    pub url:Text,
    pub response:Option<ExtractableJson>,
    pub headers:HashMap<String,Text>
}
pub struct PostStep{
    pub rest:RestData,
    pub body:BodyData,
}
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
impl Executable for PostStep {
    fn execute(&self, runtime: &Environment) {
        let client = reqwest::blocking::Client::new();
        let request_body=&self.body.fill(runtime);
        let mut initial= client
            .post(&self.rest.url.fill(runtime).to_string());

        for key in self.rest.headers.keys() {
            initial = initial.header(key,self.rest.headers.get(key).unwrap().fill(runtime).to_string())
        }

        let body=initial.body(request_body).send().unwrap().text().unwrap();


        if let Option::Some(resp)=&self.rest.response{
            let val=serde_json::from_str(body.as_str()).unwrap();
            let corr_val=Value::from(&val);
            resp.extract(corr_val,runtime);
        }
    }
}
pub struct PutStep{
    pub rest:RestData,
    pub body:BodyData,
}
impl Executable for PutStep {
    fn execute(&self, runtime: &Environment) {
        let client = reqwest::blocking::Client::new();
        let request_body=&self.body.fill(runtime);
        let mut initial= client
            .put(&self.rest.url.fill(runtime).to_string());

        for key in self.rest.headers.keys() {
            initial = initial.header(key,self.rest.headers.get(key).unwrap().fill(runtime).to_string())
        }

        let body=initial.body(request_body).send().unwrap().text().unwrap();


        if let Option::Some(resp)=&self.rest.response{
            let val=serde_json::from_str(body.as_str()).unwrap();
            let corr_val=Value::from(&val);
            resp.extract(corr_val,runtime);
        }
    }
}
pub struct PatchStep{
    pub rest:RestData,
    pub body:BodyData,
}
impl Executable for PatchStep {
    fn execute(&self, runtime: &Environment) {
        let client = reqwest::blocking::Client::new();
        let request_body=&self.body.fill(runtime);
        let mut initial= client
            .patch(&self.rest.url.fill(runtime).to_string());

        for key in self.rest.headers.keys() {
            initial = initial.header(key,self.rest.headers.get(key).unwrap().fill(runtime).to_string())
        }

        let body=initial.body(request_body).send().unwrap().text().unwrap();


        if let Option::Some(resp)=&self.rest.response{
            let val=serde_json::from_str(body.as_str()).unwrap();
            let corr_val=Value::from(&val);
            resp.extract(corr_val,runtime);
        }
    }
}
pub struct GetStep{
    pub rest:RestData
}
impl Executable for GetStep{
    fn execute(&self, runtime: &Environment) {
        let client = reqwest::blocking::Client::new();
        let mut initial= client
            .get(&self.rest.url.fill(runtime).to_string());

        for key in self.rest.headers.keys() {
            initial = initial.header(key,self.rest.headers.get(key).unwrap().fill(runtime).to_string())
        }

        let body= initial.send().unwrap().text().unwrap();


        if let Option::Some(resp)=&self.rest.response{
            let val=serde_json::from_str(&body).unwrap();
            let corr_val=Value::from(&val);
            resp.extract(corr_val,runtime);
        }

    }
}
pub struct DeleteStep{
    pub rest:RestData
}
impl Executable for DeleteStep{
    fn execute(&self, runtime: &Environment) {
        let client = reqwest::blocking::Client::new();
        let mut initial= client
            .delete(&self.rest.url.fill(runtime).to_string());

        for key in self.rest.headers.keys() {
            initial = initial.header(key,self.rest.headers.get(key).unwrap().fill(runtime).to_string())
        }

        let body= initial.send().unwrap().text().unwrap();

        if let Option::Some(resp)=&self.rest.response{
            let val=serde_json::from_str(&body).unwrap();
            let corr_val=Value::from(&val);
            resp.extract(corr_val,runtime);
        }

    }
}
#[cfg(test)]
mod tests{
    use crate::{PostStep, RestData, GetStep, BodyData};
    use corr_templates;
    use corr_templates::json::parser::parse;
    use corr_core::runtime::{Environment, ValueProvider, Variable, Value};
    use corr_journeys::Executable;
    use std::collections::HashMap;
    use corr_templates::text::Text;
    use reqwest::header::CONTENT_TYPE;

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
                headers
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
                headers
            }
        };
        let runtime=Environment::new_rc(MockChannel);
        step.execute(&runtime);
        println!("{:?}",(*runtime.channel).borrow().reference_store)
    }
}