// use crate::parser::{Parsable, ws};
// use crate::parser::ParseResult;
// use nom::combinator::{map, opt};
// use nom::sequence::{tuple, preceded};
// use nom::bytes::complete::tag;
// use crate::template::text::{Text};
// use nom::character::complete::char;
// use crate::journey::step::rest::{RestStep, RestVerb,Body};
// use nom::branch::alt;
// use crate::template::json::Json;
// use crate::template::json::extractable::EJson;
// impl Parsable for RestVerb{
//     fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
//         alt((
//             map(tag("get"),|_|RestVerb::GET),
//             map(tag("post"),|_|RestVerb::POST),
//             map(tag("put"),|_|RestVerb::PUT),
//             map(tag("patch"),|_|RestVerb::PATCH),
//             map(tag("delete"),|_|RestVerb::DELETE),
//
//         ))(input)
//     }
// }
// impl Parsable for RestStep{
//     fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
//         map(tuple((
//             ws(RestVerb::parser),
//             ws(char('(')),
//             ws(Text::parser),
//             opt(preceded(ws(char(',')), ws(Body::parser))),
//             opt(preceded(ws(char(',')),EJson::parser)),
//             ws(char(')'))
//         )),|(verb,_,url,body,response,_)|{RestStep{
//             url,
//             verb,
//             body,
//             response
//         }})(input)
//
//     }
// }
// impl Parsable for Body{
//     fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
//         alt((
//             map(Text::parser,|text|{Body::Text(text)}),
//             map(Json::parser,|json|{Body::Json(json)}),
//         ))(input)
//     }
// }