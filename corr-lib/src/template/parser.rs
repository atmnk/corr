use crate::parser::{Parsable, ParseResult, ws, identifier_part, scriptlet_keyword};
use crate::template::{Expression, VariableReferenceName};
use nom::combinator::{map, verify};
use crate::core::{Value, Variable};
use nom::sequence::{preceded, terminated, tuple};
use nom::bytes::complete::tag;
use nom::branch::alt;
use nom::character::complete::char;
use nom::multi::{separated_list0, separated_list1};
use crate::get_scriptlet_keywords;

impl Parsable for Expression{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(Value::parser,|val|Expression::Constant(val)),
            map(tuple((scriptlet_keyword,ws(char('(')),separated_list0(ws(char(',')),Expression::parser),ws(char(')')))),|(name,_,expressions,_)|Expression::Function(name.to_string(),expressions)),
            map(Variable::parser,|val|Expression::Variable(val.name,val.data_type)),
            ))(input)
    }
}
impl Parsable for VariableReferenceName {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            separated_list1(char
                                ('.'),
                            map(identifier_part,|val|{val.to_string()})),|parts| { VariableReferenceName {parts}})(input)
    }
}