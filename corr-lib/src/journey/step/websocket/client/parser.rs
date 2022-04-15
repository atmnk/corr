use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::multi::many1;
use nom::sequence::tuple;
use crate::journey::step::Step;
use crate::journey::step::websocket::client::{WebSocketClientConnectStep, WebSocketHook, WebSocketSendStep};
use crate::parser::{Parsable, ParseResult, ws};
use crate::template::{Expression, VariableReferenceName};

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
                ws(Expression::parser),
                ws(tag("and")),
                ws(tag("listener")),
                ws(VariableReferenceName::parser),
                ws(tag("=>")),
                ws(tag("{")),
                many1(ws(Step::parser)),
                ws(tag("}"))
            )),|(_,_,_,connection_name,_,_,url,_,_,variable,_,_,block,_)|{WebSocketClientConnectStep{url,connection_name,
        hook:WebSocketHook{
            block,
            variable
        }
        }})(input)
    }
}

impl Parsable for WebSocketSendStep {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple(
            (
                ws(tag("send")),
                opt(ws(tag("binary"))),
                ws(Expression::parser),
                ws(tag("on")),
                ws(tag("websocket")),
                ws(tag("named")),
                ws(Expression::parser))),|(_,ib,message,_,_,_,name)|{WebSocketSendStep{name,is_binary:ib.map(|_|true).unwrap_or(false),message}})(input)
    }
}