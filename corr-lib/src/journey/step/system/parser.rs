use crate::parser::{Parsable, ws};
use crate::journey::step::system::{SystemStep, PrintStep};
use crate::parser::ParseResult;
use nom::combinator::{map};
use nom::sequence::{preceded, terminated};
use nom::bytes::complete::tag;
use crate::template::text::{Text};
use nom::character::complete::char;

impl Parsable for PrintStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        preceded(ws(tag("print")),map(terminated(Text::parser,char(';')),|txt|{PrintStep::WithText(txt)}))(input)
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

    #[tokio::test]
    async fn should_parse_printstep(){
        let j= r#"print fillable text `Hello`;"#;
        assert_if(j
                  ,PrintStep::parser(j)
                  ,PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]}))

    }
    #[tokio::test]
    async fn should_parse_systemstep_with_printstep(){
        let j= r#"print fillable text `Hello`;"#;
        assert_if(j
                  ,SystemStep::parser(j)
                  ,SystemStep::Print(PrintStep::WithText(Text{blocks:vec![Block::Text("Hello".to_string())]})))

    }
}
