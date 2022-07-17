use crate::parser::{ParseResult, ws, non_back_quote, identifier_part, executable_identifier};
use crate::journey::{ImportStatement, Journey};
use nom::branch::alt;
use nom::sequence::{terminated, preceded, tuple};
use nom::character::complete::{char};
use nom::bytes::complete::{tag};
use nom::combinator::{map, opt};
use nom::multi::{many0, separated_list0};
use crate::journey::step::Step;
use crate::parser::Parsable;
use crate::core::Variable;
use crate::template::VariableReferenceName;

impl Parsable for ImportStatement {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("import")),ws(VariableReferenceName::parser),opt(preceded(ws(tag("as")),ws(VariableReferenceName::parser))))),|
            (_,pn,lno)|{
            if let Some(ln) = lno {
                Self{
                    physical_name:pn,
                    logical_name:ln
                }
            } else {
                Self{
                    physical_name:pn.clone(),
                    logical_name:VariableReferenceName::from(pn.parts.last().unwrap())
                }
            }
        })(input)
    }
}
impl Parsable for Journey{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map( tuple((
            many0(ws(ImportStatement::parser)),
            parse_executable_name,ws(tag("(")),separated_list0(ws(tag(",")),Variable::parser),ws(tag(")")),ws(char('{')),steps,ws(char('}')))),|(import_statements,name,_,params,_,_,steps,_)|{
            Journey{
                import_statements,
                name,
                steps,
                params
            }
        })(input)
    }
}
pub fn steps<'a>(input:&'a str) ->ParseResult<'a,Vec<Step>>{
    many0(ws(Step::parser))(input)
}

pub fn parse_name<'a>(input:&'a str) ->ParseResult<'a,String>{
    alt((
        terminated(preceded(char('`'),non_back_quote),char('`')),
        map(identifier_part,|val| val.to_string())
    ))(input)

}
pub fn parse_executable_name<'a>(input:&'a str) ->ParseResult<'a,String>{
    alt((
        terminated(preceded(char('`'),non_back_quote),char('`')),
        map(executable_identifier,|val| val.to_string())
    ))(input)

}
#[cfg(test)]
mod tests{
    use crate::parser::Parsable;
    use crate::parser::util::{assert_no_error};
    use crate::journey::Journey;
    #[tokio::test]
    async fn should_parse_journey_with_plain_import_statements(){
        let j= r#"import com.qalens.Hello `Hello World`(){
            print text `Hello <%name%>`
            print text `Hello <%name%>`
        }"#;
        assert_no_error(j
                        ,Journey::parser(j)
        )
    }
    #[tokio::test]
    async fn should_parse_journey_with_multiple_import_statements(){
        let j= r#"import com.qalens.Hello import com.qalens.World`Hello World`(){
            print text `Hello <%name%>`
            print text `Hello <%name%>`
        }"#;
        assert_no_error(j
                        ,Journey::parser(j)
        )
    }
    #[tokio::test]
    async fn should_parse_journey_with_alias_import_statements(){
        let j= r#"import com.qalens.Hello as MyHello`Hello World`(){
            print text `Hello <%name%>`
            print text `Hello <%name%>`
        }"#;
        assert_no_error(j
                        ,Journey::parser(j)
        )
    }
    #[tokio::test]
    async fn should_parse_journey(){
        let j= r#"`Hello World`(){
            print text `Hello <%name%>`
            print text `Hello <%name%>`
        }"#;
        assert_no_error(j
                        ,Journey::parser(j)
        )
    }
    #[tokio::test]
    async fn should_parse_journey_with_identifiers(){
        let j= r#"Hello(){
            print text `Hello <%name%>`
            print text `Hello <%name%>`
        }"#;
        assert_no_error(j
                        ,Journey::parser(j)
        )
    }
    #[tokio::test]
    async fn should_parse_journey_with_parameters(){
        let j= r#"`Hello World`(name){
            print text `Hello <%name%>`
            print text `Hello <%name%>`
        }"#;
        assert_no_error(j
                        ,Journey::parser(j)
        )
    }
    #[tokio::test]
    async fn should_parse_journey_with_comments(){
        let j= r#"`Server`(){
//    print text `Hello1`
//    print text `Hello2`
//    print text `Hello3`
//    listen on 9088 with {
//        on post with url text `/` matching request body object {"filed1":fields.for(field)=>field.name} {
//            respond with body concat(fields,james)
//        }
//    }
    print text `Hello`
}"#;
        assert_no_error(j
                        ,Journey::parser(j)
        )
    }
}

