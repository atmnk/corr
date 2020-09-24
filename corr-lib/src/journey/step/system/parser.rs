use crate::parser::{Parsable, ws};
use crate::journey::step::system::{SystemStep, PrintStep, ForLoopStep};
use crate::parser::ParseResult;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, terminated, tuple};
use nom::bytes::complete::tag;
use crate::template::text::{Text};
use nom::character::complete::char;
use nom::branch::alt;
use crate::template::VariableReferenceName;
use crate::journey::step::Step;
use nom::multi::many0;

impl Parsable for PrintStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        preceded(ws(tag("print")),map(terminated(Text::parser,char(';')),|txt|{PrintStep::WithText(txt)}))(input)
    }
}
impl Parsable for ForLoopStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
            for_left_part,
            ws(for_right_part))), |(on,(with,index,steps))|{ForLoopStep::WithVariableReference(on, with, index, steps)})(input)
    }
}
fn for_left_part<'a>(input: &'a str) -> ParseResult<'a, VariableReferenceName>{
    map(tuple((
        ws(VariableReferenceName::parser),
        ws(char('.')),
        ws(tag("for"))
        )),|(vrn,_,_)|vrn)(input)
}
fn for_right_part<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>, Option<VariableReferenceName>, Vec<Step>)>{
    alt((
        arged_for_parser,
        unarged_for_parser

    ))(input)
}
fn unarged_for_parser<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>,Option<VariableReferenceName>,Vec<Step>)>{
    map(one_or_many_steps,|steps|{(Option::None,Option::None,steps)})(input)
}
fn one_or_many_steps<'a>(input: &'a str) -> ParseResult<'a, Vec<Step>>{
    alt((
        map(Step::parser,|step|{vec![step]}),
        preceded(ws(char('{')),terminated(many0(Step::parser),ws(char('}'))))
    ))(input)
}
fn arged_for_parser<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>,Option<VariableReferenceName>,Vec<Step>)>{
    map(tuple((
        preceded(
            ws(char('(')),
            terminated(
            opt(
                tuple((
                    VariableReferenceName::parser,
                    opt(preceded(ws(char(',')),VariableReferenceName::parser))))),ws(char(')')))),ws(tag("=>")),one_or_many_steps)),
        |(opt_vars,_,steps)|{
            let mut with = Option::None;
            let mut index= Option::None;
            if let Some(vars)=opt_vars{
                with = Option::Some(vars.0);
                index = vars.1
            }
            (with,index,steps)})(input)
}
impl Parsable for SystemStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(PrintStep::parser,|ps|{SystemStep::Print(ps)}),
            map(ForLoopStep::parser,|fls|{SystemStep::ForLoop(fls)})))
            // map(tuple((ws(tag("let ")),ws(identifier),ws(char('=')),Expression::parser)),|(_,var,_,expr)|{SystemStep::Assign(var,expr)}),
        (input)
    }
}
#[cfg(test)]
mod tests{
    use crate::journey::step::system::{SystemStep, PrintStep, ForLoopStep};
    use crate::parser::Parsable;
    use crate::template::text::{Text, Block};
    use crate::parser::util::assert_if;
    use crate::template::VariableReferenceName;
    use crate::journey::step::Step;
    use crate::journey::step::system::parser::{one_or_many_steps, unarged_for_parser, for_right_part, for_left_part, arged_for_parser};

    #[tokio::test]
    async fn should_parse_for_left_part(){
        let j= r#"atmaram.naik.for"#;
        assert_if(j
                  ,for_left_part(j)
                  ,VariableReferenceName::from("atmaram.naik"))

    }

    #[tokio::test]
    async fn should_parse_for_right_without_args(){
        let j= r#"print fillable text `Hello`;"#;
        assert_if(j
                  , for_right_part(j)
                  , (Option::None,Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))
            ]))

    }
    #[tokio::test]
    async fn should_parse_for_right_with_args(){
        let j= r#"(name,index)=>print fillable text `Hello`;"#;
        assert_if(j
                  , for_right_part(j)
                  , (Option::Some(VariableReferenceName::from("name")),Option::Some(VariableReferenceName::from("index")),vec![
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))
            ]))

    }

    #[tokio::test]
    async fn should_parse_unarged_for(){
        let j= r#"print fillable text `Hello`;"#;
        assert_if(j
                  ,unarged_for_parser(j)
                  ,(Option::None,Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))
            ]))

    }

    #[tokio::test]
    async fn should_parse_one_or_many_steps_when_one_step(){
        let j= r#"print fillable text `Hello`;"#;
        assert_if(j
                  ,one_or_many_steps(j)
                  ,vec![
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))
            ])

    }

    #[tokio::test]
    async fn should_parse_one_or_many_steps_when_multiple_step(){
        let j= r#"{ print fillable text `Hello`;
            print fillable text `Hello World`;
        }"#;
        assert_if(j
                  ,one_or_many_steps(j)
                  ,vec![
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]}))),
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello World".to_string())]})))
            ])

    }

    #[tokio::test]
    async fn should_parse_arged_for_without_variables(){
        let j= r#"()=>print fillable text `Hello`;"#;
        assert_if(j
                  ,arged_for_parser(j)
                  ,(Option::None,Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))
            ]))

    }

    #[tokio::test]
    async fn should_parse_arged_for_with_loop_variable(){
        let j= r#"(name)=>print fillable text `Hello`;"#;
        assert_if(j
                  ,arged_for_parser(j)
                  ,(Option::Some(VariableReferenceName{parts:vec!["name".to_string()]}),Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))
            ]))

    }
    #[tokio::test]
    async fn should_parse_arged_for_with_loop_variable_and_index_variable(){
        let j= r#"(name,index)=>print fillable text `Hello`;"#;
        assert_if(j
                  ,arged_for_parser(j)
                  ,(Option::Some(VariableReferenceName{parts:vec!["name".to_string()]}),Option::Some(VariableReferenceName{parts:vec!["index".to_string()]}),vec![
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))
            ]))

    }

    #[tokio::test]
    async fn should_parse_printstep(){
        let j= r#"print fillable text `Hello`;"#;
        assert_if(j
                  ,PrintStep::parser(j)
                  ,PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]}))

    }

    #[tokio::test]
    async fn should_parse_for_step(){
        let j= r#"atmaram.for print fillable text `Hello`;"#;
        assert_if(j
                  ,ForLoopStep::parser(j)
                  ,ForLoopStep::WithVariableReference(VariableReferenceName{ parts:vec![format!("atmaram")]},Option::None,Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))
            ]))

    }




    #[tokio::test]
    async fn should_parse_systemstep_with_printstep(){
        let j= r#"print fillable text `Hello`;"#;
        assert_if(j
                  ,SystemStep::parser(j)
                  ,SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))

    }
    #[tokio::test]
    async fn should_parse_systemstep_with_for_step(){
        let j= r#"atmaram.for print fillable text `Hello`;"#;
        assert_if(j
                  ,SystemStep::parser(j)
                  ,SystemStep::ForLoop(ForLoopStep::WithVariableReference(VariableReferenceName{ parts:vec![format!("atmaram")]},Option::None,Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))
            ])))

    }
}
