use crate::parser::{Parsable, ParseResult, ws};
use crate::template::json::extractable::{EJson, EPair};
use nom::branch::alt;
use nom::combinator::map;
use nom::sequence::{preceded, terminated, tuple};
use nom::bytes::complete::tag;
use crate::core::Variable;
use nom::multi::separated_list0;
use crate::core::parser::string;
use nom::character::complete::char;

impl Parsable for EPair{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            tuple((string,ws(char(':')),EJson::parser)),|(key,_,value)|{
                EPair{
                    key,
                    value
                }
            }
        )(input)
    }
}
impl Parsable for EJson{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(preceded(ws(tag("{{")),terminated(Variable::parser,ws(tag("}}")))),|val|{EJson::Variable(val)}),
            map(
                preceded(ws(char('[')),terminated(separated_list0(ws(char(',')),EJson::parser),ws(char(']')))),
                |val|{EJson::StaticArray(val)}
            ),
            map(
                tuple((
                    ws(tag("<%")),
                    ws(tag("for")),
                    ws(char('(')),
                    ws(Variable::parser),
                    ws(tag("in")),
                    ws(Variable::parser),
                    ws(char(')')),
                    ws(char('{')),
                    ws(tag("%>")),
                    ws(EJson::parser),
                    ws(tag("<%")),
                    ws(char('}')),
                    ws(tag("%>")),
                )),|(_,_,_,with,_,on,_,_,_,json,_,_,_)|{
                    EJson::DynamicArray(with,on,Box::new(json))
                }
            ),
            map(preceded(ws(char('{')),terminated(separated_list0(
                char(','),EPair::parser
            ),ws(char('}')))),|val|{EJson::Object(val)})
        ))(input)
    }
}