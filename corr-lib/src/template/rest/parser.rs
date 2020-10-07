use crate::parser::{Parsable, ParseResult, ws};
use crate::template::rest::{RestVerb, FillableRequest, FillableRequestBody, FillableRequestHeaders, URL, FillableRequestHeaderPair, FillableRequestHeaderValue};
use nom::branch::alt;
use nom::combinator::{map, opt};
use nom::bytes::complete::tag;
use nom::sequence::{preceded, tuple, terminated};
use nom::character::complete::char;
use crate::template::text::Text;
use crate::template::object::FillableObject;
use nom::multi::separated_list1;
use crate::core::parser::string;
use crate::template::Expression;

impl Parsable for RestVerb {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(tag("get"),|_|RestVerb::GET),
            map(tag("put"),|_|RestVerb::PUT),
            map(tag("post"),|_|RestVerb::POST),
            map(tag("patch"),|_|RestVerb::PATCH),
            map(tag("delete"),|_|RestVerb::DELETE),
            ))(input)
    }
}

impl Parsable for FillableRequest{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            tuple((
                ws(RestVerb::parser),
                ws(tag("request")),
                preceded(ws(char('{')),terminated(
                    tuple((preceded(tuple((ws(tag("url")),ws(char(':')))),URL::parser),
                        opt(preceded(tuple((ws(char(',')),ws(tag("body")),ws(char(':')))),FillableRequestBody::parser)),
                       opt(preceded(tuple((ws(char(',')),ws(tag("headers")),ws(char(':')))),FillableRequestHeaders::parser)),
                    ))
                         ,ws(char('}'))))

                ))
        ,|(verb,_,(url,body,headers))|FillableRequest{
                verb,headers,body,url
            }
        )(input)
    }
}
impl Parsable for URL{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(Expression::parser,|expr| URL::WithExpression(expr)),
            map(Text::parser,|txt| URL::WithText(txt))
            ))
        (input)
    }
}
impl Parsable for FillableRequestBody{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(FillableObject::parser,|obj|
        FillableRequestBody::WithObject(obj))(input)
    }
}
impl Parsable for FillableRequestHeaders{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(preceded(ws(char('{')),terminated(separated_list1(ws(char(',')),FillableRequestHeaderPair::parser),ws(char('}')))),|headers|
        FillableRequestHeaders{
            headers
        })(input)
    }
}
impl Parsable for FillableRequestHeaderPair{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
            ws(string),
            ws(char(':')),
            FillableRequestHeaderValue::parser
            )),|(key,_,value)|
        FillableRequestHeaderPair{
            key,
            value
        })(input)
    }
}
impl Parsable for FillableRequestHeaderValue{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
                map(ws(Expression::parser),|expr|FillableRequestHeaderValue::WithExpression(expr)),
                map(ws(Text::parser),|txt|FillableRequestHeaderValue::WithText(txt))
            ))(input)

    }
}
#[cfg(test)]
mod tests{
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    use crate::template::rest::{RestVerb, FillableRequest, URL, FillableRequestHeaders, FillableRequestHeaderPair, FillableRequestHeaderValue, FillableRequestBody};
    use crate::template::Expression;
    use crate::core::Value;
    use crate::template::object::{FillableObject, FillableMapObject, FillablePair};
    use crate::template::text::{Text, Block};


