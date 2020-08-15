use crate::parser::{Parsable, ParseResult, identifier, ws};
use crate::template::{Expression, ExpressionBlock};
use nom::combinator::map;
use crate::core::{Value, Variable};
use nom::sequence::{preceded, terminated, tuple};
use nom::bytes::complete::tag;
use nom::branch::alt;
use nom::character::complete::char;
use nom::multi::{separated_list0};

impl Parsable for Expression{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(Value::parser,|val|Expression::Constant(val)),
            map(tuple((identifier,ws(char('(')),separated_list0(ws(char(',')),Expression::parser),ws(char(')')))),|(name,_,expressions,_)|Expression::Function(name,expressions)),
            map(Variable::parser,|val|Expression::Variable(val.name,val.data_type)),

            ))(input)
    }
}
impl Parsable for ExpressionBlock{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(preceded(tag("{{"),terminated(Expression::parser,tag("}}"))),|expression|ExpressionBlock{
            expression
        })(input)
    }
}