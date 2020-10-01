use crate::journey::step::Step;
use nom::combinator::map;
use crate::parser::{ParseResult, ws};
use crate::journey::step::system::SystemStep;
use crate::parser::Parsable;

impl Parsable for Step{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
                map(ws(SystemStep::parser),Step::System)(input)
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
}