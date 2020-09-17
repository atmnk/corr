use crate::journey::step::Step;
use nom::combinator::map;
use crate::parser::ParseResult;
use crate::journey::step::system::SystemStep;
use crate::parser::Parsable;
use nom::branch::alt;
use crate::journey::step::rest::RestStep;

impl Parsable for Step{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
                map(SystemStep::parser,Step::System),
                map(RestStep::parser,Step::Rest)
        ))(input)
    }
}