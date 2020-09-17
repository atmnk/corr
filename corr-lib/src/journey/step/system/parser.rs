use crate::parser::{Parsable, ws};
use crate::journey::step::system::{SystemStep, PrintStep};
use crate::parser::ParseResult;
use nom::combinator::{map, opt};
use nom::sequence::{tuple, preceded};
use nom::bytes::complete::tag;
use crate::template::text::{Text};
use nom::character::complete::char;
use nom::branch::alt;
use crate::core::Variable;
use nom::multi::many0;
use crate::journey::step::Step;
use crate::template::Expression;
use crate::journey::parser::steps;
impl Parsable for PrintStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        preceded(ws(tag("print")),map(Text::parser,|txt|{PrintStep::WithText(txt)}))(input)
    }
}
impl Parsable for SystemStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        // alt((
            map(PrintStep::parser,|pt|{SystemStep::Print(pt)})
            // map(tuple((ws(tag("let ")),ws(identifier),ws(char('=')),Expression::parser)),|(_,var,_,expr)|{SystemStep::Assign(var,expr)}),
            // map(tuple((
            //     ws(tag("for")),
            //     ws(char('(')),
            //     ws(Variable::parser),
            //     ws(tag("in")),
            //     ws(Variable::parser),
            //     opt(preceded(ws(char(',')),Variable::parser)),
            //     ws(char(')')),
            //     ws(char('{')),
            //     steps,
            //     ws(char('}'))
            // )),|(_,_,with_var,_,in_var,index_var,_,_,steps,_)|{SystemStep::For(with_var,in_var,Box::new(Step::System(SystemStep::Collection(steps))),index_var)}))
        (input)
    }
}
#[cfg(test)]
mod tests{
    use crate::journey::step::system::{SystemStep, PrintStep};
    use crate::parser::Parsable;
    use crate::template::text::{Text, Block};
    use crate::parser::util::assert_if;
    use crate::core::{Variable, Value};
    use crate::journey::step::Step;
    use crate::template::Expression;
    use nom::lib::std::collections::HashMap;

    #[tokio::test]
    async fn should_parse_print_step(){
        let j= r#"print fillable text `Hello`"#;
        assert_if(j
                  ,SystemStep::parser(j)
                  ,SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))

    }
    // #[tokio::test]
    // async fn should_parse_for_step(){
    //     let j= r#"for (a in b) { print(`Hello`);}"#;
    //     assert_if(j
    //               ,SystemStep::parser(j)
    //               ,SystemStep::For(
    //             Variable::new("a"),
    //             Variable::new("b"),
    //             Box::new(Step::System(SystemStep::Collection(vec![
    //                 Step::System(SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))
    //             ]))),
    //             Option::None
    //             ))
    //
    // }
    // #[tokio::test]
    // async fn should_parse_let_step(){
    //     let j= r#"let a = array![
    //     object! {
    //         "name":"Clothes"
    //     },
    //     object! {
    //         "name":"Electronics"
    //     }
    // ];"#;
    //     let mut hm1 = HashMap::new();
    //     hm1.insert(format!("name"),Value::String(format!("Clothes")));
    //
    //     let mut hm2 = HashMap::new();
    //     hm2.insert(format!("name"),Value::String(format!("Electronics")));
    //     assert_if(j
    //               ,SystemStep::parser(j)
    //               ,SystemStep::Assign(format!("a"),Expression::Constant(Value::Array(vec![
    //             Value::Map(hm1),Value::Map(hm2)
    //         ]))));
    //
    // }
}
