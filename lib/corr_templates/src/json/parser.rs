use crate::json::{Json, JsonArrayProducer, Producer, JsonTimesProducer};
use nom::{IResult, InputTakeAtPosition, AsChar};
use super::corr_core::runtime::{Value, Variable, VarType};
use nom::combinator::{map, opt};
use nom::branch::alt;
use nom::sequence::{delimited, tuple, terminated};
use nom::bytes::complete::{escaped, is_not, tag};
use nom::character::complete::{anychar, char, digit1, multispace0};
use std::str;
use nom::error::ParseError;
use nom::multi::many0;
use nom::error_position;
use std::collections::HashMap;
use crate::parser::{identifier, ws, var_type, function_scriplet, var_scriplet, long_lit, string_lit, null_lit, double_lit, boolean_lit};

#[derive(Debug,PartialEq,Clone)]
pub struct Pair{
    key:String,
    value:Json
}
type ParserError<'a, T> = Result<(&'a [u8], T), nom::Err<(&'a [u8], nom::error::ErrorKind)>>;
pub fn parse<'a>(str:&'a str) ->Option<Json>{
    let res=parse_bytes(str.as_bytes());
    match res {
        Ok((_, ret)) => Option::Some(ret),
        _ => Option::None
    }
}
fn parse_bytes<'a>(i: &[u8])->IResult<&[u8], Json>{
    json(i)
}





fn array(i: &[u8]) -> IResult<&[u8], Vec<Json>> {
    let fun = tuple((ws(tag("[")),many0(terminated(json,ws(tag(",")))),opt(json),ws(tag("]"))));
    let (i,(_,start_elems,opt_last_elem,_)) = fun(i)?;
    let mut  vec=Vec::new();
    for  se in start_elems {
        vec.push(se);
    }
    if opt_last_elem.is_some() {
        let last_elem=opt_last_elem.unwrap();
        vec.push(last_elem);
    }
    Ok((i,vec))
}

