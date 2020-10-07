use crate::parser::{Parsable, ParseResult, ws};
use crate::template::rest::extractable::{ExtractableResponse, ExtractableResponseBody, ExtractableResponseHeaders, ExtractableResponseHeaderPair, ExtractableResponseHeaderValue};
use nom::branch::alt;
use nom::sequence::{preceded, tuple, terminated};
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use crate::template::object::extractable::ExtractableObject;
use nom::character::complete::char;
use nom::multi::separated_list1;
use crate::core::parser::string;
use crate::template::VariableReferenceName;

impl Parsable for ExtractableResponse{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            extractableresponse_starting_with_body,
            extractableresponse_starting_with_headers
            ))(input)
    }
}
fn extractableresponse_starting_with_body<'a>(input: &'a str) -> ParseResult<'a, ExtractableResponse> {
    map(
        tuple((
                  preceded(ws(tag("body")),ws(ExtractableResponseBody::parser)),
                    opt(preceded(tuple((ws(tag("and")),ws(tag("headers")))),ws(ExtractableResponseHeaders::parser)))
              )),|(body,headers)|ExtractableResponse{
            body:Option::Some(body),
            headers
        })(input)
}
fn extractableresponse_starting_with_headers<'a>(input: &'a str) -> ParseResult<'a, ExtractableResponse> {
    map(
        tuple((
            preceded(ws(tag("headers")),ws(ExtractableResponseHeaders::parser)),
            opt(preceded(tuple((ws(tag("and")),ws(tag("body")))),ws(ExtractableResponseBody::parser)))
        )),|(headers,body)|ExtractableResponse{
            body,
            headers:Option::Some(headers)
        })(input)
}
impl Parsable for ExtractableResponseBody{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(ExtractableObject::parser,|eo|
        ExtractableResponseBody::WithObject(eo))(input)
    }
}
impl Parsable for ExtractableResponseHeaders{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            preceded(
                ws(char('{')),
                terminated(
                    separated_list1(ws(char(',')),ws(ExtractableResponseHeaderPair::parser)),
                    ws(char('}')))),|headers|ExtractableResponseHeaders{
                headers
            })(input)
    }
}
impl Parsable for ExtractableResponseHeaderPair{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
            ws(string),
            ws(char(':')),
            ws(ExtractableResponseHeaderValue::parser)
            )),|(key,_,value)|ExtractableResponseHeaderPair{
            key,
            value
        })(input)
    }
}
impl Parsable for ExtractableResponseHeaderValue{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(VariableReferenceName::parser,
        |var|ExtractableResponseHeaderValue::WithVariableReference(var))(input)
    }
}

#[cfg(test)]
mod tests{
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    use crate::template::rest::extractable::{ExtractableResponse, ExtractableResponseBody, ExtractableResponseHeaders, ExtractableResponseHeaderPair, ExtractableResponseHeaderValue};
    use crate::template::object::extractable::{ExtractableObject, ExtractableMapObject, ExtractablePair};
    use crate::template::VariableReferenceName;


    #[test]
    fn should_parse_extractableresponse_starting_with_body(){
        let text=r#"body object {"name":name } and headers { "X-API-KEY": x_api_key}"#;
        let a=ExtractableResponse::parser(text);
        let emo = ExtractableMapObject::WithPairs(vec![ExtractablePair::WithKeyValue(format!("name"),ExtractableObject::WithVariableReference(VariableReferenceName::from("name")))]);
        assert_if(text,a,ExtractableResponse{
            body:Option::Some(ExtractableResponseBody::WithObject(ExtractableObject::WithMapObject(emo))),
            headers:Option::Some(ExtractableResponseHeaders{
                headers:vec![ExtractableResponseHeaderPair{
                    key:format!("X-API-KEY"),
                    value:ExtractableResponseHeaderValue::WithVariableReference(VariableReferenceName::from("x_api_key"))
                }]
            })
        });
    }
    #[test]
    fn should_parse_extractableresponsebody(){
        let text=r#"object {"name":name, "place" :place }"#;
        let a=ExtractableResponseBody::parser(text);
        let emo = ExtractableMapObject::WithPairs(vec![
            ExtractablePair::WithKeyValue(format!("name"),ExtractableObject::WithVariableReference(VariableReferenceName::from("name"))),
            ExtractablePair::WithKeyValue(format!("place"),ExtractableObject::WithVariableReference(VariableReferenceName::from("place")))
        ]);
        assert_if(text,a,ExtractableResponseBody::WithObject(ExtractableObject::WithMapObject(emo)));
    }
    #[test]
    fn should_parse_extractableresponseheaders(){
        let text=r#"{"name":name, "place" :place }"#;
        let a=ExtractableResponseHeaders::parser(text);
        assert_if(text,a,ExtractableResponseHeaders{
            headers:vec![
                ExtractableResponseHeaderPair{
                key:format!("name"),
                value:ExtractableResponseHeaderValue::WithVariableReference(VariableReferenceName::from("name"))
            },
            ExtractableResponseHeaderPair{
                key:format!("place"),
                value:ExtractableResponseHeaderValue::WithVariableReference(VariableReferenceName::from("place"))
            }
            ]
        });
    }
    #[test]
    fn should_parse_extractableresponseheaderpair(){
        let text=r#""name":name"#;
        let a=ExtractableResponseHeaderPair::parser(text);
        assert_if(text,a,
                ExtractableResponseHeaderPair{
                    key:format!("name"),
                    value:ExtractableResponseHeaderValue::WithVariableReference(VariableReferenceName::from("name"))
                });
    }
    #[test]
    fn should_parse_extractableresponseheadervalue(){
        let text=r#"name"#;
        let a=ExtractableResponseHeaderValue::parser(text);
        assert_if(text,a,
                  ExtractableResponseHeaderValue::WithVariableReference(VariableReferenceName::from("name")));
    }
}