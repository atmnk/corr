use nom::{IResult, InputTake, Compare};
use corr_core::runtime::{Variable, VarType};
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
use crate::json::extractable::{ExtractableJson, CaptuarableArray};

#[derive(Debug,PartialEq)]
pub struct Pair{
    key:String,
    value:ExtractableJson
}
type ParserError<'a, T> = Result<(&'a [u8], T), nom::Err<(&'a [u8], nom::error::ErrorKind)>>;
pub fn parse<'a>(str:&'a str) ->Option<ExtractableJson>{
    let res=parse_bytes(str.as_bytes());
    match res {
        Ok((_,ret))=>Option::Some(ret),
        _=>Option::None
    }
}
fn parse_bytes<'a>(i: &[u8])->IResult<&[u8], ExtractableJson>{
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
fn array(i: &[u8]) -> IResult<&[u8], Vec<ExtractableJson>> {
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
fn loop_scriplet(i: &[u8]) -> IResult<&[u8], CaptuarableArray>{
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
        ws(tag("%>")),
        ws(json),
        ws(tag("<%")),
        ws(tag("}")),
        ws(tag("%>"))
    ));
    let (i,(_,_,_,with,_,vt,_,on,_,_,_,inner_json,_,_,_)) = fun(i)?;
    let ca=CaptuarableArray{
        as_var:Variable{
            name:with.to_string(),
            data_type:vt
        },
        in_var:Variable{
            name:on.to_string(),
            data_type:Option::Some(VarType::List)
        },
        inner_json:Box::new(inner_json)
    };
    Ok((i,ca))
}
fn capturable_array(i: &[u8]) -> IResult<&[u8], CaptuarableArray> {
    let fun = tuple((ws(tag("[")),loop_scriplet,ws(tag("]"))));
    let (i,(_,loop_s,_)) = fun(i)?;
    Ok((i,loop_s))
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
fn pair(i: &[u8]) -> IResult<&[u8], Pair>{
    let fun= tuple((string_lit,ws(tag(":")),json));
    let (i,(key,_,value)) = fun(i)?;
    let pair_value=Pair {
        key:key.to_string(),
        value
    };
    Ok((i,pair_value))
}
fn object(i: &[u8]) -> IResult<&[u8], HashMap<String,ExtractableJson>> {
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
fn json<'a>(i: &[u8])->IResult<&[u8], ExtractableJson>{
    alt((
        map(capturable_array, |ca|ExtractableJson::TemplatedDynamicArray(ca)),
        map(array, |val|ExtractableJson::TemplatedStaticArray(val)),
        map(object, |map|ExtractableJson::Object(map)),
        map(var_scriplet, |var|ExtractableJson::Variable(var)),
    ))(i)
}