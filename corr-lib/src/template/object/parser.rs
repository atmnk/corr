use crate::parser::{Parsable, ParseResult, ws};
use crate::template::object::{FillableObject, FillableMapObject, FillablePair};
use nom::combinator::map;
use nom::sequence::{preceded, tuple, terminated};
use nom::bytes::complete::tag;
use nom::branch::alt;
use crate::template::Expression;
use nom::character::complete::char;
use nom::multi::{separated_list0};
use crate::core::parser::string;

impl Parsable for FillableObject{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
       preceded(tuple((ws(tag("fillable")),ws(tag("object")))),fillable_obj_rhs_parser
       )(input)
    }
}
fn fillable_obj_rhs_parser<'a>(input: &'a str) -> ParseResult<'a, FillableObject> {
             alt((
                 map(Expression::parser,|expr|FillableObject::WithExpression(expr)),
                 map(FillableMapObject::parser,|fmo|FillableObject::WithMap(fmo)),
                 map(preceded(ws(char('[')),terminated(separated_list0(ws(char(',')),ws(fillable_obj_rhs_parser)),ws(char(']')))),|arr|FillableObject::WithArray(arr))
             )
    )(input)
}
impl Parsable for FillableMapObject{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(preceded(ws(char('{')),terminated(separated_list0(ws(char(',')),FillablePair::parser),ws(char('}')))),|fps|FillableMapObject::WithPairs(fps))(input)
    }
}
impl Parsable for FillablePair{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(string),ws(char(':')),ws(fillable_obj_rhs_parser))),|(key,_,value)|FillablePair::WithKeyAndValue(key,value))(input)
    }
}

#[cfg(test)]
mod tests{
    use crate::template::object::{FillablePair, FillableObject, FillableMapObject};
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    use crate::template::Expression;
    use crate::core::{Value};
    use crate::template::object::parser::fillable_obj_rhs_parser;

    #[test]
    fn should_parse_fillablepair(){
        let text=r#""name":"Atmaram""#;
        let a=FillablePair::parser(text);
        assert_if(text,a,FillablePair::WithKeyAndValue(format!("name"),FillableObject::WithExpression(Expression::Constant(Value::String(format!("Atmaram"))))))
    }

    #[test]
    fn should_parse_fillablemapobject(){
        let text=r#"{"name":"Atmaram"}"#;
        let a=FillableMapObject::parser(text);
        assert_if(text,a,FillableMapObject::WithPairs(vec![FillablePair::WithKeyAndValue(format!("name"),FillableObject::WithExpression(Expression::Constant(Value::String(format!("Atmaram")))))]))
    }

    #[test]
    fn should_parse_fillable_obj_rhs_when_expression(){
        let text=r#"name"#;
        let a=fillable_obj_rhs_parser(text);
        assert_if(text,a,FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None)))
    }

    #[test]
    fn should_parse_fillable_obj_rhs_when_object(){
        let text=r#"{"name":"Atmaram"}"#;
        let a=fillable_obj_rhs_parser(text);
        assert_if(text,a,FillableObject::WithMap(FillableMapObject::WithPairs(vec![FillablePair::WithKeyAndValue(format!("name"),FillableObject::WithExpression(Expression::Constant(Value::String(format!("Atmaram")))))])))
    }
    #[test]
    fn should_parse_fillable_obj_rhs_when_array(){
        let text=r#"[name,place]"#;
        let a=fillable_obj_rhs_parser(text);
        assert_if(text,a,FillableObject::WithArray(
            vec![
                FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None)),
                FillableObject::WithExpression(Expression::Variable(format!("place"),Option::None))
            ]
        ))
    }
}