use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::tuple;
use crate::journey::step::websocket::client::{WebSocketClientConnectStep, WebSocketSendStep};
use crate::parser::{Parsable, ParseResult, ws};
use crate::template::Expression;

impl Parsable  for WebSocketClientConnectStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple(
            (
                ws(tag("connect")),
                ws(tag("websocket")),
                ws(tag("named")),
                ws(Expression::parser),
                ws(tag("with")),
                ws(tag("url")),
                ws(Expression::parser))),|(_,_,_,connection_name,_,_,url)|{WebSocketClientConnectStep{url,connection_name}})(input)
    }
}

impl Parsable for WebSocketSendStep {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple(
            (
                ws(tag("send")),
                ws(Expression::parser),
                ws(tag("on")),
                ws(tag("websocket")),
                ws(tag("named")),
                ws(Expression::parser))),|(_,message,_,_,_,name)|{WebSocketSendStep{name,message}})(input)
    }
}