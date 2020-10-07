use crate::parser::{Parsable, ParseResult, ws};
use crate::journey::step::rest::RestSetp;
use nom::combinator::{map, opt};
use nom::sequence::{tuple, preceded};
use crate::template::rest::FillableRequest;
use nom::bytes::complete::tag;
use crate::template::rest::extractable::ExtractableResponse;

impl Parsable for RestSetp{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            tuple((
                ws(FillableRequest::parser),
                opt(preceded(ws(tag("matching")),ExtractableResponse::parser))
                )),
            |(request,response)|RestSetp{
                request,
                response
            }
        )(input)
    }
}
#[cfg(test)]
mod tests{
    use crate::parser::Parsable;
    use crate::parser::util::assert_if;
    use crate::journey::step::rest::RestSetp;
    use crate::template::rest::{FillableRequest, URL, RestVerb, FillableRequestHeaders, FillableRequestHeaderPair, FillableRequestHeaderValue};
    use crate::template::{Expression, VariableReferenceName};
    use crate::core::Value;
    use crate::template::rest::extractable::{ExtractableResponse, ExtractableResponseBody, ExtractableResponseHeaders, ExtractableResponseHeaderPair, ExtractableResponseHeaderValue};
    use crate::template::object::extractable::{ExtractableObject, ExtractablePair, ExtractableMapObject};


    #[tokio::test]
    async fn should_parse_reststep_with_extraction(){
        let j= r#"get request {
            url: "http://localhost",
            headers: {
                "X-API-KEY": x_api_key
            }
        } matching body object {"name":name } and headers { "X-API-KEY": x_api_key}"#;
        let emo = ExtractableMapObject::WithPairs(vec![ExtractablePair::WithKeyValue(format!("name"),ExtractableObject::WithVariableReference(VariableReferenceName::from("name")))]);
        assert_if(j
                  ,RestSetp::parser(j)
                  ,RestSetp {
                request:FillableRequest{
                    url:URL::WithExpression(Expression::Constant(Value::String(format!("http://localhost")))),
                    verb:RestVerb::GET,
                    body:Option::None,
                    headers:Option::Some(FillableRequestHeaders{
                        headers:vec![FillableRequestHeaderPair{
                            key:format!("X-API-KEY"),
                            value:FillableRequestHeaderValue::WithExpression(Expression::Variable(format!("x_api_key"),Option::None))
                        }]
                    })
                },
                response:Option::Some(ExtractableResponse{
                    body:Option::Some(ExtractableResponseBody::WithObject(ExtractableObject::WithMapObject(emo))),
                    headers:Option::Some(ExtractableResponseHeaders{
                        headers:vec![ExtractableResponseHeaderPair{
                            key:format!("X-API-KEY"),
                            value:ExtractableResponseHeaderValue::WithVariableReference(VariableReferenceName::from("x_api_key"))
                        }]
                    })
                })
            })

    }
    #[tokio::test]
    async fn should_parse_reststep_when_no_extraction(){
        let j= r#"get request {
            url: "http://localhost",
            headers: {
                "X-API-KEY": x_api_key
            }
        }"#;
        assert_if(j
                  ,RestSetp::parser(j)
                  ,RestSetp{
                request:FillableRequest{
                    url:URL::WithExpression(Expression::Constant(Value::String(format!("http://localhost")))),
                    verb:RestVerb::GET,
                    body:Option::None,
                    headers:Option::Some(FillableRequestHeaders{
                        headers:vec![FillableRequestHeaderPair{
                            key:format!("X-API-KEY"),
                            value:FillableRequestHeaderValue::WithExpression(Expression::Variable(format!("x_api_key"),Option::None))
                        }]
                    })
                },
                response:Option::None
            })

    }
}