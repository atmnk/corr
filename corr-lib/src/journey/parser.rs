use crate::parser::{ParseResult, ws, non_back_quote, identifier_part};
use crate::journey::Journey;
use nom::branch::alt;
use nom::sequence::{terminated, preceded, tuple};
use nom::character::complete::{char};
use nom::bytes::complete::{tag};
use nom::combinator::map;
use nom::multi::{many0};
use crate::journey::step::Step;
use crate::parser::Parsable;
impl Parsable for Journey{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map( tuple((parse_name,tag("()"),ws(char('{')),steps,ws(char('}')))),|(name,_,_,steps,_)|{
            Journey{
                name,
                steps
            }
        })(input)
    }
}
pub fn steps<'a>(input:&'a str) ->ParseResult<'a,Vec<Step>>{
    many0(terminated(ws(Step::parser),char(';')))(input)
}

pub fn parse_name<'a>(input:&'a str) ->ParseResult<'a,String>{
    alt((
        terminated(preceded(char('`'),non_back_quote),char('`')),
        map(identifier_part,|val| val.to_string())
    ))(input)

}
#[cfg(test)]
mod tests{
    use crate::journey::step::system::SystemStep;
    use crate::parser::Parsable;
    use crate::template::text::{Text, Block};
    use crate::parser::util::{assert_no_error,assert_if};
    use crate::core::{Variable, Value};
    use crate::journey::step::Step;
    use crate::template::Expression;
    use nom::lib::std::collections::HashMap;
    use crate::journey::Journey;

    #[tokio::test]
    async fn should_parse_complex_journey(){
        let j= r#"`Hello World`(){
            print fillable text `Hello <%concat("Atmaram","Naik")%>`;
        }"#;
        assert_no_error(j
                        ,Journey::parser(j)
        )
    }
}

