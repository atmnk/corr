use crate::io::StringIO;
use serde::{Serialize, Deserialize, Serializer};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Messanger<T> where T:StringIO{
    pub string_io:Box<T>
}
impl<T> Messanger<T> where T:StringIO{
    pub fn new(str_io:T)->Messanger<T>{
        Messanger{
            string_io:Box::new(str_io)
        }
    }
    pub fn ask(&mut self, var_desc: VariableDesciption)->RawVariableValue {
        self.string_io.write(format!("Please enter value for {} of type {:?}",var_desc.name,var_desc.data_type));
        let line = self.string_io.read();
        RawVariableValue {
            value:Option::Some(line),
            name:var_desc.name.clone(),
            data_type:var_desc.data_type.clone()
        }

    }

    pub fn tell(&mut self, words: String) {
        self.string_io.write(words);
    }
}

#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub struct VariableDesciption {
    pub name:String,
    pub data_type:VarType
}

#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub enum VarType{
    String,
    Long,
    Boolean,
    Double
}
#[derive(Debug,PartialEq,Clone)]
pub enum Value{
    String(String),
    Long(i64),
    Boolean(bool),
    Double(f64),
    Object(HashMap<String,Value>),
    Array(Vec<Value>),
    Null
}
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match self {
            Value::String(val)=>{
                serializer.serialize_str(val.as_str())
            },
            Value::Long(val)=>{
                serializer.serialize_i64(*val)
            },
            Value::Double(val)=>{
                serializer.serialize_f64(*val)
            },
            Value::Null=>{
                serializer.serialize_none()
            },
            Value::Boolean(val)=>{
                serializer.serialize_bool(*val)
            },
            Value::Object(val)=>{
                serializer.serialize_some(val)
            },
            Value::Array(val)=>{
                serializer.serialize_some(val)
            }
        }
    }
}
#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub struct RawVariableValue{
    pub value:Option<String>,
    pub name:String,
    pub data_type:VarType

}
impl RawVariableValue{
    pub fn is_valid(&self)-> bool {
        match &self.value {
            Option::None=> return true,
            _=>{}
        }
        match &self.data_type {
            VarType::String=>true,
            VarType::Long=>match self.value.clone().unwrap().parse::<i64>() {
                Ok(val)=>true,
                _=>false
            },
            VarType::Boolean=>match self.value.clone().unwrap().parse::<bool>() {
                Ok(val)=>true,
                _=>false
            },
            VarType::Double=>match self.value.clone().unwrap().parse::<f64>() {
                Ok(val)=>true,
                _=>false
            }
        }
    }
    pub fn to_value(&self)->Value{
        match &self.value {
            Option::None=> return Value::Null,
            _=>{}
        }
        match &self.data_type {
            VarType::String=>Value::String(self.value.clone().unwrap()),
            VarType::Long=>match self.value.clone().unwrap().parse::<i64>() {
                Ok(val)=>Value::Long(val),
                _=>Value::Null
            },
            VarType::Boolean=>match self.value.clone().unwrap().parse::<bool>() {
                Ok(val)=>Value::Boolean(val),
                _=>Value::Null
            },
            VarType::Double=>match self.value.clone().unwrap().parse::<f64>() {
                Ok(val)=>Value::Double(val),
                _=>Value::Null
            }
        }
    }
}

pub trait ValueProvider{
    fn read(&mut self,variable:Variable)->Value;

    //    fn iterate<F>(&mut self,refering_as:Variable,to_list:Variable,inner:F) where F:Fn();
    fn write(&mut self,text:String);
    fn close(&mut self);
}
pub struct Environment<T>{
    pub channel:Rc<RefCell<T>>
}

impl<T> Environment<T> where T:ValueProvider{
    pub fn iterate<F>(&self, refering_as: Variable, to_list: Variable, inner: F) where F: Fn(usize) {
        let mut length = (*self.channel).borrow_mut().read(Variable{
            name:format!("{}.size",to_list.name),
            data_type:Option::Some(VarType::Long)
        });
        match length {
            Value::Long(l)=>{
                let size = l as usize;
                for i in 0..size {
                    inner(i);
                }
            },
            _=>{}
        }

    }
}

