use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::tuple;
use num_traits::ToPrimitive;
use crate::core::parser::{positive_integer, string};
use crate::parser::{Parsable, ParseResult, ws};
use crate::workload::{ WorkLoad};

impl Parsable for WorkLoad {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(string),ws(tag("{")),
                   ws(tag("journey")),ws(tag(":")),ws(string), ws(tag(",")),
                   ws(tag("users")),ws(tag(":")),ws(positive_integer), ws(tag(",")),
                   ws(tag("perUserRampUp")),ws(tag(":")),ws(positive_integer), ws(tag(",")),
                   ws(tag("duration")),ws(tag(":")),ws(positive_integer),ws( tag("}")),
                   )),|(name,_,
            _,_,journey,_,
            _,_,users,_,
            _,_,rampUp,_,
            _,_,duration,_
                                   )|WorkLoad{
            concurrentUsers:users.to_usize().unwrap(),
            perUserRampUp:rampUp.to_u64().unwrap(),
            journey,
            duration,
            name
        })(input)
    }
}