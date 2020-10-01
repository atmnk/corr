use serde::{Deserialize, Serialize};
use nom::lib::std::collections::HashMap;
use crate::core::runtime::HeapObject;
use std::sync::Arc;
use futures::lock::Mutex;

pub mod proto;
pub mod runtime;
pub mod parser;
#[derive(Debug, Clone,Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum DataType {
    String,
    Double,
    PositiveInteger,
    Integer,
    Boolean,
    List,
    Object
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Value{
    String(String),
    PositiveInteger(usize),
    Integer(i64),
    Boolean(bool),
    Double(f64),
    Null,
    Array(Vec<Value>),
    Map(HashMap<String,Value>)
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Number{
    PositiveInteger(usize),
    Integer(i64),
    Double(f64)
}
impl Number{
    pub fn to_value(&self)->Value{
        match self {
            Number::PositiveInteger(lng)=>Value::PositiveInteger(lng.clone()),
            Number::Integer(lng)=>Value::Integer(lng.clone()),
            Number::Double(dbl)=>Value::Double(dbl.clone())
        }
    }
    pub fn add(&self,number:Number)->Number{
        match self {
            Number::PositiveInteger(lng1)=> {
                match number {
                    Number::PositiveInteger(lng2)=>Number::PositiveInteger(lng1+lng2),
                    Number::Integer(lng2)=>Number::Integer(lng1.clone() as i64+lng2),
                    Number::Double(dbl1)=>Number::Double(lng1.clone() as f64+dbl1)
                }
            },
            Number::Integer(lng1)=> {
                match number {
                    Number::PositiveInteger(lng2)=>Number::Integer(lng1+lng2 as i64),
                    Number::Integer(lng2)=>Number::Integer(lng1+lng2),
                    Number::Double(dbl1)=>Number::Double(lng1.clone() as f64+dbl1)
                }
            },
            Number::Double(dbl1)=> {
                match number {
                    Number::PositiveInteger(lng1)=>Number::Double(dbl1+lng1 as f64),
                    Number::Integer(lng1)=>Number::Double(dbl1+lng1 as f64),
                    Number::Double(dbl2)=>Number::Double(dbl1+dbl2)
                }
            },
        }
    }
    pub fn multiply(&self,number:Number)->Number{
        match self {
            Number::PositiveInteger(lng1)=> {
                match number {
                    Number::PositiveInteger(lng2)=>Number::PositiveInteger(lng1*lng2),
                    Number::Integer(lng2)=>Number::Integer(lng1.clone() as i64*lng2),
                    Number::Double(dbl1)=>Number::Double(lng1.clone() as f64*dbl1)
                }
            },
            Number::Integer(lng1)=> {
                match number {
                    Number::PositiveInteger(lng2)=>Number::Integer(lng1*lng2 as i64),
                    Number::Integer(lng2)=>Number::Integer(lng1*lng2),
                    Number::Double(dbl1)=>Number::Double(lng1.clone() as f64*dbl1)
                }
            },
            Number::Double(dbl1)=> {
                match number {
                    Number::PositiveInteger(lng1)=>Number::Double(dbl1*lng1 as f64),
                    Number::Integer(lng1)=>Number::Double(dbl1*lng1 as f64),
                    Number::Double(dbl2)=>Number::Double(dbl1*dbl2)
                }
            },
        }
    }
    pub fn subtract(&self, number:Number) ->Number{
        match self {
            Number::PositiveInteger(lng1)=> {
                match number {
                    Number::PositiveInteger(lng2)=>{
                        if lng1.clone() > lng2{
                            Number::PositiveInteger(lng1-lng2)
                        } else {
                            Number::Integer(lng1.clone() as i64 - lng2 as i64)
                        }
                    },
                    Number::Integer(lng2)=>Number::Integer(lng1.clone() as i64-lng2),
                    Number::Double(dbl1)=>Number::Double(lng1.clone() as f64-dbl1)
                }
            },
            Number::Integer(lng1)=> {
                match number {
                    Number::PositiveInteger(lng2)=>Number::Integer(lng1-lng2 as i64),
                    Number::Integer(lng2)=>Number::Integer(lng1-lng2),
                    Number::Double(dbl1)=>Number::Double(lng1.clone() as f64-dbl1)
                }
            },
            Number::Double(dbl1)=> {
                match number {
                    Number::PositiveInteger(lng1)=>Number::Double(dbl1-lng1 as f64),
                    Number::Integer(lng1)=>Number::Double(dbl1-lng1 as f64),
                    Number::Double(dbl2)=>Number::Double(dbl1-dbl2)
                }
            },
        }
    }
    pub fn divide(&self,number:Number)->Number{
        match self {
            Number::PositiveInteger(lng1)=> {
                match number {
                    Number::PositiveInteger(lng2)=>{
                        Number::PositiveInteger(lng1/lng2)
                    },
                    Number::Integer(lng2)=>Number::Integer(lng1.clone() as i64/lng2),
                    Number::Double(dbl1)=>Number::Double(lng1.clone() as f64/dbl1)
                }
            },
            Number::Integer(lng1)=> {
                match number {
                    Number::PositiveInteger(lng2)=>Number::Integer(lng1/lng2 as i64),
                    Number::Integer(lng2)=>Number::Integer(lng1-lng2),
                    Number::Double(dbl1)=>Number::Double(lng1.clone() as f64/dbl1)
                }
            },
            Number::Double(dbl1)=> {
                match number {
                    Number::PositiveInteger(lng1)=>Number::Double(dbl1/lng1 as f64),
                    Number::Integer(lng1)=>Number::Double(dbl1/lng1 as f64),
                    Number::Double(dbl2)=>Number::Double(dbl1/dbl2)
                }
            },
        }
    }
}
impl Value {
    pub fn from_json_value(value:serde_json::Value)->Self{
        match value {
            serde_json::Value::Null=>Value::Null,
            serde_json::Value::Bool(val)=>Value::Boolean(val),
            serde_json::Value::String(string)=>Value::String(string),
            serde_json::Value::Number(num)=>{
                if num.is_i64() {
                    Value::Integer(num.as_i64().unwrap())
                } else if num.is_u64() {
                    Value::PositiveInteger(num.as_u64().unwrap() as usize)
                } else {
                    Value::Double(num.as_f64().unwrap())
                }
            },
            serde_json::Value::Object(map)=>{
                let mut hm = HashMap::new();
                for (key,value) in map {
                    hm.insert(key.clone(),Value::from_json_value(value));
                }
                Value::Map(hm)
            },
            serde_json::Value::Array(vec_val)=>{
                let mut new_vec = Vec::new();
                for value in vec_val {
                    new_vec.push(Value::from_json_value(value))
                }
                Value::Array(new_vec)
            }
        }
    }
    pub fn to_heap_object(&self)->HeapObject{
        match self {
            Value::Map(map)=>{
                let mut hm=HashMap::new();
                for (key,value) in map {
                    hm.insert(key.clone(),Arc::new(Mutex::new(value.to_heap_object())));
                }
                HeapObject::Object(hm)
            },
            Value::Array(vec)=>{
                let mut vec_val=Vec::new();
                for value in vec {
                    vec_val.push(Arc::new(Mutex::new(value.to_heap_object())));
                }
                HeapObject::List(vec_val)
            },
            _=>HeapObject::Final(self.clone())
        }
    }
    pub fn to_json_value(&self)->serde_json::Value{
        match self {
            Value::Boolean(val)=>serde_json::Value::Bool(val.clone()),
            Value::String(val)=>serde_json::Value::String(val.clone()),
            Value::Double(val)=>serde_json::Value::Number(serde_json::Number::from_f64(val.clone()).unwrap()),
            Value::Integer(val)=>serde_json::Value::Number(serde_json::Number::from(val.clone())),
            Value::PositiveInteger(val)=>serde_json::Value::Number(serde_json::Number::from(val.clone())),
            Value::Null=>serde_json::Value::Null,
            Value::Map(hm)=>{
                let mut new_hm = serde_json::Map::new();
                for (key,value) in hm {
                    new_hm.insert(key.clone(),value.to_json_value());
                }
                serde_json::Value::Object(new_hm)
            },
            Value::Array(arr)=>{
                let mut new_vec = Vec::new();
                for val in arr {
                    new_vec.push(val.to_json_value())
                }
                serde_json::Value::Array(new_vec)
            }
        }
    }
    pub fn to_number(&self)->Option<Number>{
        match self {
            Value::Double(dbl)=>Option::Some(Number::Double(dbl.clone())),
            Value::PositiveInteger(lng)=>Option::Some(Number::PositiveInteger(lng.clone())),
            Value::Integer(lng)=>Option::Some(Number::Integer(lng.clone())),
            Value::String(str)=>{
                if let Ok(val) = str.parse::<usize>(){
                    Option::Some(Number::PositiveInteger(val))
                } else if let Ok(val) = str.parse::<i64>(){
                    Option::Some(Number::Integer(val))
                } else if let Ok(val) = str.parse::<f64>(){
                    Option::Some(Number::Double(val))
                } else {
                    Option::None
                }
            },
            _=>Option::None
        }
    }
    pub fn is_of_type(&self,data_type:DataType)->bool{
        match self {
            Value::Null=>true,
            _=>{
                match data_type {
                    DataType::PositiveInteger=> if let Value::PositiveInteger(_)=self{
                        true
                    } else {
                        false
                    },
                    DataType::Integer=> if let Value::Integer(_)=self{
                        true
                    } else {
                        false
                    },
                    DataType::String=> if let Value::String(_)=self{
                        true
                    } else {
                        false
                    },
                    DataType::Boolean=> if let Value::Boolean(_)=self{
                        true
                    } else {
                        false
                    },
                    DataType::Double=> if let Value::Double(_)=self{
                        true
                    } else {
                        false
                    },
                    _=> false
                }
            }
        }

    }
    pub fn to_string(&self)->String{
        match self {
            Value::String(str)=>str.clone(),
            Value::Null=>"null".to_string(),
            Value::PositiveInteger(lng)=>format!("{}",lng),
            Value::Integer(lng)=>format!("{}",lng),
            Value::Double(dbl)=>format!("{}",dbl),
            Value::Boolean(bln)=>format!("{}",bln),
            Value::Map(hm)=>{
                let mut vec=vec![];
                let mut sb= "".to_string();
                sb.push_str("{");
                for (key,value) in hm {
                    if let Value::String(_)= value {
                        vec.push(format!("\"{}\":\"{}\"",key,value.to_string()))
                    } else {
                        vec.push(format!("\"{}\":{}",key,value.to_string()))
                    }

                }
                sb.push_str(vec.join(",").as_str());
                sb.push_str("}");
                sb
            },
            Value::Array(arr)=>{
                let mut vec=vec![];
                let mut sb= "".to_string();
                sb.push_str("[");
                for value in arr {
                    if let Value::String(_)= value {
                        vec.push(format!("\"{}\"",value.to_string()))
                    } else {
                        vec.push(format!("{}",value.to_string()))
                    }

                }
                sb.push_str(vec.join(",").as_str());
                sb.push_str("]");
                sb
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableValue{
    pub name:String,
    pub value:Value
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable{
    pub name:String,
    pub data_type:Option<DataType>
}
impl Variable{
    pub fn new(name:&str)->Variable{
        return Variable{
            name:name.to_string(),
            data_type:Option::None
        }
    }
}
pub fn convert(name:String,value:String,data_type:DataType)->Option<VariableValue>{
    match data_type {
        DataType::String=>Option::Some(VariableValue{name,value:Value::String(value)}),
        DataType::PositiveInteger=>{
            if let Ok(val) = value.parse::<usize>(){
                Option::Some(VariableValue{name,value:Value::PositiveInteger(val)})
            } else {
                Option::None
            }
        },
        DataType::Integer=>{
            if let Ok(val) = value.parse::<i64>(){
                Option::Some(VariableValue{name,value:Value::Integer(val)})
            } else {
                Option::None
            }
        },
        _=>Option::None
    }
}

#[cfg(test)]
pub mod tests{
    use crate::core::{Number, Value};

    #[test]
    fn should_convert_positive_integer_to_value(){
        let a = Number::PositiveInteger(23);
        assert_eq!(a.to_value(),Value::PositiveInteger(23));
    }
    #[test]
    fn should_convert_integer_to_value(){
        let a = Number::Integer(23);
        assert_eq!(a.to_value(),Value::Integer(23));
    }
    #[test]
    fn should_convert_double_to_value(){
        let a = Number::Double(23.0);
        assert_eq!(a.to_value(),Value::Double(23.0));
    }

    #[test]
    fn should_add_positive_integer_to_positive_integer(){
        let a = Number::PositiveInteger(2).add(Number::PositiveInteger(3));
        assert_eq!(a,Number::PositiveInteger(5));
    }
    #[test]
    fn should_add_positive_integer_to_double(){
        let a = Number::Double(2.0).add(Number::PositiveInteger(3));
        assert_eq!(a,Number::Double(5.0));
    }

    #[test]
    fn should_add_double_to_positive_integer(){
        let a = Number::PositiveInteger(2).add(Number::Double(3.0));
        assert_eq!(a,Number::Double(5.0));
    }
    #[test]
    fn should_add_double_to_double(){
        let a = Number::Double(2.1).add(Number::Double(3.0));
        assert_eq!(a,Number::Double(5.1));
    }
    #[test]
    fn should_multiply_positive_integer_to_positive_integer(){
        let a = Number::PositiveInteger(2).multiply(Number::PositiveInteger(3));
        assert_eq!(a,Number::PositiveInteger(6));
    }
    #[test]
    fn should_multiply_positive_integer_to_double(){
        let a = Number::Double(2.0).multiply(Number::PositiveInteger(3));
        assert_eq!(a,Number::Double(6.0));
    }

    #[test]
    fn should_multiply_double_to_positive_integer(){
        let a = Number::PositiveInteger(2).multiply(Number::Double(3.0));
        assert_eq!(a,Number::Double(6.0));
    }
    #[test]
    fn should_multiply_double_to_double(){
        let a = Number::Double(2.0).multiply(Number::Double(3.0));
        assert_eq!(a,Number::Double(6.0));
    }

    #[test]
    fn should_subtract_positive_integer_from_positive_integer(){
        let a = Number::PositiveInteger(2).subtract(Number::PositiveInteger(3));
        assert_eq!(a,Number::Integer(-1));
    }
    #[test]
    fn should_subtract_positive_integer_from_double(){
        let a = Number::Double(2.0).subtract(Number::PositiveInteger(3));
        assert_eq!(a,Number::Double(-1.0));
    }

    #[test]
    fn should_subtract_double_from_positive_integer(){
        let a = Number::PositiveInteger(2).subtract(Number::Double(3.0));
        assert_eq!(a,Number::Double(-01.0));
    }
    #[test]
    fn should_subtract_double_from_double(){
        let a = Number::Double(2.0).subtract(Number::Double(3.0));
        assert_eq!(a,Number::Double(-1.0));
    }

    #[test]
    fn should_divide_positive_integer_from_positive_integer(){
        let a = Number::PositiveInteger(4).divide(Number::PositiveInteger(2));
        assert_eq!(a,Number::PositiveInteger(2));
    }
    #[test]
    fn should_divide_positive_integer_from_double(){
        let a = Number::Double(4.0).divide(Number::PositiveInteger(2));
        assert_eq!(a,Number::Double(2.0));
    }

    #[test]
    fn should_divide_double_from_positive_integer(){
        let a = Number::PositiveInteger(4).divide(Number::Double(2.0));
        assert_eq!(a,Number::Double(2.0));
    }
    #[test]
    fn should_divide_double_from_double(){
        let a = Number::Double(4.4).divide(Number::Double(2.0));
        assert_eq!(a,Number::Double(2.2));
    }

    #[test]
    fn should_convert_positive_integer_value_to_positive_integer_number(){
        let a = Value::PositiveInteger(2).to_number();
        assert_eq!(a.unwrap(),Number::PositiveInteger(2))
    }
    #[test]
    fn should_convert_integer_value_to_integer_number(){
        let a = Value::Integer(2).to_number();
        assert_eq!(a.unwrap(),Number::Integer(2))
    }

    #[test]
    fn should_convert_double_value_to_double_number(){
        let a = Value::Double(2.0).to_number();
        assert_eq!(a.unwrap(),Number::Double(2.0))
    }
    #[test]
    fn should_convert_string_positive_integer_value_to_positive_integer_number(){
        let a = Value::String("2".to_string()).to_number();
        assert_eq!(a.unwrap(),Number::PositiveInteger(2))
    }
    #[test]
    fn should_convert_string_integer_value_to_integer_number(){
        let a = Value::String("-2".to_string()).to_number();
        assert_eq!(a.unwrap(),Number::Integer(-2))
    }
    #[test]
    fn should_convert_string_double_value_to_double_number(){
        let a = Value::String("2.0".to_string()).to_number();
        assert_eq!(a.unwrap(),Number::Double(2.0))
    }
}
