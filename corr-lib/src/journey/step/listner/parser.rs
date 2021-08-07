use crate::parser::{Parsable, ParseResult, ws};
use nom::combinator::{map, opt};
use nom::sequence::{tuple, preceded};
use crate::template::rest::{FillableRequest, RestVerb};
use nom::bytes::complete::tag;
use crate::template::rest::extractable::ExtractableResponse;
use crate::journey::step::listner::{StartListenerStep, Stub, StubResponse};
use crate::core::parser::{port, string, positive_integer};
use nom::multi::many0;
use crate::template::Expression;
use crate::journey::step::Step;
use rand::distributions::Exp;
use crate::template::text::extractable::ExtractableText;

impl Parsable for Stub {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            tuple((
                ws(tag("on")),
                ws(RestVerb::parser),
                ws(tag("with")),
                ws(tag("url")),
                ExtractableText::parser,
                ws(tag("{")),
                many0(Step::parser),
                tuple(((
                    ws(tag("respond")),
                    ws(tag("with")),
                    opt(tuple((
                        ws(tag("status")),
                        ws(positive_integer),
                        ws(tag("and"))))),
                    ws(tag("body")),
                    Expression::parser,
                ))),
                ws(tag("}")),
            )),
            |(_,
                 method,_,_,
                 url,_,
                 steps,
                 (_,_,status,_,body),_)| {
                Stub{
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
            on get with url text `/values/<%id%>` {
           print text `GET`
           let resp = object {}
           objects.for(obj)=>{
               if obj.id==id {
                  let resp = obj
               }
           }
           respond with body resp
           }
        }"#;
        assert_no_error(j,StartListenerStep::parser(j))

    }
}