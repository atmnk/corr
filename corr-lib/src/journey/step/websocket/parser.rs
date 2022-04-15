use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, preceded, tuple};
use crate::journey::step::Step;
use crate::journey::step::websocket::{OnMessage, WebSocketServerStep, WebSocketStep};
use crate::parser::{Parsable, ParseResult, ws};
use crate::template::Expression;
use crate::template::object::extractable::ExtractableObject;


impl Parsable for WebSocketServerStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(preceded(tuple((ws(tag("websocket")),ws(tag("server")))),
            tuple((ws(Expression::parser), ws(tag("on")),ws(tag("message")),ws(tag("with")),ws(ExtractableObject::parser),delimited(
                ws(tag("{")),many0(ws(WebSocketStep::parser)),ws(tag("}"))
            )))
        ),|(port,_,_,_,extract,steps)|WebSocketServerStep{port,on_message:OnMessage{
            extract,
            steps
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