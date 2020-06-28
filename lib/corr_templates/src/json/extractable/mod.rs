use super::corr_core::runtime::{Variable, Value, Environment, VarType};
use std::collections::HashMap;
use crate::Extractable;

pub mod parser;
#[derive(Clone,PartialEq,Debug)]
pub struct CaptuarableArray{
    as_var:Variable,
    in_var:Variable,
    inner_json:Box<ExtractableJson>
}
#[derive(Clone,PartialEq,Debug)]
pub enum ExtractableJson {
    Variable(Variable),
    TemplatedStaticArray(Vec<ExtractableJson>),
    TemplatedDynamicArray(CaptuarableArray),
    Object(HashMap<String,ExtractableJson>),
}

impl Extractable for ExtractableJson {
    fn extract(&self, val: Value, runtime: &Environment) {
        println!("{}",val.to_string());
        match self {
            ExtractableJson::Variable(var)=>{
                var.extract(val,runtime)
            },
            ExtractableJson::TemplatedStaticArray(arr)=>{
                if let Option::Some(val_type)=val.get_associated_var_type(){
                    if val_type == VarType::List{
                        if let Value::Array(vec) = val {
                            let mut  counter=0;
                            for val in arr{
                                if let Option::Some(val_at_index)=vec.get(counter){
                                    val.extract(val_at_index.clone(),runtime);
                                } else {
                                    break;
                                }
                                counter+=1;
                            }
                        }

                    } else {
                        runtime.error(format!("Expected Array found {:?}",val))
                    }

                }

            },
            ExtractableJson::TemplatedDynamicArray(ca)=>{
                if let Option::Some(val_type)=val.get_associated_var_type(){
                    if val_type == VarType::List{
                        if let Value::Array(vec) = val {
                            runtime.iterate_outside_building_inside(ca.as_var.clone(),ca.in_var.clone(),vec.len(),|i|{
                                if let Option::Some(val_at_index)=vec.get(i){
                                   (&ca.inner_json).extract(val_at_index.clone(),runtime);
                                }
                            });
                        } else {
                            runtime.error(format!("Expected Array found {:?}",val))
                        }
                    }

                } else {
                    runtime.error(format!("Expected Array found {:?}",val))
                }

            },
            ExtractableJson::Object(extract_map)=>{
                if let Option::Some(val_type)=val.get_associated_var_type(){
                    if val_type == VarType::Object{
                        if let Value::Object(map) = val {
                            for key in extract_map.keys(){
                                if let Some(value) = map.get(key){
                                    extract_map.get(key).unwrap().extract(
                                        value.clone(),runtime);
                                } else {
                                    runtime.error(format!("Expected Object {:?} to have key {:?}", map,key))
                                }
                            }
                        } else {
                            runtime.error(format!("Expected Object found {:?}",val))
                        }
                    } else {
                        runtime.error(format!("Expected Object found {:?}",val))
                    }

                }

            }
        }
    }
}
#[cfg(test)]
mod tests{
    use crate::json::extractable::parser::parse;
    use super::super::corr_core::runtime::{ValueProvider, Variable, Value, Environment};
    use nom::lib::std::collections::HashMap;
    use crate::Extractable;

    impl ValueProvider for MockProvider{


        fn read(&mut self, var: Variable) -> Value {
            for (val,value) in &self.0 {
                if *val == var.name {
                    return value.clone()
                }
            }
            return Value::Null
        }
        fn write(&mut self, str: String) { println!("{}",str) }
        fn close(&mut self) {  }
        fn set_index_ref(&mut self, _: Variable, _: Variable) { unimplemented!() }
        fn drop(&mut self, _: std::string::String) { unimplemented!() }

        fn load_ith_as(&mut self, _i: usize, _index_ref_var: Variable, _list_ref_var: Variable) {
            unimplemented!()
        }

        fn save(&self, _var: Variable, _value: Value) {
            unimplemented!()
        }

        fn load_value_as(&mut self, _ref_var: Variable, _val: Value) {
            unimplemented!()
        }
    }
    #[derive(Debug)]
    struct MockProvider(Vec<(String,Value)>);
    #[test]
    fn should_extract_boolean_variable(){
        let tmp=parse(r#"{{is_male:Boolean}}"#).unwrap();
        let rt=Environment::new_rc(MockProvider(vec![]));
        tmp.extract(Value::Boolean(true),&rt);
        println!("{:?}",(*rt.channel).borrow().reference_store);
    }
    #[test]
    fn should_extract_string_variable(){
        let tmp=parse(r#"{{is_male:String}}"#).unwrap();
        let rt=Environment::new_rc(MockProvider(vec![]));
        tmp.extract(Value::String(format!("Hello")),&rt);
        println!("{:?}",(*rt.channel).borrow().reference_store);
    }
    #[test]
    fn should_extract_static_array(){
        let tmp=parse(r#"[{{is_male:Boolean}},{{name:String}}]"#).unwrap();
        let rt=Environment::new_rc(MockProvider(vec![]));
        tmp.extract(Value::Array(vec![Value::Boolean(true),Value::String(format!("Atmaram"))]),&rt);
        println!("{:?}",(*rt.channel).borrow().reference_store);
    }
    #[test]
    fn should_extract_dynamic_array(){
        let tmp=parse(r#"[<%for (abc:String in pqr){%>{{abc}}<%}%>]"#).unwrap();
        let rt=Environment::new_rc(MockProvider(vec![]));
        tmp.extract(Value::Array(vec![Value::String(format!("Atmaram")),Value::String(format!("Naik"))]),&rt);
        println!("{:?}",(*rt.channel).borrow().reference_store);
    }
    #[test]
    fn should_extract_object(){
        let tmp=parse(r#"{"name":{{name}},"age":{{age}}}"#).unwrap();
        let rt=Environment::new_rc(MockProvider(vec![]));
        let mut map=HashMap::new();
        map.insert(format!("name"),Value::String(format!("Atmaram")));
        map.insert(format!("age"),Value::Double(34.12));
        tmp.extract(Value::Object(map),&rt);
        println!("{:?}",(*rt.channel).borrow().reference_store);
    }
    #[test]
    fn should_print_error_when_key_not_present(){
        let tmp=parse(r#"{"name":{{name}},"age":{{age}}}"#).unwrap();
        let rt=Environment::new_rc(MockProvider(vec![]));
        let mut map=HashMap::new();
        map.insert(format!("name"),Value::String(format!("Atmaram")));
        tmp.extract(Value::Object(map),&rt);
        println!("{:?}",(*rt.channel).borrow().reference_store);
    }
    #[test]
    fn should_print_error_when_object_not_present(){
        let tmp=parse(r#"{"name":{{name}},"age":{{age}}}"#).unwrap();
        let rt=Environment::new_rc(MockProvider(vec![]));
        let mut map=HashMap::new();
        map.insert(format!("name"),Value::String(format!("Atmaram")));
        tmp.extract(Value::Array(vec![]),&rt);
        println!("{:?}",(*rt.channel).borrow().reference_store);
    }
    #[test]
    fn should_print_error_when_array_not_present(){
        let tmp=parse(r#"[{{name}}]"#).unwrap();
        let rt=Environment::new_rc(MockProvider(vec![]));
        let mut map=HashMap::new();
        map.insert(format!("name"),Value::String(format!("Atmaram")));
        tmp.extract(Value::Object(map),&rt);
        println!("{:?}",(*rt.channel).borrow().reference_store);
    }
}