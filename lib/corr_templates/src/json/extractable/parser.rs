use nom::{IResult, InputTakeAtPosition, AsChar};
use corr_core::runtime::{Variable, VarType};
use nom::combinator::{map, opt};
use nom::branch::alt;
use nom::sequence::{delimited, tuple, terminated};
use nom::bytes::complete::{escaped, is_not, tag};
use nom::character::complete::{anychar, char, multispace0};
use std::str;
use nom::error::ParseError;
use nom::multi::many0;
use nom::error_position;
use std::collections::HashMap;
use crate::json::extractable::{ExtractableJson, CaptuarableArray};
use crate::parser::{ws, identifier, var_scriplet, var_type, string_lit};

#[derive(Debug,PartialEq)]
pub struct Pair{
    key:String,
    value:ExtractableJson
}
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
fn json<'a>(i: &[u8])->IResult<&[u8], ExtractableJson>{
    alt((
        map(capturable_array, |ca|ExtractableJson::TemplatedDynamicArray(ca)),
        map(array, |val|ExtractableJson::TemplatedStaticArray(val)),
        map(object, |map|ExtractableJson::Object(map)),
        map(var_scriplet, |var|ExtractableJson::Variable(var)),
    ))(i)
}