fn producer_jap(i: &[u8])-> IResult<&[u8], JsonArrayProducer>{
    let fun = tuple((
        ws(tag("for")),
        ws(tag("(")),
        ws(identifier),
        ws(tag(":")),
        ws(var_type),
        ws(tag("in")),
        ws(identifier),
        ws(tag(")")),
        ws(tag("{")),
        ws(producer),
        ws(tag("}")),
    ));
    let (i,(_,_,with,_,vt,_,on,_,_,inner_producer,_)) = fun(i)?;
    let jap=JsonArrayProducer{
        as_var:Variable{
            name:with.to_string(),
            data_type:vt
        },
        in_var:Variable{
            name:on.to_string(),
            data_type:Option::Some(VarType::List)
        },
        inner_producer:inner_producer
    };
    Ok((i,jap))
}
fn producer_jtp(i: &[u8])-> IResult<&[u8], JsonTimesProducer>{
    let fun = tuple((
        ws(tag("times")),
        ws(tag("(")),
        ws(long_lit),
        ws(tag(",")),
        ws(identifier),
        ws(tag(",")),
        ws(identifier),
        ws(tag(":")),
        ws(var_type),
        ws(tag("in")),
        ws(identifier),
        ws(tag(")")),
        ws(tag("{")),
        ws(producer),
        ws(tag("}")),
    ));
    let (i,(_,_,times,_,couner,_,with,_,vt,_,on,_,_,inner_producer,_)) = fun(i)?;
    let jtp=JsonTimesProducer{
        as_var:Variable{
            name:with.to_string(),
            data_type:vt
        },
        in_var:Variable{
            name:on.to_string(),
            data_type:Option::Some(VarType::List)
        },
        counter_var:Variable{
            name:couner.to_string(),
            data_type:Option::Some(VarType::Long)
        },
        times:times as usize,
        inner_producer:inner_producer
    };
    Ok((i,jtp))
}
fn producer_json(i: &[u8])-> IResult<&[u8], Json>{
    let fun = tuple((
        ws(tag("%>")),
        ws(json),
        ws(tag("<%"))
    ));
    let (i,(_,js_element,_)) = fun(i)?;
    Ok((i,js_element))
}
fn producer(i: &[u8])-> IResult<&[u8], Box<Producer>>{
    alt((
        map(producer_jap,|jap|Box::new(Producer::JsonArrayProducer(jap))),
        map(producer_jtp,|jtp|Box::new(Producer::JsonArrayTimesProducer(jtp))),
        map(producer_json, |js|Box::new(Producer::Json(js)))
    ))(i)
}
fn loop_scriplet(i: &[u8]) -> IResult<&[u8], JsonArrayProducer>{
    let fun = tuple((
        ws(tag("<%")),
        ws(tag("for")),
        ws(tag("(")),
        ws(identifier),
        ws(tag(":")),
        ws(var_type),
        ws(tag("in")),
        ws(identifier),
        ws(tag(")")),
        ws(tag("{")),
        ws(producer),
        ws(tag("}")),
        ws(tag("%>"))
    ));
    let (i,(_,_,_,with,_,vt,_,on,_,_,inner_producer,_,_)) = fun(i)?;
    let jap=JsonArrayProducer{
        as_var:Variable{
            name:with.to_string(),
            data_type:vt
        },
        in_var:Variable{
            name:on.to_string(),
            data_type:Option::Some(VarType::List)
        },
        inner_producer:inner_producer
    };
    Ok((i,jap))
}
fn loop_times_scriplet(i: &[u8]) -> IResult<&[u8], JsonTimesProducer>{
    let fun = tuple((
        ws(tag("<%")),
        ws(tag("times")),
        ws(tag("(")),
        ws(long_lit),
        ws(tag(",")),
        ws(identifier),
        ws(tag(",")),
        ws(identifier),
        ws(tag(":")),
        ws(var_type),
        ws(tag("in")),
        ws(identifier),
        ws(tag(")")),
        ws(tag("{")),
        ws(producer),
        ws(tag("}")),
        ws(tag("%>"))
    ));
    let (i,(_,_,_,times,_,couner,_,with,_,vt,_,on,_,_,inner_producer,_,_)) = fun(i)?;
    let jtp=JsonTimesProducer{
        as_var:Variable{
            name:with.to_string(),
            data_type:vt
        },
        in_var:Variable{
            name:on.to_string(),
            data_type:Option::Some(VarType::List)
        },
        counter_var:Variable{
            name:couner.to_string(),
            data_type:Option::Some(VarType::Long)
        },
        times:times as usize,
        inner_producer:inner_producer
    };
    Ok((i,jtp))
}
fn dynamic_array(i: &[u8]) -> IResult<&[u8], JsonArrayProducer> {
    let fun = tuple((ws(tag("[")),loop_scriplet,ws(tag("]"))));
    let (i,(_,loop_s,_)) = fun(i)?;
    Ok((i,loop_s))
}
fn dynamic_times_array(i: &[u8]) -> IResult<&[u8], JsonTimesProducer> {
    let fun = tuple((ws(tag("[")),loop_times_scriplet,ws(tag("]"))));
    let (i,(_,loop_s,_)) = fun(i)?;
    Ok((i,loop_s))
}
fn pair(i: &[u8]) -> IResult<&[u8], Pair>{
    let fun= tuple((string_lit,ws(tag(":")),json));
    let (i,(key,_,value)) = fun(i)?;
    let pair_value=Pair {
        key:key.to_string(),
        value
    };
    Ok((i,pair_value))
}
fn object(i: &[u8]) -> IResult<&[u8], HashMap<String,Json>> {
    let fun = tuple((ws(tag("{")),many0(terminated(pair,ws(tag(",")))),opt(pair),ws(tag("}"))));
    let (i,(_,start_pairs,opt_last_pair,_)) = fun(i)?;
    let mut  map=HashMap::new();
    for  sp in start_pairs {
        map.insert(sp.key,sp.value);
    }
    if opt_last_pair.is_some() {
        let last_pair=opt_last_pair.unwrap();
        map.insert(last_pair.key,last_pair.value);
    }
    Ok((i,map))
}

