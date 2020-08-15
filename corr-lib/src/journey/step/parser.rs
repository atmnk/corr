use crate::journey::step::Step;
use nom::combinator::map;
use crate::parser::ParseResult;
use crate::journey::step::system::SystemStep;
use crate::parser::Parsable;
impl Parsable for Step{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(SystemStep::parser,Step::System)(input)
    }
}