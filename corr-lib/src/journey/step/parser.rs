use crate::journey::step::Step;
use nom::combinator::map;
use crate::parser::{ParseResult, ws};
use crate::journey::step::system::SystemStep;
use crate::parser::Parsable;
use nom::branch::alt;
use crate::journey::step::rest::RestSetp;

impl Parsable for Step{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(ws(SystemStep::parser),Step::System),
            map(ws(RestSetp::parser),Step::Rest)
            ))(input)
    }
}
#[cfg(test)]
mod tests{
    use crate::parser::Parsable;
    use crate::parser::util::{assert_no_error};
    use crate::journey::step::Step;

    #[tokio::test]
    async fn should_parse_step_with_system_step(){
        let j= r#"
            print text `Hello <%concat("Atmaram","Naik")%>`;
        "#;
        assert_no_error(j
                        ,Step::parser(j)
        )
    }
    #[tokio::test]
    async fn should_parse_step_with_rest_step(){
        let j= r#"
            post request { url: text `http://localhost:9090`, body: object { "name":name }, headers:{ "Content-Type": "application/json"}} matching  headers { "Authorization":token} and body object { "id":id };
        "#;
        assert_no_error(j
                        ,Step::parser(j)
        )
    }
}