pub fn json<'a>(i: &[u8])->IResult<&[u8], Json>{
    alt((
        map(null_lit,|_val|{Json::Constant(Value::Null)}),
        map(boolean_lit,|val|{Json::Constant(Value::Boolean(val))}),
        map(string_lit, |val|{Json::Constant(Value::String(val.to_string()))}),
        map(double_lit, |val| Json::Constant(Value::Double(val))),
        map(long_lit, |val|Json::Constant(Value::Long(val))),
        map(dynamic_array, |jap|Json::TemplatedDynamicArray(jap)),
        map(dynamic_times_array, |jta|Json::TemplatedTimesDynamicArray(jta)),
        map(array, |val|Json::TemplatedStaticArray(val)),
        map(object, |map|Json::Object(map)),
        map(function_scriplet, |fun|Json::Function(fun)),
        map(var_scriplet, |var|Json::Variable(var)),
    ))(i)
}
#[cfg(test)]
mod tests{
    use crate::json::parser::parse;
    use crate::json::{Json, JsonArrayProducer, Producer, JsonTimesProducer};
    use super::super::corr_core::runtime::{Value, Variable, VarType};
    use std::collections::HashMap;
    use crate::json::Json::{TemplatedDynamicArray, TemplatedTimesDynamicArray};
    use super::super::corr_core::runtime::VarType::List;
    use crate::{Argument, Function};

    #[test]
    fn should_parse_null(){
        assert_eq!(parse("null").unwrap(),Json::Constant(Value::Null));
    }
    #[test]
    fn should_parse_null_with_first_capital(){
        assert_eq!(parse("Null").unwrap(),Json::Constant(Value::Null));
    }
    #[test]
    fn should_parse_null_with_all_capital(){
        assert_eq!(parse("NULL").unwrap(),Json::Constant(Value::Null));
    }
    #[test]
    fn should_parse_boolean_true(){
        assert_eq!(parse("true").unwrap(),Json::Constant(Value::Boolean(true)));
    }
    #[test]
    fn should_parse_boolean_false(){
        assert_eq!(parse("false").unwrap(),Json::Constant(Value::Boolean(false)));
    }
    #[test]
    fn should_parse_boolean_treu_with_all_capital(){
        assert_eq!(parse("TRUE").unwrap(),Json::Constant(Value::Boolean(true)));
    }
    #[test]
    fn should_parse_boolean_false_with_all_capital(){
        assert_eq!(parse("FALSE").unwrap(),Json::Constant(Value::Boolean(false)));
    }
    #[test]
    fn should_parse_static_array_of_booleans(){
        assert_eq!(parse("[true,false]").unwrap(),Json::TemplatedStaticArray(vec![
            Json::Constant(Value::Boolean(true)),
            Json::Constant(Value::Boolean(false)),
        ]));
    }
    #[test]
    fn should_parse_static_array_of_mixed(){
        assert_eq!(parse("[true,null]").unwrap(),Json::TemplatedStaticArray(vec![
            Json::Constant(Value::Boolean(true)),
            Json::Constant(Value::Null),
        ]));
    }
    #[test]
    fn should_parse_object_with_null(){
        let mut map=HashMap::new();
        map.insert(format!("name"),Json::Constant(Value::Null));
        assert_eq!(parse(r#"{"name":null}"#).unwrap(),Json::Object(map));
    }
    #[test]
    fn should_parse_dynamic_array(){
        let mut map=HashMap::new();
        map.insert(format!("name"),TemplatedDynamicArray(JsonArrayProducer {
            as_var: Variable { name: format!("abc"),
            data_type: Some(VarType::String) }, in_var: Variable { name: format!("pqr"), data_type: Some(List) },
            inner_producer: Box::new(Producer::Json(Json::Variable(Variable { name: format!("abc"), data_type: None }))) }));
        assert_eq!(parse(r#"{"name":[<%for(abc:String in pqr){%>{{abc}}<%}%>]}"#).unwrap(),Json::Object(map));
    }
    #[test]
    fn should_parse_function(){
        assert_eq!(parse(r#"{{concat("Atmaram")}}"#).unwrap(),Json::Function(Function{
            name:format!("concat"),
            args:vec![Argument::Final(Value::String(format!("Atmaram")))]
        }));
    }
    #[test]
    fn should_parse_dynamic_times_array(){
        let mut map=HashMap::new();
        map.insert(format!("name"),TemplatedTimesDynamicArray(JsonTimesProducer {
            counter_var:Variable{name:format!("i"),data_type:Option::Some(VarType::Long)},
            times:2,
            as_var: Variable { name: format!("abc"),
                data_type: Some(VarType::Object) }, in_var: Variable { name: format!("pqr"), data_type: Some(List) },
            inner_producer: Box::new(Producer::Json(Json::Variable(Variable { name: format!("i"), data_type: None }))) }));
        assert_eq!(parse(r#"{"name":[<%times(2,i,abc:Object in pqr){%>{{i}}<%}%>]}"#).unwrap(),Json::Object(map));
    }
}