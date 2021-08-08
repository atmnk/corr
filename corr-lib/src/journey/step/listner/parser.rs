use crate::parser::{Parsable, ParseResult, ws};
use nom::combinator::{map, opt};
use nom::sequence::{tuple};
use crate::template::rest::{RestVerb};
use nom::bytes::complete::tag;

use crate::journey::step::listner::{StartListenerStep, Stub, StubResponse};
use crate::core::parser::{positive_integer};
use nom::multi::many0;
use crate::template::Expression;
use crate::journey::step::Step;

use crate::template::text::extractable::ExtractableText;
use crate::template::rest::extractable::ExtractableRestData;

impl Parsable for Stub {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            tuple((
                ws(tag("on")),
                ws(RestVerb::parser),
                ws(tag("with")),
                ws(tag("url")),
                ExtractableText::parser,
                opt(tuple((ws(tag("matching")), ws(tag("request")),ExtractableRestData::parser))),
                ws(tag("{")),
                many0(Step::parser),
                tuple((
                    ws(tag("respond")),
                    ws(tag("with")),
                    opt(tuple((
                        ws(tag("status")),
                        ws(positive_integer),
                        ws(tag("and"))))),
                    ws(tag("body")),
                    Expression::parser,
                )),
                ws(tag("}")),
            )),
            |(_,
                 method,_,_,
                 url,
                 rd,
                 _,
                 steps,
                 (_,_,status,_,body),_)| {
                Stub{
                    rest_data:rd.map(|(_,_,r)|r).unwrap_or(ExtractableRestData{headers:Option::None,body:Option::None}),
                    method,
                    url,
                    steps,
                    response: StubResponse::from(status.map(|(_,s,_)|s),body)
                }
            }
        )(input)
    }
}
impl Parsable for StartListenerStep {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            tuple((
                ws(tag("listen")),
                ws(tag("on")),
                ws(Expression::parser),
                ws(tag("with")),
                ws(tag("{")),
                many0(ws(Stub::parser)),
                ws(tag("}"))
                )),
            |(_,_,port,_,_,stubs,_)| StartListenerStep {
               port,
                stubs
            }
        )(input)
    }
}
#[cfg(test)]
mod tests{
    use crate::parser::util::{assert_if, assert_no_error};
    use crate::journey::step::listner::{StartListenerStep, Stub, StubResponse};
    use crate::parser::Parsable;
    use crate::template::rest::RestVerb;
    use crate::template::Expression;
    use crate::template::text::extractable::ExtractableText;
    use crate::core::Variable;
    use crate::template::rest::extractable::ExtractableRestData;

    #[tokio::test]
    async fn should_parse_start_listner_step(){
        let j= r#"listen on p1 with {
        on get with url text `/values/<%id%>` {
           respond with body name
        }

    }"#;
        assert_if(j,StartListenerStep::parser(j),StartListenerStep{
            port:Expression::Variable(format!("p1"),Option::None),
            stubs:vec![Stub{
                rest_data:ExtractableRestData
                {
                    headers:Option::None,
                    body:Option::None
                },
                method:RestVerb::GET,
                steps:vec![],
                url:ExtractableText::Multi(Option::Some("/values/".to_string()),vec![]
                    ,Option::Some(Variable::new("id"))),
                response:StubResponse{
                    status:200,
                    body:Expression::Variable(format!("name"),Option::None)
                }
            }]
        })

    }

    #[tokio::test]
    async fn should_parse_start_listner_step_with_variables(){
        let j= r#"listen on p1 with {
            on post with url text `/values` matching request body object { "name":name,"place":place } {
                print text `POST`
                let obj.id = id
                let id = id + 1
                let obj.name = concat(fake("FirstName")," ",fake("LastName"))
                let obj.company = concat(fake("CompanyName"))
                objects.push(obj)
                sync objects
                respond with body objects
            }
        }"#;
        assert_no_error(j,StartListenerStep::parser(j))

    }
}