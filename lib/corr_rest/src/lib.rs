use corr_core::runtime::{Environment, ValueProvider};
use corr_templates::json::parser::parse;
use corr_templates::json::{Json, Fillable};
use corr_journeys::Executable;
pub struct PostStep;
impl<T> Executable<T> for PostStep where T:ValueProvider{
    fn execute(&self, runtime: &Environment<T>) {
        let tmpl=parse(r#"{"name":[<%for(abc:String in pqr){%>{{abc}}<%}%>]}"#);
        tmpl.unwrap().fill(runtime);
    }
}

pub struct GetStep;
impl<T> Executable<T> for GetStep where T:ValueProvider{
    fn execute(&self, runtime: &Environment<T>) {
        let template=r#"{"name":[<%for(abc:Object in pqr){%>[<%for(xyz:String in abc.name){%>{{xyz}}<%}%>]<%}%>]}"#;
        (*runtime.channel).borrow_mut().write(format!("{:?}",template));
        let tmpl=parse(template);
        let val=tmpl.unwrap().fill(runtime);
        (*runtime.channel).borrow_mut().write(format!("{:?}",val));

    }
}