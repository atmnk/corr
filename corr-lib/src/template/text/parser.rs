use nom::bytes::complete::{ is_not, escaped_transform, tag};
use nom::character::complete::{char};
use nom::combinator::{map, opt};
use nom::sequence::{tuple, terminated, preceded};
use std::str;
use nom::branch::alt;
use nom::multi::{many0};
use crate::template::text::{Text, Block, Scriplet};
use crate::parser::{Parsable, ParseResult, ws, sp};
use crate::template::{Expression};
use crate::core::Variable;

impl Parsable for Scriplet{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        preceded(tag("<%"),terminated(map(ws(Expression::parser),|val|{Scriplet::Expression(val)}),tag("%>")))(input)
    }
}
// impl Parsable for LoopBlock{
//     fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
//         preceded(tag("<%"),terminated(ws(text_loop),tag("%>")))(input)
//     }
// }
// fn text_loop<'a>(input: &'a str) -> ParseResult<'a, LoopBlock> {
//     map(tuple((
//             ws(tag("for")),
//             char('('),
//             ws(Variable::parser),
//             tag("in"),
//             ws(Variable::parser),
//             opt(preceded(ws(char(',')),Variable::parser)),
//             ws(char(')')),
//             ws(char('{')),
//             map(opt(alt((
//                 map(text_loop,|lb| vec![Block::Loop(lb)]),
//                 preceded(tag("%>"),terminated(Vec::<Block>::parser,tag("<%")))
//                 ))),|o_vec|if let Some(val)=o_vec{
//                 val
//             } else {
//                 vec![]
//             }),
//             ws(char('}')),
//         )),|(_,_,with,_,on,index_var,_,_,inner,_)|LoopBlock {
//         on,
//         with,
//         inner,
//         index_var
//     })(input)
// }
impl Parsable for Block{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(text_block,|val|Block::Text(val)),
            map(Scriplet::parser, |val| Block::Scriplet(val))

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
            terminated(preceded(tuple((ws(tag("fillable")),ws(tag("text")),char('`'))),Vec::<Block>::parser),char('`')),
            |blocks| Text{blocks}
        )(input)
    }
}
pub fn text_block<'a>(input:&'a str) ->ParseResult<'a,String>{
    map(escaped_transform(is_not(r#"\<`"#), '\\', |i: &'a str| alt((tag("<"),tag("\\"),tag("`")))(i)),|val| val.to_string())(input)
}

#[cfg(test)]
mod tests{
    use crate::parser::util::assert_if;
    use crate::template::text::{Text, Block};
    use crate::parser::{Parsable, ParseResult};
    use crate::template::text::parser::{text_block};
    use nom::error::convert_error;
    use crate::core::Variable;

    // #[test]
    // fn should_parse_text(){
    //     let text=r#"`Hello`"#;
    //     let a=Text::parser(text);
    //     assert_if(text,a,Text{
    //         blocks:vec![Block::Final("Hello".to_string())]
    //     })
    // }
    // #[test]
    // fn should_parse_text_with_loop_block(){
    //     let text = r#"`<% for (a in b){%>Hello<%}%>`"#;
    //     let a=Text::parser(text);
    //     assert_if(text,a,Text{blocks:vec![
    //         Block::Loop(LoopBlock{
    //             on:Variable::new("b"),
    //             with:Variable::new("a"),
    //             inner:vec![Block::Final("Hello".to_string())],
    //             index_var:Option::None
    //         })
    //     ]})
    // }
    // #[test]
    // fn should_parse_loop_block(){
    //     let text = r#"<% for (a in b){%>Hello<%}%>"#;
    //     let a=Block::parser(text);
    //     assert_if(text,a,Block::Loop(LoopBlock{
    //         on:Variable::new("b"),
    //         with:Variable::new("a"),
    //         inner:vec![Block::Final("Hello".to_string())],
    //         index_var:Option::None
    //     }))
    // }
    // #[test]
    // fn should_parse_text_loop(){
    //     let text = r#"for (a in b){}"#;
    //     let a=text_loop(text);
    //     assert_if(text,a,LoopBlock{
    //         on:Variable::new("b"),
    //         with:Variable::new("a"),
    //         inner:vec![],
    //         index_var:Option::None
    //     })
    // }
    // #[test]
    // fn should_parse_text_block(){
    //     let (_i,a)=text_block(r#"Hello`"#).unwrap();
    //     assert_eq!(a,"Hello".to_string())
    // }
    // #[test]
    // fn should_parse_block_final(){
    //     let (_i,a)=Block::parser(r#"Hello`"#).unwrap();
    //     assert_eq!(a,Block::Final("Hello".to_string()))
    // }
    // #[test]
    // fn should_parse_vec_block(){
    //     let (_i,a)=Vec::<Block>::parser(r#"Hello`"#).unwrap();
    //     assert_eq!(a,vec![Block::Final("Hello".to_string())])
    // }

}