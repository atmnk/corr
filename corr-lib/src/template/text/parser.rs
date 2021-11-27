use nom::bytes::complete::{ is_not, escaped_transform, tag};
use nom::character::complete::{char};
use nom::combinator::{map, opt};
use nom::sequence::{tuple, terminated, preceded, delimited};
use std::str;
use nom::branch::alt;
use nom::multi::{many0};
use crate::template::text::{Text, Block, Scriplet, TextForLoop, TextLoopInnerTemplate};
use crate::parser::{Parsable, ParseResult, ws};
use crate::template::{Expression, VariableReferenceName};
fn scriptlet_contents<'a>(input: &'a str) -> ParseResult<'a, Scriplet> {
    alt((
        map(TextForLoop::parser, |val| { Scriplet::ForLoop(val) }),
        map(Expression::parser, |val| { Scriplet::Expression(val) })
    ))(input)
}
impl Parsable for Scriplet{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            delimited(tag("${"),ws(scriptlet_contents),tag("}$")),
            delimited(tag("<%"),ws(scriptlet_contents),tag("%>")),
            ))(input)
    }
}
impl Parsable for TextForLoop{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((text_for_loop_left_part,
            text_for_loop_right_part)),|(on,(with,index,inner))|TextForLoop::WithVariableReference(on,with,index,Box::new(inner)))(input)
    }
}
impl Parsable for TextLoopInnerTemplate{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(TextForLoop::parser,|val|TextLoopInnerTemplate::ForLoop(val)),
            map(Expression::parser,|val|TextLoopInnerTemplate::Expression(val)),
            map(alt((preceded(tag("%>"),terminated(many0(Block::parser),tag("<%"))),
                     preceded(tag("${"),terminated(many0(Block::parser),tag("}$")))
            )),|val|TextLoopInnerTemplate::Blocks(val)),
            ))(input)
    }
}
fn text_for_loop_left_part<'a>(input: &'a str) -> ParseResult<'a, VariableReferenceName> {
    terminated(VariableReferenceName::parser,tuple((ws(char('.')),ws(tag("for")))))(input)
}
fn text_for_loop_right_part<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>,Option<VariableReferenceName>,TextLoopInnerTemplate)> {
    alt((
        arged_text_for_loop,
        unarged_text_for_loop
        ))(input)
}
fn unarged_text_for_loop<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>,Option<VariableReferenceName>,TextLoopInnerTemplate)> {
    map(
        TextLoopInnerTemplate::parser,|tlit|{(Option::None,Option::None,tlit)}
    )(input)
}
fn arged_text_for_loop<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>,Option<VariableReferenceName>,TextLoopInnerTemplate)> {
    map(
        tuple((
            preceded(ws(char('(')),terminated(
                    opt(
                        tuple((
                            VariableReferenceName::parser,
                            opt(preceded(ws(char(',')),VariableReferenceName::parser))
                            ))
                    )
                 ,ws(char(')')))),
            ws(tag("=>")),
            ws(TextLoopInnerTemplate::parser)
        )),
        |(opt_refs,_,tlit)|{
            let mut with = Option::None;
            let mut index = Option::None;
            if let Some(refs) = opt_refs {
                with = Option::Some(refs.0);
                index = refs.1;
            }
            (with,index,tlit)
        }
    )(input)
}
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
            terminated(preceded(tuple((ws(tag("text")),char('`'))),Vec::<Block>::parser),char('`')),
            |blocks| Text{blocks}
        )(input)
    }
}
pub fn text_block<'a>(input:&'a str) ->ParseResult<'a,String>{
    map(escaped_transform(is_not(r#"\<`$"#), '\\', |i: &'a str| alt((tag("$"),tag("<"),tag("\\"),tag("`")))(i)),|val| val.to_string())(input)
}

#[cfg(test)]
mod tests{
    use crate::template::text::{Text, Block, Scriplet, TextLoopInnerTemplate, TextForLoop};
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    use crate::template::{Expression, VariableReferenceName, Operator, BinaryOperator};
    use crate::core::DataType;
    use crate::template::text::parser::{text_block, text_for_loop_left_part, text_for_loop_right_part, unarged_text_for_loop, arged_text_for_loop};

    #[test]
    fn should_parse_text_for_loop_left_part(){
        let text=r#"atmaram.naik.for"#;
        let a=text_for_loop_left_part(text);
        assert_if(text,a,VariableReferenceName::from("atmaram.naik"))
    }

    #[test]
    fn should_parse_text_for_loop_right_part_when_unarged_for(){
        let text=r#"%>Atmaram<%"#;
        let a=text_for_loop_right_part(text);
        assert_if(text,a,(Option::None,Option::None,TextLoopInnerTemplate::Blocks(vec![Block::Text(format!("Atmaram"))])))
    }
    #[test]
    fn should_parse_text_for_loop_right_part_when_arged_for(){
        let text=r#" ( name , index )=>name"#;
        let a=text_for_loop_right_part(text);
        assert_if(text,a,(Option::Some(VariableReferenceName::from("name")),Option::Some(VariableReferenceName::from("index")),TextLoopInnerTemplate::Expression(Expression::Variable(format!("name"),Option::None))))
    }
    #[test]
    fn should_parse_unarged_text_for_loop(){
        let text=r#"%>Atmaram<%"#;
        let a=unarged_text_for_loop(text);
        assert_if(text,a,(Option::None,Option::None,TextLoopInnerTemplate::Blocks(vec![Block::Text(format!("Atmaram"))])))
    }
    #[test]
    fn should_parse_arged_text_for_loop_without_variables(){
        let text=r#"()=>%>Atmaram<%"#;
        let a=arged_text_for_loop(text);
        assert_if(text,a,(Option::None,Option::None,TextLoopInnerTemplate::Blocks(vec![Block::Text(format!("Atmaram"))])))
    }
    #[test]
    fn should_parse_arged_text_for_loop_with_loop_variable(){
        let text=r#"(name)=>%>Atmaram<%"#;
        let a=arged_text_for_loop(text);
        assert_if(text,a,(Option::Some(VariableReferenceName::from("name")),Option::None,TextLoopInnerTemplate::Blocks(vec![Block::Text(format!("Atmaram"))])))
    }
    #[test]
    fn should_parse_arged_text_for_loop_with_loop_variable_and_index(){
        let text=r#"(name,index)=>%>Atmaram<%"#;
        let a=arged_text_for_loop(text);
        assert_if(text,a,(Option::Some(VariableReferenceName::from("name")),Option::Some(VariableReferenceName::from("index")),TextLoopInnerTemplate::Blocks(vec![Block::Text(format!("Atmaram"))])))
    }

    #[test]
    fn should_parse_scriplet_with_binary_operator(){
        let txt=r#"<% a + b %>"#;
        assert_if(txt,Scriplet::parser(txt),Scriplet::Expression(
            Expression::Operator(Operator::Binary(BinaryOperator::Add),vec![
                Expression::Variable("a".to_string(),Option::None),
                Expression::Variable("b".to_string(),Option::None),
            ])
        ))

    }

    #[test]
    fn should_parse_scriplet_with_unary(){
        let text=r#"<% b++ %>"#;
        Scriplet::parser(text).unwrap();

    }

    #[test]
    fn should_parse_scriplet_with_expression(){
        let text=r#"<%name:String%>"#;
        let a=Scriplet::parser(text);
        assert_if(text,a,Scriplet::Expression(Expression::Variable("name".to_string(),Option::Some(DataType::String))))
    }

    #[test]
    fn should_parse_scriplet_with_for(){
        let text=r#"<%names.for(name)=>name%>"#;
        let a=Scriplet::parser(text);
        assert_if(text,a,Scriplet::ForLoop(TextForLoop::WithVariableReference(VariableReferenceName::from("names"),Option::Some(VariableReferenceName::from("name")),Option::None,Box::new(TextLoopInnerTemplate::Expression(Expression::Variable(format!("name"),Option::None))))))
    }

    #[test]
    fn should_parse_textforloop(){
        let text=r#"names.for %>Hello World<%"#;
        let a=TextForLoop::parser(text);
        assert_if(text,a,TextForLoop::WithVariableReference(VariableReferenceName::from("names"),Option::None,Option::None,Box::new(TextLoopInnerTemplate::Blocks(vec![Block::Text(format!("Hello World"))]))))
    }

    #[test]
    fn should_parse_textloopinnertemplate_with_for_loop(){
        let text=r#"names.for%>Atmaram<%"#;
        let a=TextLoopInnerTemplate::parser(text);
        assert_if(text,a,TextLoopInnerTemplate::ForLoop(
            TextForLoop::WithVariableReference(
                VariableReferenceName::from("names"),
                Option::None,
                Option::None,
                Box::new(TextLoopInnerTemplate::Blocks(vec![Block::Text(format!("Atmaram"))]))
            )))

    }

    #[test]
    fn should_parse_textloopinnertemplate_with_expression(){
        let text=r#"name"#;
        let a=TextLoopInnerTemplate::parser(text);
        assert_if(text,a,TextLoopInnerTemplate::Expression(Expression::Variable(format!("name"),Option::None)))

    }

    #[test]
    fn should_parse_textloopinnertemplate_with_text_block(){
        let text=r#"%>Atmaram<%"#;
        let a=TextLoopInnerTemplate::parser(text);
        assert_if(text,a,TextLoopInnerTemplate::Blocks(vec![Block::Text(format!("Atmaram"))]))

    }

    #[test]
    fn should_parse_text_with_scriptlet_and_text(){
        let text=r#"text `Hello <%name%>`"#;
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
        let text=r#"text `Hello <%i%>-<%name%>`"#;
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
}