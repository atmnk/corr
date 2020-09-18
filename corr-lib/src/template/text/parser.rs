use nom::bytes::complete::{ is_not, escaped_transform, tag};
use nom::character::complete::{char};
use nom::combinator::{map};
use nom::sequence::{tuple, terminated, preceded};
use std::str;
use nom::branch::alt;
use nom::multi::{many0};
use crate::template::text::{Text, Block, Scriplet};
use crate::parser::{Parsable, ParseResult, ws};
use crate::template::{Expression};

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
    use crate::template::text::{Text, Block, Scriplet};
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    use crate::template::Expression;
    use crate::core::DataType;
    use crate::template::text::parser::text_block;

    #[test]
    fn should_parse_text_with_scriptlet_and_text(){
        let text=r#"fillable text `Hello <%name%>`"#;
        let a=Text::parser(text);
        assert_if(text,a,Text{
            blocks:vec![
                Block::Text("Hello ".to_string()),
                Block::Scriplet(Scriplet::Expression(Expression::Variable("name".to_string(),Option::None)))
            ]
        })
    }
    #[test]
    fn should_parse_text_with_multiple_scriptlet_and_text(){
        let text=r#"fillable text `Hello <%i%>-<%name%>`"#;
        let a=Text::parser(text);
        assert_if(text,a,Text{
            blocks:vec![
                Block::Text("Hello ".to_string()),
                Block::Scriplet(Scriplet::Expression(Expression::Variable("i".to_string(),Option::None))),
                Block::Text("-".to_string()),
                Block::Scriplet(Scriplet::Expression(Expression::Variable("name".to_string(),Option::None)))
            ]
        })
    }
    #[test]
    fn should_parse_text_block_with_escaped_back_tick(){
        let text=r#"Atmaram\`Hello"#;
        let a=text_block(text);
        assert_if(text,a,r#"Atmaram`Hello"#.to_string())
    }
    #[test]
    fn should_parse_text_block_of_text(){
        let text=r#"Atmaram"#;
        let a=Block::parser(text);
        assert_if(text,a,Block::Text(r#"Atmaram"#.to_string()))
    }
    #[test]
    fn should_parse_scriplet_block_of_text(){
        let text=r#"<%name%>"#;
        let a=Block::parser(text);
        assert_if(text,a,Block::Scriplet(Scriplet::Expression(Expression::Variable("name".to_string(),Option::None))))
    }

    #[test]
    fn should_parse_scriplet_with_expression(){
        let text=r#"<%name:String%>"#;
        let a=Scriplet::parser(text);
        assert_if(text,a,Scriplet::Expression(Expression::Variable("name".to_string(),Option::Some(DataType::String))))
    }
    // use crate::parser::util::assert_if;

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