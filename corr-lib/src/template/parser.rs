use crate::parser::{Parsable, ParseResult, ws, identifier_part, scriptlet_keyword};
use crate::template::{Expression, VariableReferenceName};
use nom::combinator::{map};
use crate::core::{Value, Variable};
use nom::sequence::{tuple};
use nom::branch::alt;
use nom::character::complete::char;
use nom::multi::{separated_list0, separated_list1};

impl Parsable for Expression{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(Value::parser,|val|Expression::Constant(val)),
            map(tuple((scriptlet_keyword,ws(char('(')),separated_list0(ws(char(',')),Expression::parser),ws(char(')')))),|(name,_,expressions,_)|Expression::Function(name.to_string(),expressions)),
            map(Variable::parser,|val|Expression::Variable(val.name,val.data_type)),
            ))(input)
    }
}
impl Parsable for VariableReferenceName {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            separated_list1(char('.'),
                            map(identifier_part,|val|{val.to_string()})),|parts| { VariableReferenceName {parts}})(input)
    }
}
#[cfg(test)]
mod tests{
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    use crate::template::{Expression, VariableReferenceName};
    use crate::core::{Value};

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