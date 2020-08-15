use crate::parser::{ParseResult, ws, non_back_quote, identifier};
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
        identifier
    ))(input)

}

