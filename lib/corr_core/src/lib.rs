use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::collections::HashMap;

#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub enum DesiredAction{
    Tell(VariableDesciption),
    Listen(String),
    Quit
}
#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub enum Action{
    Told(RawVariableValue),
    Quit,
    Ping,
    Pong,
    Ignorable
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
//#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
//pub enum Variable{
//    String(String,String),
//    Long(String,String),
//    Boolean(String,String),
//    Double(String,String),
//    Null(String)
//}
//impl Variable{
//    pub fn data_type(&self)->VarType{
//        match self {
//            Variable::String(_,_)=>VarType::String,
//            Variable::Long(_,_)=>VarType::Long,
//            Variable::Boolean(_,_)=>VarType::Boolean,
//            Variable::Double(_,_)=>VarType::Double,
//            Variable::Null(_)=>VarType::Any,
//        }
//    }
//    pub fn is_valid(&self)-> bool {
//        match self {
//            Variable::String(_,val)=>true,
//            Variable::Long(_,val)=>match val.parse::<i64>() {
//                Ok(val)=>true,
//                _=>false
//            },
//            Variable::Boolean(_,val)=>match val.parse::<bool>() {
//                Ok(val)=>true,
//                _=>false
//            },
//            Variable::Double(_,val)=>match val.parse::<f64>() {
//                Ok(val)=>true,
//                _=>false
//            },
//            Variable::Null(_)=> true,
//        }
//    }
//    pub fn to_value(&self)->Value{
//        match self {
//            Variable::String(_,val)=>Value::String(val.clone()),
//            Variable::Long(_,val)=>match val.parse::<i64>() {
//                Ok(val)=>Value::Long(val.clone()),
//                _=>Value::Long(0)
//            },
//            Variable::Boolean(_,val)=>match val.parse::<bool>() {
//                Ok(val)=>Value::Boolean(val),
//                _=>Value::Boolean(false)
//            },
//            Variable::Double(_,val)=>match val.parse::<f64>() {
//                Ok(val)=>Value::Double(val),
//                _=>Value::Double(0.0)
//            },
//            Variable::Null(_)=> Value::Null,
//        }
//    }
//}
#[cfg(test)]
mod tests{
    use crate::Value;
    use std::collections::HashMap;
    use self::serde_json::Map;

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
}


