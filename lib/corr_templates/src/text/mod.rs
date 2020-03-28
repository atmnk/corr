use corr_core::runtime::{Variable, Value, ValueProvider, Environment};
use crate::json::Fillable;

pub mod parser;
#[derive(Clone,PartialEq,Debug)]
pub enum Producer{
    Text(Text),
    TextProducer(TextProducer)
}
#[derive(Clone,PartialEq,Debug)]
pub struct TextProducer{
    as_var:Variable,
    in_var:Variable,
    inner_producer:Box<Producer>
}
#[derive(Clone,PartialEq,Debug)]
pub enum TextBlock{
    Final(String),
    Variable(Variable),
    Loop(TextProducer)
}
#[derive(Clone,PartialEq,Debug)]
pub struct Text{
    blocks:Vec<TextBlock>
}
impl Fillable<Value> for Text{
    fn fill(&self, runtime: &Environment) -> Value {
        let mut vec=Vec::new();
        for block in &self.blocks{
            vec.push(block.fill(runtime));
        }
        Value::Array(vec)
    }
}
impl Fillable<Value> for Producer {
    fn fill(&self, runtime:&Environment) ->Value {
        match self {
            Producer::Text(txt)=>{
                Value::Array(vec![txt.fill(runtime)])
            },
            Producer::TextProducer(tap)=>{
                tap.fill(runtime)
            }
        }
    }
}
impl Fillable<Value> for TextProducer{
    fn fill(&self, runtime: &Environment) -> Value {
        let res=Vec::new();
        let res=runtime.build_iterate(self.as_var.clone(), self.in_var.clone(),res, |_|{
            self.inner_producer.fill(&runtime)
        });
        Value::Array(res.into_iter().map(|v|{
            match v {
                Value::Array(l)=>l,
                _=>vec![]
            }
        }).flatten().collect())
    }
}
impl Fillable<Value> for TextBlock{
    fn fill(&self, runtime: &Environment) -> Value {
        match self {
            TextBlock::Final(val)=>Value::String(val.clone()),
            TextBlock::Variable(var)=>runtime.channel.borrow_mut().read(var.clone()),
            TextBlock::Loop(tp)=>tp.fill(runtime)
        }
    }
}
#[cfg(test)]
mod tests{
    use corr_core::runtime::{Environment, Value, ValueProvider, Variable};
    use super::parser::parse;
    use crate::json::Fillable;

    impl ValueProvider for MockProvider {

        fn read(&mut self, _: Variable) -> Value {
            let ret = self.1[self.0].clone();
            self.0 += 1;
            ret
        }
        fn write(&mut self, str: String) { println!("{}", str) }
        fn close(&mut self) {}
        fn set_index_ref(&mut self, _: Variable, _: Variable) {}
        fn drop(&mut self, _: String) {}

        fn load_ith_as(&mut self, _i: usize, _index_ref_var: Variable, _list_ref_var: Variable) {}

        fn save(&self, _var: Variable, _value: Value) {
            unimplemented!()
        }

        fn load_value_as(&mut self, _ref_var: Variable, _val: Value) {
            unimplemented!()
        }
    }

    #[derive(Debug)]
    struct MockProvider(usize, Vec<Value>);
    #[test]
    fn should_parse_escape_withing_loop(){
        let val=parse(r#"Below Are All Ids<% for (id:Long in ids){%>\{id:{{id}}}<%}%>"#).unwrap();
        assert_eq!(val.fill(&Environment::new_rc(MockProvider(0,vec![Value::Long(2),Value::Long(1),Value::Long(3)]))).to_string(),r#"Below Are All Ids{id:1}{id:3}"#);
    }
}