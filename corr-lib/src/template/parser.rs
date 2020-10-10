use crate::parser::{Parsable, ParseResult, ws, identifier_part, function_name};
use crate::template::{Expression, VariableReferenceName, Assignable};
use nom::combinator::{map};
use crate::core::{Value, Variable};
use nom::sequence::{tuple};
use nom::branch::alt;
use nom::character::complete::char;
use nom::multi::{separated_list0, separated_list1};
use crate::template::text::Text;
use crate::template::object::FillableObject;

impl Parsable for Assignable {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(Expression::parser,|expr|Assignable::Expression(expr)),
            map(FillableObject::parser,|flb_obj|Assignable::FillableObject(flb_obj)),
            map(Text::parser,|txt|Assignable::FillableText(txt))
            ))(input)
    }
}
impl Parsable for Expression{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(Value::parser,|val|Expression::Constant(val)),
            map(tuple((function_name, ws(char('(')), separated_list0(ws(char(',')), Expression::parser), ws(char(')')))), |(name,_,expressions,_)|Expression::Function(name.to_string(), expressions)),
            map(Variable::parser,|val|Expression::Variable(val.name,val.data_type)),
            ))(input)
    }
}
impl Parsable for VariableReferenceName {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            separated_list1(ws(char('.')),
                            map(identifier_part,|val|{val.to_string()})),|parts| { VariableReferenceName {parts}})(input)
    }
}
#[cfg(test)]
mod tests{
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    use crate::template::{Expression, VariableReferenceName, Assignable};
    use crate::core::{Value};
    use crate::template::text::{Text, Block, Scriplet};
    use crate::template::object::FillableObject;

    #[tokio::test]
    async fn should_parse_assignable_when_expression(){
        let txt = r#"name"#;
        let a = Assignable::parser(txt);
        assert_if(txt,a,Assignable::Expression(Expression::Variable(format!("name"),Option::None)))
    }

    #[tokio::test]
    async fn should_parse_assignable_when_fillabletext(){
        let txt = r#"text `Hello <%name%>`"#;
        let a = Assignable::parser(txt);
        assert_if(txt,a,Assignable::FillableText(Text {
            blocks:vec![
                Block::Text(format!("Hello ")),
                Block::Scriplet(Scriplet::Expression(Expression::Variable(format!("name"),Option::None)))
            ]
        }))

    }

    #[tokio::test]
    async fn should_parse_assignable_when_fillableobject(){
        let txt = r#"object name"#;
        let a = Assignable::parser(txt);
        assert_if(txt,a,Assignable::FillableObject(FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None))))
    }

    #[test]
    fn should_parse_expression_when_constant(){
        let text=r#""Atmaram""#;
        let a=Expression::parser(text);
        assert_if(text,a,Expression::Constant(Value::String("Atmaram".to_string())))
    }

    #[test]
    fn should_parse_expression_when_function(){
        let text=r#"concat("Atmaram","Naik")"#;
        let a=Expression::parser(text);
        assert_if(text,a,Expression::Function("concat".to_string(),vec![Expression::Constant(Value::String("Atmaram".to_string())),Expression::Constant(Value::String("Naik".to_string()))]))
    }
    #[test]
    fn should_parse_expression_when_variable(){
        let text=r#"name"#;
        let a=Expression::parser(text);
        assert_if(text,a,Expression::Variable("name".to_string(),Option::None))
    }

    #[test]
    fn should_parse_variable_reference_name(){
        let text=r#"place.name"#;
        let a=VariableReferenceName::parser(text);
        assert_if(text,a,VariableReferenceName{parts:vec!["place".to_string(),"name".to_string()]})
    }
    #[test]
    fn should_parse_partially_when_keyword_in_variable_reference_name(){
        let text=r#"place.name.print"#;
        let a=VariableReferenceName::parser(text);
        assert_if(text,a,VariableReferenceName{parts:vec!["place".to_string(),"name".to_string()]})
    }
}