    #[test]
    fn should_parse_restverb_get(){
        let text=r#"get"#;
        let a=RestVerb::parser(text);
        assert_if(text,a,RestVerb::GET);
    }
    #[test]
    fn should_parse_restverb_put(){
        let text=r#"put"#;
        let a=RestVerb::parser(text);
        assert_if(text,a,RestVerb::PUT);
    }
    #[test]
    fn should_parse_restverb_post(){
        let text=r#"post"#;
        let a=RestVerb::parser(text);
        assert_if(text,a,RestVerb::POST);
    }
    #[test]
    fn should_parse_restverb_patch(){
        let text=r#"patch"#;
        let a=RestVerb::parser(text);
        assert_if(text,a,RestVerb::PATCH);
    }
    #[test]
    fn should_parse_restverb_delete(){
        let text=r#"delete"#;
        let a=RestVerb::parser(text);
        assert_if(text,a,RestVerb::DELETE);
    }
    #[test]
    fn should_parse_fillablerequest_when_no_headers_and_body(){
        let text=r#"get request {
            url: "http://localhost"
        }"#;
        let a=FillableRequest::parser(text);
        assert_if(text,a,FillableRequest{
            url:URL::WithExpression(Expression::Constant(Value::String(format!("http://localhost")))),
            verb:RestVerb::GET,
            body:Option::None,
            headers:Option::None
        });
    }

    #[test]
    fn should_parse_fillablerequest_when_headers_and_no_body(){
        let text=r#"get request {
            url: "http://localhost",
            headers: {
                "X-API-KEY": x_api_key
            }
        }"#;
        let a=FillableRequest::parser(text);
        assert_if(text,a,FillableRequest{
            url:URL::WithExpression(Expression::Constant(Value::String(format!("http://localhost")))),
            verb:RestVerb::GET,
            body:Option::None,
            headers:Option::Some(FillableRequestHeaders{
                headers:vec![FillableRequestHeaderPair{
                    key:format!("X-API-KEY"),
                    value:FillableRequestHeaderValue::WithExpression(Expression::Variable(format!("x_api_key"),Option::None))
                }]
            })
        });
    }
    #[test]
    fn should_parse_fillablerequest_when_headers_and_body(){
        let text=r#"get request {
            url: "http://localhost",
            body: object {
                "name" : name
            },
            headers: {
                "X-API-KEY": x_api_key
            }
        }"#;
        let a=FillableRequest::parser(text);
        let fmo = FillableMapObject::WithPairs(vec![FillablePair::WithKeyAndValue(format!("name"),FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None)))]);
        assert_if(text,a,FillableRequest{
            url:URL::WithExpression(Expression::Constant(Value::String(format!("http://localhost")))),
            verb:RestVerb::GET,
            body:Option::Some(FillableRequestBody::WithObject(FillableObject::WithMap(fmo))),
            headers:Option::Some(FillableRequestHeaders{
                headers:vec![FillableRequestHeaderPair{
                    key:format!("X-API-KEY"),
                    value:FillableRequestHeaderValue::WithExpression(Expression::Variable(format!("x_api_key"),Option::None))
                }]
            })
        });
    }

    #[test]
    fn should_parse_url_when_expression(){
        let text=r#"name"#;
        let a=URL::parser(text);
        assert_if(text,a,URL::WithExpression(Expression::Variable(format!("name"),Option::None)));
    }
    #[test]
    fn should_parse_url_when_text(){
        let text=r#"text `http://localhost`"#;
        let a=URL::parser(text);
        assert_if(text,a,URL::WithText(Text{
            blocks:vec![Block::Text(format!("http://localhost"))]
        }));
    }
    #[test]
    fn should_parse_fillablerequestbody(){
        let text=r#"object {
                "name" : name
            }"#;
        let fmo = FillableMapObject::WithPairs(vec![FillablePair::WithKeyAndValue(format!("name"),FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None)))]);
        let a=FillableRequestBody::parser(text);
        assert_if(text,a,FillableRequestBody::WithObject(FillableObject::WithMap(fmo)));
    }
    #[test]
    fn should_parse_fillablerequestheaders(){
        let text=r#"{
                "X-API-KEY": x_api_key,
                "Authorization": token
            }"#;
        let a=FillableRequestHeaders::parser(text);
        assert_if(text,a,FillableRequestHeaders{
            headers:vec![
                FillableRequestHeaderPair{
                    key:format!("X-API-KEY"),
                    value:FillableRequestHeaderValue::WithExpression(Expression::Variable(format!("x_api_key"),Option::None))
                },
                FillableRequestHeaderPair{
                    key:format!("Authorization"),
                    value:FillableRequestHeaderValue::WithExpression(Expression::Variable(format!("token"),Option::None))
                }
            ]
        });
    }
    #[test]
    fn should_parse_fillablerequestheaderpair(){
        let text=r#""X-API-KEY": x_api_key"#;
        let a=FillableRequestHeaderPair::parser(text);
        assert_if(text,a,
                FillableRequestHeaderPair{
                    key:format!("X-API-KEY"),
                    value:FillableRequestHeaderValue::WithExpression(Expression::Variable(format!("x_api_key"),Option::None))
                });
    }
    #[test]
    fn should_parse_fillablerequestheadervalue(){
        let text=r#"x_api_key"#;
        let a=FillableRequestHeaderValue::parser(text);
        assert_if(text,a,FillableRequestHeaderValue::WithExpression(Expression::Variable(format!("x_api_key"),Option::None)));
    }
}