use nom::bytes::complete::{ is_not, escaped_transform, tag};
use nom::character::complete::{char};
use nom::combinator::{map, opt};
use nom::sequence::{tuple, terminated, preceded};
use std::str;
use nom::branch::alt;
use nom::multi::{many0};
use crate::template::text::{Text, Block, LoopBlock};
use crate::parser::{Parsable, ParseResult, ws, identifier, sp};
use crate::template::{ExpressionBlock};
impl Parsable for LoopBlock{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        preceded(tag("<%"),terminated(ws(text_loop),tag("%>")))(input)
    }
}
fn text_loop<'a>(input: &'a str) -> ParseResult<'a, LoopBlock> {
    map(tuple((
            ws(tag("for")),
            char('('),
            ws(identifier),
            tag("in"),
            ws(identifier),
            ws(char(')')),
            ws(char('{')),
            map(opt(alt((
                map(text_loop,|lb| vec![Block::Loop(lb)]),
                preceded(tag("%>"),terminated(Vec::<Block>::parser,tag("<%")))
                ))),|o_vec|if let Some(val)=o_vec{
                val
            } else {
                vec![]
            }),
            ws(char('}')),
        )),|(_,_,with,_,on,_,_,inner,_)|LoopBlock {
        on,
        with,
        inner
    })(input)
}
impl Parsable for Block{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(LoopBlock::parser,|val|Block::Loop(val)),
            map(ExpressionBlock::parser, |val| Block::Expression(val)),
            map(text_block,|val|Block::Final(val))
        ))(input)
    }
}

impl Parsable for Vec<Block> {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        many0(Block::parser)(input)
    }
}
impl Parsable for Text{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            terminated(preceded(tuple((tag("@text"),sp,char('`'))),Vec::<Block>::parser),char('`')),
            |blocks| Text{blocks}
        )(input)
    }
}
pub fn text_block<'a>(input:&'a str) ->ParseResult<'a,String>{
    map(escaped_transform(is_not(r#"\{<`"#), '\\', |i: &'a str| alt((tag("{"),tag("<"),tag("\\"),tag("`")))(i)),|val| val.to_string())(input)
}

#[cfg(test)]
mod tests{
    use crate::template::text::{Text, Block, LoopBlock};
    use crate::parser::{Parsable, ParseResult};
    use crate::template::text::parser::{text_block, text_loop};
    use nom::error::convert_error;

    #[test]
    fn should_parse_text(){
        let text=r#"@text `Hello`"#;
        let a=Text::parser(text);
        assert_if(text,a,Text{
            blocks:vec![Block::Final("Hello".to_string())]
        })
    }
    #[test]
    fn should_parse_text_with_loop_block(){
        let text = r#"@text `<% for (a in b){%>Hello<%}%>`"#;
        let a=Text::parser(text);
        assert_if(text,a,Text{blocks:vec![
            Block::Loop(LoopBlock{
                on:"b".to_string(),
                with:"a".to_string(),
                inner:vec![Block::Final("Hello".to_string())]
            })
        ]})
    }
    #[test]
    fn should_parse_loop_block(){
        let text = r#"<% for (a in b){%>Hello<%}%>"#;
        let a=Block::parser(text);
        assert_if(text,a,Block::Loop(LoopBlock{
            on:"b".to_string(),
            with:"a".to_string(),
            inner:vec![Block::Final("Hello".to_string())]
        }))
    }
    #[test]
    fn should_parse_text_loop(){
        let text = r#"for (a in b){}"#;
        let a=text_loop(text);
        assert_if(text,a,LoopBlock{
            on:"b".to_string(),
            with:"a".to_string(),
            inner:vec![]
        })
    }
    #[test]
    fn should_parse_text_block(){
        let (_i,a)=text_block(r#"Hello`"#).unwrap();
        assert_eq!(a,"Hello".to_string())
    }
    #[test]
    fn should_parse_block_final(){
        let (_i,a)=Block::parser(r#"Hello`"#).unwrap();
        assert_eq!(a,Block::Final("Hello".to_string()))
    }
    #[test]
    fn should_parse_vec_block(){
        let (_i,a)=Vec::<Block>::parser(r#"Hello`"#).unwrap();
        assert_eq!(a,vec![Block::Final("Hello".to_string())])
    }
    pub fn assert_if<'a,T>(text:&'a str,result:ParseResult<'a,T>,to:T) where T:PartialEq+std::fmt::Debug{
        match result {
            Ok((_i,val))=>{
                assert_eq!(val,to)
            },
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e))=>{
                assert!(false,format!("Unable to parse following errors {}",convert_error(text,e)))
            },
            _=>{}
        }
    }
}