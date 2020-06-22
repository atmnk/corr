use nom::{IResult, InputTakeAtPosition, AsChar};
use nom::bytes::complete::{ is_not, escaped_transform, tag};
use nom::character::complete::{multispace0};
use nom::combinator::{map, opt};
use nom::sequence::{tuple, terminated};
use std::str;
use corr_core::runtime::{Variable, VarType};
use nom::branch::alt;
use nom::error::ParseError;
use crate::text::{TextProducer, Producer, TextBlock, Text};
use nom::multi::{many0};
use crate::parser::{var_scriplet, function_scriplet, identifier, ws, var_type};

fn coded_block(i: &[u8])->IResult<&[u8], TextBlock>{
    alt((
        map(loop_scriplet,|val|{TextBlock::Loop(val)}),
        map(var_scriplet,|val|{TextBlock::Variable(val)}),
        map(function_scriplet,|val|{TextBlock::Function(val)}),

    ))(i)
}
pub fn parse<'a>(str:&'a str) ->Option<Text>{
    let res=text(str.as_bytes());
    match res {
        Ok((_, ret)) => Option::Some(ret),
        _ => Option::None
    }
}
pub fn text(i: &[u8])->IResult<&[u8], Text>{
    let (i,(val,opt_last))=tuple((many0(tuple((opt(text_lit),coded_block))),opt(text_lit)))(i)?;
    let mut blocks=Vec::new();
    for (text,coded) in val{
        if let Some(tvalue)=text{
            blocks.push(TextBlock::Final(tvalue));
        }
        blocks.push(coded);
    }
    if let Some(tvalue)=opt_last{
        blocks.push(TextBlock::Final(tvalue));
    }
    Ok((i,Text{
        blocks
    }))
}
fn text_lit(i: &[u8]) -> IResult<&[u8], String> {
    map(escaped_transform(is_not(r#"\{<""#), '\\', |i: &[u8]| alt((tag("{"),tag("<"),tag("\\"),tag("\"")))(i)),
        |abc| str::from_utf8(&abc).unwrap().to_string())(i)
}

fn producer_tap(i: &[u8])-> IResult<&[u8], TextProducer>{
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
    let jap=TextProducer{
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
fn producer_text(i: &[u8])-> IResult<&[u8], Text>{
    let fun = tuple((
        ws(tag("%>")),
        ws(text),
        ws(tag("<%"))
    ));
    let (i,(_,txt_element,_)) = fun(i)?;
    Ok((i,txt_element))
}
fn producer(i: &[u8])-> IResult<&[u8], Box<Producer>>{
    alt((
        map(producer_tap,|tap|Box::new(Producer::TextProducer(tap))),
        map(producer_text, |txt|Box::new(Producer::Text(txt)))
    ))(i)
}
fn loop_scriplet(i: &[u8]) -> IResult<&[u8], TextProducer>{
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
        tag("%>")
    ));
    let (i,(_,_,_,with,_,vt,_,on,_,_,inner_producer,_,_)) = fun(i)?;
    let jap=TextProducer{
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
#[cfg(test)]
mod tests{
    use corr_core::runtime::{Variable, VarType};
    use crate::text::parser::{text_lit, var_scriplet, text, identifier};
    use crate::text::{TextBlock, Text};
    use nom::AsBytes;

    #[test]
    fn should_parse_plain_text(){
        let (_,pp)=text("Atmaram".as_bytes()).unwrap();
        assert_eq!(pp, Text{
            blocks:vec![TextBlock::Final("Atmaram".to_string())]
        });
    }
    #[test]
    fn should_parse_plain_text_until_control_char(){
        let (_,pp)=text_lit("Atmaram{Naik".as_bytes()).unwrap();
        assert_eq!(pp, "Atmaram");
    }
    #[test]
    fn should_parse_plain_text_escaping_control_char(){
        let (_,pp)=text_lit(r#"Atmaram\{Naik"#.as_bytes()).unwrap();
        assert_eq!(pp, "Atmaram{Naik");
    }
    #[test]
    fn should_parse_variable_scriplet(){
        let (_,pp)=var_scriplet("{{atmaram : String}}".as_bytes()).unwrap();
        assert_eq!(pp, Variable{
            name:format!("atmaram"),
            data_type:Option::Some(VarType::String)
        });
    }
    #[test]
    fn should_parse_loop_scriplet(){
        let (_,pp)=text(r#"<%for (abc:String in pqr){%>{{abc}}<%}%>"#.as_bytes()).unwrap();
        println!("{:?}",pp)
    }
    #[test]
    fn should_parse_loop(){
        let a=text(r#"Abc<%for (abc:String in pqr){%>{{abc}}<%}%>"#.as_bytes());
        let (_,k)=a.unwrap();
        println!("{:?}",k)
    }
    #[test]
    fn should_parse_complex(){
        let a= text(r#"ABC<%for (id:Long in ids){%>
                                                             \{
                                                                 \"id\": {{id}}
                                                             }
                                                         <%}%>"#.as_bytes());
        let (_,k)=a.unwrap();
        println!("{:?}",k)
    }
    #[test]
    fn should_parse_identifier(){
        println!("{:?}",identifier("atmaram".as_bytes()));
    }
}