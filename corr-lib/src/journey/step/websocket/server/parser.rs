
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::multi::{many0};
use nom::sequence::{delimited, preceded, tuple};
use crate::journey::step::Step;
use crate::journey::step::websocket::server::{WebSocketServerSendToClient, WebSocketServerHook, WebSocketServerStep};
use crate::parser::{Parsable, ParseResult, ws};
use crate::template::{Expression, VariableReferenceName};



impl Parsable for WebSocketServerStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(preceded(tuple((ws(tag("websocket")),ws(tag("server")))),
            tuple((ws(Expression::parser),ws(tag("with")),ws(tag("listener")),ws(VariableReferenceName::parser), ws(tag("=>")),
                   delimited(
                ws(tag("{")),many0(ws(Step::parser)),ws(tag("}"))
            )
            ))
        ),|(port,_,_,variable,_,block)|WebSocketServerStep{port,hook:WebSocketServerHook{
            variable,
            block
        }})(input)
    }
}
impl Parsable for WebSocketServerSendToClient {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple(
            (
                ws(tag("to")),
                ws(tag("websocket")),
                ws(tag("client")),
                ws(Expression::parser),
                ws(tag("send")),
                opt(ws(tag("binary"))),
                ws(Expression::parser))),|(_,_,_,id,_,ib,message)|{ WebSocketServerSendToClient {id,is_binary:ib.map(|_|true).unwrap_or(false),message}})(input)
    }
}