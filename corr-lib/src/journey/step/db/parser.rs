use crate::parser::{Parsable, ParseResult, ws};
use crate::journey::step::db::{DefineConnectionStep, ExecuteStep};
use nom::sequence::tuple;
use nom::bytes::complete::tag;
use crate::template::{VariableReferenceName, Expression};
use nom::combinator::{map, opt};

impl Parsable for DefineConnectionStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("let")),
               ws(VariableReferenceName::parser),
               ws(tag("=")),
               ws(tag("connect")),
               ws(tag("postgres")),
               ws(Expression::parser)
        )),|(_,connection_name,_,_,_,connection_string)|DefineConnectionStep{
            connection_string,
            connection_name
        })(input)
    }
}

impl Parsable for ExecuteStep {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("on")),
                   ws(VariableReferenceName::parser),
                   ws(tag("execute")),
                   ws(Expression::parser),
                   ws(tag("with")),
                   opt(ws(tag("multiple"))),
                   ws(Expression::parser)
        )),|(_,connection_name,_,query,_,is_multiple,value)| ExecuteStep {
            value,
            query,
            is_single:is_multiple.map(|_|false).unwrap_or(true),
            connection_name
        })(input)
    }
}