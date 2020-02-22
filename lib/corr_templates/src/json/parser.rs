use crate::json::{Json, JsonArrayProducer, Producer};
use nom::{IResult, InputTake, Compare};
use super::corr_core::runtime::{Value, Variable, VarType};
use nom::combinator::{map, opt};
use nom::branch::alt;
use nom::sequence::{delimited, tuple, terminated};
use nom::bytes::complete::{escaped, is_not, tag};
use nom::character::complete::{anychar, char, digit1};
use std::str;
use nom::error::ParseError;
use nom::multi::many0;
use nom::error_position;
use std::collections::HashMap;
#[derive(Debug,PartialEq)]
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
#[inline]
fn non_ascii(chr: u8) -> bool {
    chr >= 0x80 && chr <= 0xFD
}
fn ws<F, I, O, E>(inner: F) -> impl Fn(I) -> IResult<I, O, E>
    where
        F: Fn(I) -> IResult<I, O, E>,
        I: InputTake + Clone + PartialEq + for<'a> Compare<&'a [u8; 1]>,
        E: ParseError<I>,
{
    move |i: I| {
        let i = alt::<_, _, (), _>((tag(b" "), tag(b"\t")))(i.clone())
            .map(|(i, _)| i)
            .unwrap_or(i);
        let (i, res) = inner(i)?;
        let i = alt::<_, _, (), _>((tag(b" "), tag(b"\t")))(i.clone())
            .map(|(i, _)| i)
            .unwrap_or(i);
        Ok((i, res))
    }
}
fn boolean_lit(i: &[u8]) -> IResult<&[u8], bool> {
    map(ws(alt((tag("false"),tag("False"),tag("FALSE"), tag("true"),tag("True"),tag("TRUE")))), |s| {
        let val = str::from_utf8(s);
        match  val {
            Ok(bool_s) => match bool_s {
                "true" | "True" | "TRUE" => true,
                "false" | "False" | "FALSE" => false,
                _=> false
            }
            _ => false
        }
    })(i)
}
fn var_type(i: &[u8]) -> IResult<&[u8], Option<VarType>> {
    map(ws(alt((tag("List"),tag("Object"),tag("Long"), tag("Double"),tag("Boolean"),tag("String")))), |s| {
        let val = str::from_utf8(s);
        match  val {
            Ok(inner_tag) => match inner_tag {
                "Long" => Option::Some(VarType::Long),
                "Double" => Option::Some(VarType::Double),
                "Boolean" => Option::Some(VarType::Boolean),
                "String" => Option::Some(VarType::String),
                "List" => Option::Some(VarType::List),
                "Object" => Option::Some(VarType::Object),
                _=> Option::None
            }
            _ => Option::None
        }
    })(i)
}
fn null_lit(i: &[u8]) -> IResult<&[u8], ()> {
    map(ws(alt((tag("null"),tag("Null"),tag("NULL")))), |s| ())(i)
}
fn string_lit(i: &[u8]) -> IResult<&[u8], &str> {
    map(
        ws(delimited(
            char('\"'),
            opt(escaped(is_not("\\\""), '\\', anychar)),
            char('\"'),
        )),
        |s| s.map(|s| str::from_utf8(s).unwrap()).unwrap_or(""),
    )(i)
}
fn long_lit(i: &[u8]) -> IResult<&[u8], i64> {
    let num = ws(tuple((opt(tag("-")),digit1)));
    let (i,(sign,nums)) = num(i)?;
    match sign.map(|s| str::from_utf8(s).unwrap()).unwrap_or("") {
        "-"=>Ok((i,str::from_utf8(nums).unwrap().parse::<i64>().unwrap()*-1)),
        _=>Ok((i,str::from_utf8(nums).unwrap().parse::<i64>().unwrap()))
    }

}
fn double_lit(i: &[u8]) -> IResult<&[u8], f64> {
    let num = ws(tuple((opt(tag("-")),digit1,tag("."),digit1)));
    let (i,(sign,char_nums,_,mant_num)) = num(i)?;
    let str_num=format!("{}.{}",str::from_utf8(char_nums).unwrap(),str::from_utf8(mant_num).unwrap());
    let f_num=str_num.parse::<f64>().unwrap();
    match sign.map(|s| str::from_utf8(s).unwrap()).unwrap_or("") {
        "-"=>Ok((i,(f_num * -1.0))),
        _=>Ok((i,f_num))
    }

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
fn identifier(input: &[u8]) -> ParserError<&str> {
    if !nom::character::is_alphabetic(input[0]) && input[0] != b'_' && input[0] != b'.' && !non_ascii(input[0]) {
        return Err(nom::Err::Error(error_position!(
            input,
            nom::error::ErrorKind::AlphaNumeric
        )));
    }
    for (i, ch) in input.iter().enumerate() {
        if i == 0 || nom::character::is_alphanumeric(*ch) || *ch == b'_'|| *ch == b'.' || non_ascii(*ch) {
            continue;
        }
        return Ok((&input[i..], str::from_utf8(&input[..i]).unwrap()));
    }
    Ok((&input[1..], str::from_utf8(&input[..1]).unwrap()))
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
fn dynamic_array(i: &[u8]) -> IResult<&[u8], JsonArrayProducer> {
    let fun = tuple((ws(tag("[")),loop_scriplet,ws(tag("]"))));
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
fn variable_expression(i: &[u8]) -> IResult<&[u8], Variable>{
    let fun=tuple((ws(identifier),ws(opt(tuple((ws(tag(":")),ws(var_type)))))));
    let (i,(var_name,var_type))=fun(i)?;
    let data_type= match var_type {
        Option::Some((_,val))=> val,
        Option::None=>Option::None
    };
    Ok((i,Variable{
        name:var_name.to_string(),
        data_type
    }))
}
fn var_scriplet(i: &[u8]) -> IResult<&[u8], Variable> {
    let fun = tuple((
        ws(tag("{{")),
        ws(variable_expression),
        ws(tag("}}"))
    ));
    let (i,(_,expr,_)) = fun(i)?;
    Ok((i,expr))
}
fn json<'a>(i: &[u8])->IResult<&[u8], Json>{
    alt((
        map(null_lit,|val|{Json::Constant(Value::Null)}),
        map(boolean_lit,|val|{Json::Constant(Value::Boolean(val))}),
        map(string_lit, |val|{Json::Constant(Value::String(val.to_string()))}),
        map(double_lit, |val| Json::Constant(Value::Double(val))),
        map(long_lit, |val|Json::Constant(Value::Long(val))),
        map(dynamic_array, |jap|Json::TemplatedDynamicArray(jap)),
        map(array, |val|Json::TemplatedStaticArray(val)),
        map(object, |map|Json::Object(map)),
        map(var_scriplet, |var|Json::Variable(var)),
    ))(i)
}
#[cfg(test)]
mod tests{
    use crate::json::parser::parse;
    use crate::json::{Json, JsonArrayProducer, Producer};
    use super::super::corr_core::runtime::{Value, Variable, VarType};
    use std::collections::HashMap;
    use crate::json::Json::{TemplatedDynamicArray, Constant};
    use super::super::corr_core::runtime::VarType::List;
    use super::super::corr_core::runtime::Value::Boolean;

    #[test]
    fn should_parse_null(){
        assert_eq!(parse("null").unwrap(),Json::Constant(Value::Null));
    }
    #[test]
    fn should_parse_Null(){
        assert_eq!(parse("Null").unwrap(),Json::Constant(Value::Null));
    }
    #[test]
    fn should_parse_NULL(){
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
    fn should_parse_boolean_TRUE(){
        assert_eq!(parse("TRUE").unwrap(),Json::Constant(Value::Boolean(true)));
    }
    #[test]
    fn should_parse_boolean_FALSE(){
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
}