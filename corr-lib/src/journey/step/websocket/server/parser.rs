use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::{many0};
use nom::sequence::{delimited, preceded, tuple};
use crate::journey::step::Step;
use crate::journey::step::websocket::server::{WebSocketServerHook, WebSocketServerStep, WebSocketStep};
use crate::parser::{Parsable, ParseResult, ws};
use crate::template::{Expression, VariableReferenceName};
use crate::template::object::extractable::ExtractableObject;


impl Parsable for WebSocketServerStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(preceded(tuple((ws(tag("websocket")),ws(tag("server")))),
            tuple((ws(Expression::parser),ws(tag("with")),ws(tag("listener")),ws(VariableReferenceName::parser), ws(tag("=>")),
                   delimited(
                ws(tag("{")),many0(ws(WebSocketStep::parser)),ws(tag("}"))
            )
            ))
        ),|(port,_,_,variable,_,block)|WebSocketServerStep{port,hook:WebSocketServerHook{
            variable,
            block
        }})(input)
    }
}

impl Parsable for WebSocketStep {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(preceded(tuple((ws(tag("send")),ws(tag("to")),ws(tag("client")))),ws(Expression::parser)),|exp|WebSocketStep::SendStep(exp)),
            map(ws(Step::parser),|step|WebSocketStep::NormalStep(step))
            ))(input)
    }
}