impl<T> ValueProvider for Environment<T> where T:ValueProvider{
    fn read(&mut self, variable: Variable) -> Value {
        (*self.channel).borrow_mut().read(variable)
    }

    fn write(&mut self, text: String) {
        (*self.channel).borrow_mut().write(text);
    }

    fn close(&mut self) {
        (*self.channel).borrow_mut().close();
    }
}

#[derive(Clone,PartialEq,Debug)]
pub struct Variable{
    pub name:String,
    pub data_type:Option<VarType>
}

#[cfg(test)]
mod tests{
    use crate::runtime::Variable;
    use crate::runtime::Environment;
    use crate::runtime::{ValueProvider, VarType};
    use std::collections::HashMap;
    use crate::runtime::Value;
    use std::rc::Rc;
    use std::cell::RefCell;

    extern crate serde_json;
    #[test]
    fn should_serialize_string_value(){
        assert_eq!(serde_json::to_string(&Value::String(format!("hello"))).unwrap(),format!("\"hello\""))
    }
    #[test]
    fn should_serialize_long_value(){
        assert_eq!(serde_json::to_string(&Value::Long(34)).unwrap(),format!("34"))
    }
    #[test]
    fn should_serialize_double_value(){
        assert_eq!(serde_json::to_string(&Value::Double(34.00)).unwrap(),format!("34.0"))
    }
    #[test]
    fn should_serialize_null_value(){
        assert_eq!(serde_json::to_string(&Value::Null).unwrap(),format!("null"))
    }
    #[test]
    fn should_serialize_object_value(){
        let mut map=HashMap::new();
        map.insert(format!("hello"),Value::String(format!("hello")));
        assert_eq!(serde_json::to_string(&Value::Object(map)).unwrap(),format!(r#"{{"hello":"hello"}}"#))
    }
    #[test]
    fn should_serialize_array_value(){
        let mut array=Vec::new();
        array.push(Value::String(format!("hello")));
        assert_eq!(serde_json::to_string(&Value::Array(array)).unwrap(),format!(r#"["hello"]"#))
    }

    impl ValueProvider for MockProvider{
        fn read(&mut self, _: Variable) -> Value { let ret=self.1[self.0].clone(); self.0+=1;ret }
        fn write(&mut self, str: String) { println!("{}",str) }
        fn close(&mut self) { unimplemented!() }
    }
    struct MockProvider(usize,Vec<Value>);
    #[test]
    fn should_iterate_over_runtime(){
        let rt=Environment{
            channel: Rc::new(RefCell::new(MockProvider(0,vec![Value::Long(3),
            Value::String(format!("Atmaram 0")),
            Value::Long(2),
            Value::String(format!("Atmaram 00")),
            Value::String(format!("Atmaram 01")),
            Value::String(format!("Atmaram 1")),
            Value::Long(2),
            Value::String(format!("Atmaram 10")),
            Value::String(format!("Atmaram 11")),
            Value::String(format!("Atmaram 2")),
            Value::Long(2),
            Value::String(format!("Atmaram 20")),
            Value::String(format!("Atmaram 21"))
            ])))
        };
        let mch = Rc::clone(&rt.channel);
        rt.iterate(Variable{
            name:format!("hobby"),
            data_type:Option::Some(VarType::String)
        }, Variable{
            name:format!("hobbies"),
            data_type:Option::None
        }, |i| {
            assert_eq!(mch.borrow_mut().read(Variable{
                name:format!("name"),
                data_type:Option::Some(VarType::String)
            }),Value::String(format!("Atmaram {}",i)));
            rt.iterate(Variable{
                name:format!("category"),
                data_type:Option::Some(VarType::String)
            }, Variable{
                name:format!("categories"),
                data_type:Option::None
            }, |j| {
                assert_eq!(mch.borrow_mut().read(Variable{
                    name:format!("name"),
                    data_type:Option::Some(VarType::String)
                }),Value::String(format!("Atmaram {}{}",i,j)))
            });
        })
    }
}