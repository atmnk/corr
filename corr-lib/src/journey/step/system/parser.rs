use crate::parser::{Parsable, ws};
use crate::journey::step::system::SystemStep;
use crate::parser::ParseResult;
use nom::combinator::map;
use nom::sequence::tuple;
use nom::bytes::complete::tag;
use crate::template::text::{Text};
use nom::character::complete::char;

impl Parsable for SystemStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((tag("print("),ws(Text::parser),char(')'))),|(_,txt,_)|{SystemStep::Print(txt)})(input)
    }
}