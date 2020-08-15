use crate::parser::{Parsable, ParseResult, boolean, identifier, ws};
use crate::core::{Value, Variable, DataType};
use nom::combinator::{map, opt};
use nom::sequence::{tuple, preceded};
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::branch::alt;

impl Parsable for Value {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(boolean,|val|Value::Boolean(val))(input)
    }
}
impl Parsable for Variable {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((identifier,opt(preceded(ws(char(':')),DataType::parser)))),|(name,data_type)| Variable{name,data_type})(input)
    }
}
impl Parsable for DataType {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(tag("String"),|_|DataType::String),
            map(tag("Boolean"),|_|DataType::Boolean),
            map(tag("PositiveInteger"),|_|DataType::PositiveInteger),
            map(tag("Integer"),|_|DataType::Integer),
            map(tag("Double"),|_|DataType::Double),
            map(tag("Object"),|_|DataType::Object),
            map(tag("List"),|_|DataType::List),
            ))(input)
    }
}
