use nom::{IResult, InputTakeAtPosition, AsChar};
use corr_core::runtime::{Variable, VarType, Value};
use nom::sequence::{tuple, terminated, delimited};
use nom::bytes::complete::{tag, escaped, is_not};
use nom::error::ParseError;
use nom::character::complete::{multispace0, anychar, digit1,char};
use nom::combinator::{map, opt};
use nom::branch::alt;
use std::str;
use nom::multi::many0;
use crate::{Argument, Function};

type ParserError<'a, T> = Result<(&'a [u8], T), nom::Err<(&'a [u8], nom::error::ErrorKind)>>;
#[inline]
pub fn non_ascii(chr: u8) -> bool {
    chr >= 0x80 && chr <= 0xFD
}
pub fn ws<I, O, E: ParseError<I>, F>(inner: F) -> impl Fn(I) -> IResult<I, O, E>
    where
        F: Fn(I) -> IResult<I, O, E>,
        I: InputTakeAtPosition,
        <I as InputTakeAtPosition>::Item: AsChar + Clone,
{
    move |input: I| {
        let (input, _) = multispace0(input)?;
        terminated(&inner,multispace0)(input)
    }
}
pub fn identifier(input: &[u8]) -> ParserError<&str> {
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
pub fn variable_expression(i: &[u8]) -> IResult<&[u8], Variable>{
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
pub fn var_scriplet(i: &[u8]) -> IResult<&[u8], Variable> {
    let fun = tuple((
        tag("{{"),
        ws(variable_expression),
        tag("}}")
    ));
    let (i,(_,expr,_)) = fun(i)?;
    Ok((i,expr))
}
pub fn var_type(i: &[u8]) -> IResult<&[u8], Option<VarType>> {
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
pub fn args(i:&[u8])->IResult<&[u8],Vec<Argument>>{
    let fun = tuple((ws(tag("(")),many0(terminated(arg,ws(tag(",")))),opt(arg),ws(tag(")"))));
    let (i,(_,start_pairs,opt_last_pair,_)) = fun(i)?;
    let mut  vec=Vec::new();
    for  val in start_pairs {
        vec.push(val.clone());
    }
    if opt_last_pair.is_some() {
        let value=opt_last_pair.unwrap();
        vec.push(value.clone())
    }
    Ok((i,vec))

}
pub fn arg(i:&[u8])->IResult<&[u8],Argument> {
    alt(
        (final_arg,
         function_arg,
         variable_arg
        )
    )(i)
}

pub fn function_expression(i: &[u8]) -> IResult<&[u8], Function>{
    let (i,(f_name,f_args))=tuple((identifier,args))(i)?;
    Ok((i,Function{
        name:f_name.to_string(),
        args:f_args
    }))
}

pub fn function_scriplet(i: &[u8]) -> IResult<&[u8], Function> {
    let fun = tuple((
        tag("{{"),
        ws(function_expression),
        tag("}}")
    ));
    let (i,(_,expr,_)) = fun(i)?;
    Ok((i,expr))
}
pub fn null_lit(i: &[u8]) -> IResult<&[u8], ()> {
    map(ws(alt((tag("null"),tag("Null"),tag("NULL")))), |_s| ())(i)
}
pub fn string_lit(i: &[u8]) -> IResult<&[u8], &str> {
    map(
        ws(delimited(
            char('\"'),
            opt(escaped(is_not("\\\""), '\\', anychar)),
            char('\"'),
        )),
        |s| s.map(|s| str::from_utf8(s).unwrap()).unwrap_or(""),
    )(i)
}
pub fn long_lit(i: &[u8]) -> IResult<&[u8], i64> {
    let num = ws(tuple((opt(tag("-")),digit1)));
    let (i,(sign,nums)) = num(i)?;
    match sign.map(|s| str::from_utf8(s).unwrap()).unwrap_or("") {
        "-"=>Ok((i,str::from_utf8(nums).unwrap().parse::<i64>().unwrap()*-1)),
        _=>Ok((i,str::from_utf8(nums).unwrap().parse::<i64>().unwrap()))
    }

}
pub fn double_lit(i: &[u8]) -> IResult<&[u8], f64> {
    let num = ws(tuple((opt(tag("-")),digit1,tag("."),digit1)));
    let (i,(sign,char_nums,_,mant_num)) = num(i)?;
    let str_num=format!("{}.{}",str::from_utf8(char_nums).unwrap(),str::from_utf8(mant_num).unwrap());
    let f_num=str_num.parse::<f64>().unwrap();
    match sign.map(|s| str::from_utf8(s).unwrap()).unwrap_or("") {
        "-"=>Ok((i,(f_num * -1.0))),
        _=>Ok((i,f_num))
    }

}
pub fn boolean_lit(i: &[u8]) -> IResult<&[u8], bool> {
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
pub fn final_arg(i:&[u8])->IResult<&[u8],Argument> {
    alt((
        map(null_lit,|_val|{Argument::Final(Value::Null)}),
        map(boolean_lit,|val|{Argument::Final(Value::Boolean(val))}),
        map(string_lit, |val|{Argument::Final(Value::String(val.to_string()))}),
        map(double_lit, |val| Argument::Final(Value::Double(val))),
        map(long_lit, |val|Argument::Final(Value::Long(val)))
    ))(i)
}

pub fn function_arg(i:&[u8])->IResult<&[u8],Argument> {
    let (i,func)=function_expression(i)?;
    Ok((i,Argument::Function(func)))
}
pub fn variable_arg(i:&[u8])->IResult<&[u8],Argument> {
    let (i,var)=variable_expression(i)?;
    Ok((i,Argument::Variable(var)))
}