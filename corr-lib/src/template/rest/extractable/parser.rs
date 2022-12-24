use crate::parser::{Parsable, ParseResult, ws};
use crate::template::rest::extractable::{ExtractableRestData, ExtractableBody, ExtractableHeaders, ExtractableHeaderPair, ExtractableHeaderValue};
use nom::branch::alt;
use nom::sequence::{preceded, tuple, terminated};
use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use crate::template::object::extractable::ExtractableObject;
use nom::character::complete::char;
use nom::multi::separated_list1;
use crate::core::parser::string;
use crate::template::VariableReferenceName;
use crate::template::form::extractable::ExtractableForm;
use crate::template::text::extractable::ExtractableText;

impl Parsable for ExtractableRestData {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            extractableresponse_starting_with_body,
            extractableresponse_starting_with_headers
            ))(input)
    }
}
fn extractableresponse_starting_with_body<'a>(input: &'a str) -> ParseResult<'a, ExtractableRestData> {
    map(
        tuple((
            preceded(ws(tag("body")),ws(ExtractableBody::parser)),
            opt(preceded(tuple((ws(tag("and")),ws(tag("headers")))),ws(ExtractableHeaders::parser)))
              )),|(body,headers)| ExtractableRestData {
            body:Option::Some(body),
            headers
        })(input)
}
fn extractableresponse_starting_with_headers<'a>(input: &'a str) -> ParseResult<'a, ExtractableRestData> {
    map(
        tuple((
            preceded(ws(tag("headers")),ws(ExtractableHeaders::parser)),
            opt(preceded(tuple((ws(tag("and")),ws(tag("body")))),ws(ExtractableBody::parser)))
        )),|(headers,body)| ExtractableRestData {
            body,
            headers:Option::Some(headers)
        })(input)
}
impl Parsable for ExtractableBody {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
        map(ExtractableForm::parser,|ef|
            ExtractableBody::WithForm(ef)),
        map(ExtractableText::parser,|ef|
            ExtractableBody::WithText(ef)),
        map(ExtractableObject::parser,|eo|
            ExtractableBody::WithObject(eo)),

        ))(input)
    }
}
impl Parsable for ExtractableHeaders {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            preceded(
                ws(char('{')),
                terminated(
                    separated_list1(ws(char(',')),ws(ExtractableHeaderPair::parser)),
                    ws(char('}')))),|headers| ExtractableHeaders {
                headers
            })(input)
    }
}
impl Parsable for ExtractableHeaderPair {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
            ws(string),
            ws(char(':')),
            ws(ExtractableHeaderValue::parser)
            )),|(key,_,value)| ExtractableHeaderPair {
            key,
            value
        })(input)
    }
}
impl Parsable for ExtractableHeaderValue {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(VariableReferenceName::parser,
        |var| ExtractableHeaderValue::WithVariableReference(var))(input)
    }
}

#[cfg(test)]
mod tests{
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    use crate::template::rest::extractable::{ExtractableRestData, ExtractableBody, ExtractableHeaders, ExtractableHeaderPair, ExtractableHeaderValue};
    use crate::template::object::extractable::{ExtractableObject, ExtractableMapObject, ExtractablePair};
    use crate::template::VariableReferenceName;
    use crate::template::form::extractable::ExtractableForm;

    #[test]
    fn should_parse_extractable_body_with_form(){
        let text=r#"form {"name":name }"#;
        let a= ExtractableBody::parser(text);
        let emo = ExtractableBody::WithForm(ExtractableForm::WithFields(vec![("name".to_string(),VariableReferenceName::from("name"))]));
        assert_if(text, a, emo);
    }

    #[test]
    fn should_parse_extractableresponse_starting_with_body_object(){
        let text=r#"body object {"name":name } and headers { "X-API-KEY": x_api_key}"#;
        let a= ExtractableRestData::parser(text);
        let emo = ExtractableMapObject::WithPairs(vec![ExtractablePair::WithKeyValue(format!("name"),ExtractableObject::WithVariableReference(VariableReferenceName::from("name")))]);
        assert_if(text, a, ExtractableRestData {
            body:Option::Some(ExtractableBody::WithObject(ExtractableObject::WithMapObject(emo))),
            headers:Option::Some(ExtractableHeaders {
                headers:vec![ExtractableHeaderPair {
                    key:format!("X-API-KEY"),
                    value: ExtractableHeaderValue::WithVariableReference(VariableReferenceName::from("x_api_key"))
                }]
            })
        });
    }
    #[test]
    fn should_parse_extractableresponsebody(){
        let text=r#"object {"name":name, "place" :place }"#;
        let a= ExtractableBody::parser(text);
        let emo = ExtractableMapObject::WithPairs(vec![
            ExtractablePair::WithKeyValue(format!("name"),ExtractableObject::WithVariableReference(VariableReferenceName::from("name"))),
            ExtractablePair::WithKeyValue(format!("place"),ExtractableObject::WithVariableReference(VariableReferenceName::from("place")))
        ]);
        assert_if(text, a, ExtractableBody::WithObject(ExtractableObject::WithMapObject(emo)));
    }
    #[test]
    fn should_parse_extractableresponseheaders(){
        let text=r#"{"name":name, "place" :place }"#;
        let a= ExtractableHeaders::parser(text);
        assert_if(text, a, ExtractableHeaders {
            headers:vec![
                ExtractableHeaderPair {
                key:format!("name"),
                value: ExtractableHeaderValue::WithVariableReference(VariableReferenceName::from("name"))
            },
                ExtractableHeaderPair {
                key:format!("place"),
                value: ExtractableHeaderValue::WithVariableReference(VariableReferenceName::from("place"))
            }
            ]
        });
    }
    #[test]
    fn should_parse_extractableresponseheaderpair(){
        let text=r#""name":name"#;
        let a= ExtractableHeaderPair::parser(text);
        assert_if(text, a,
                  ExtractableHeaderPair {
                    key:format!("name"),
                    value: ExtractableHeaderValue::WithVariableReference(VariableReferenceName::from("name"))
                });
    }
    #[test]
    fn should_parse_extractableresponseheadervalue(){
        let text=r#"name"#;
        let a= ExtractableHeaderValue::parser(text);
        assert_if(text, a,
                  ExtractableHeaderValue::WithVariableReference(VariableReferenceName::from("name")));
    }
}