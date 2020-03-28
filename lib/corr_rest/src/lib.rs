extern crate reqwest;
extern crate serde_json;
use corr_core::runtime::{Environment, Value};
use corr_templates::json::{Json, Fillable};
use corr_journeys::Executable;
use corr_templates::json::extractable::{ExtractableJson, Extractable};
use reqwest::header::CONTENT_TYPE;
use corr_templates::text::Text;

pub struct PostStep{
    pub url:Text,
    pub body:Json,
    pub response:Option<ExtractableJson>
}
impl Executable for PostStep {
    fn execute(&self, runtime: &Environment) {
        let client = reqwest::blocking::Client::new();
        let request_body=serde_json::to_string(&self.body.fill(runtime)).unwrap();
        let body= client
            .post(&self.url.fill(runtime).to_string())
            .header(CONTENT_TYPE,"application/json")
            .body(request_body).send().unwrap().text().unwrap();
        let val=serde_json::from_str(&body).unwrap();
        let corr_val=Value::from(&val);
        if let Option::Some(resp)=&self.response{
            resp.extract(corr_val,runtime);
        }
    }
}

pub struct GetStep{
    pub url:Text,
    pub response:Option<ExtractableJson>
}
impl Executable for GetStep{
    fn execute(&self, runtime: &Environment) {
        let client = reqwest::blocking::Client::new();
        let body= client
            .get(&self.url.fill(runtime).to_string())
            .header(CONTENT_TYPE,"application/json")
            .send().unwrap().text().unwrap();
        let val=serde_json::from_str(&body).unwrap();
        let corr_val=Value::from(&val);
        if let Option::Some(resp)=&self.response{
            resp.extract(corr_val,runtime);
        }

    }
}
#[cfg(test)]
mod tests{
    use crate::PostStep;
    use corr_templates;
    use corr_templates::json::parser::parse;
    use corr_core::runtime::{Environment, ValueProvider, Variable, Value};
    use corr_journeys::Executable;

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
        let body=parse(r#"{"name":"PQR"}"#).unwrap();
        let response=corr_templates::json::extractable::parser::parse(r#"{"id":{{id}}}"#);
        let step=PostStep{
            url:corr_templates::text::parser::parse("http://localhost:8080/api/category").unwrap(),
            body,
            response
        };
        let runtime=Environment::new_rc(MockChannel);
        step.execute(&runtime);
        println!("{:?}",(*runtime.channel).borrow().reference_store)
    }
}