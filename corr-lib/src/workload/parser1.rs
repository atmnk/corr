use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{map, opt};
use nom::multi::separated_list0;
use nom::sequence::{delimited, tuple};
use num_traits::ToPrimitive;
use crate::core::parser::{positive_integer, string};
use crate::core::Variable;
use crate::journey::parser::parse_name;
use crate::parser::{Parsable, ParseResult, ws};
use crate::workload::{ WorkLoad};

impl Parsable for WorkLoad {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
            parse_name,
            ws(tag("(")),separated_list0(ws(tag(",")),Variable::parser),ws(tag(")")),ws(char('{')),
            opt(delimited(tuple((ws(tag("startup")),ws(tag(":")))),ws(string), ws(tag(",")))),
                   delimited(tuple((ws(tag("journey")),ws(tag(":")))),ws(string), ws(tag(","))),
                   delimited(tuple((ws(tag("users")),ws(tag(":")))),ws(positive_integer), ws(tag(","))),
            delimited(tuple((ws(tag("perUserRampUp")),ws(tag(":")))),ws(positive_integer), ws(tag(","))),
            delimited(tuple((ws(tag("duration")),ws(tag(":")))),ws(positive_integer),ws( tag("}"))),
                   )),|(name,_,_,_,_,
                                       startup_journey,
            journey,
            users,
            rampUp,
            duration
                                   )|WorkLoad{
            startup_journey,
            concurrentUsers:users.to_usize().unwrap(),
            perUserRampUp:rampUp.to_u64().unwrap(),
            journey,
            duration,
            name
        })(input)
    }
}
