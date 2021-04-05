use crate::parser::{Parsable, ParseResult, ws};
use crate::template::object::{FillableObject, FillableMapObject, FillablePair, FillableForLoop};
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple, terminated};
use nom::bytes::complete::tag;
use nom::branch::alt;
use crate::template::{Expression, VariableReferenceName};
use nom::character::complete::char;
use nom::multi::{separated_list0};
use crate::core::parser::string;

impl Parsable for FillableObject{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
       preceded(ws(tag("object")),fillable_obj_rhs_parser
       )(input)
    }
}
fn fillable_obj_rhs_parser<'a>(input: &'a str) -> ParseResult<'a, FillableObject> {
             alt((
                 map(FillableForLoop::parser,|ffl|FillableObject::WithForLoop(Box::new(ffl))),
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
impl Parsable for FillableForLoop{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
                terminated(ws(VariableReferenceName::parser),tuple((ws(char('.')),ws(tag("for"))))),
                alt((
                        arged_fillable_for,
                    unarged_fillable_for
                    ))

            )),|(on,(with,index,inner))|FillableForLoop{
            on,
            with,
            index,
            inner
        })(input)
    }
}
fn arged_fillable_for<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>,Option<VariableReferenceName>,FillableObject)> {
    map(tuple((
        preceded(ws(char('(')),terminated(tuple((ws(VariableReferenceName::parser),opt(preceded(ws(char(',')),ws(VariableReferenceName::parser))))),ws(char(')')))),
        preceded(ws(tag("=>")),ws(fillable_obj_rhs_parser))
        )),|((with,index),inner)|(Option::Some(with),index,inner)) (input)
}
fn unarged_fillable_for<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>,Option<VariableReferenceName>,FillableObject)> {
    map(ws(fillable_obj_rhs_parser),|inner|(Option::None,Option::None,inner)) (input)
}
impl Parsable for FillablePair{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(string),ws(char(':')),ws(fillable_obj_rhs_parser))),|(key,_,value)|FillablePair::WithKeyAndValue(key,value))(input)
    }
}

#[cfg(test)]
mod tests{
    use crate::template::object::{FillablePair, FillableObject, FillableMapObject, FillableForLoop};
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    use crate::template::{Expression, VariableReferenceName};
    use crate::core::{Value};
    use crate::template::object::parser::fillable_obj_rhs_parser;

    #[test]
    fn should_parse_fillablepair(){
        let text=r#""name":"Atmaram""#;
        let a=FillablePair::parser(text);
        assert_if(text,a,FillablePair::WithKeyAndValue(format!("name"),FillableObject::WithExpression(Expression::Constant(Value::String(format!("Atmaram"))))))
    }

    #[test]
    fn should_parse_fillableforloop_for_unarged_fillable_for(){
        let text=r#"names.for name"#;
        let a=FillableForLoop::parser(text);
        assert_if(text,a,FillableForLoop{
            on:VariableReferenceName::from("names"),
            with:Option::None,
            index:Option::None,
            inner:FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None))
        })
    }

    #[test]
    fn should_parse_fillableforloop_for_arged_fillable_for_without_index(){
        let text=r#"names.for(name)=> name"#;
        let a=FillableForLoop::parser(text);
        assert_if(text,a,FillableForLoop{
            on:VariableReferenceName::from("names"),
            with:Option::Some(VariableReferenceName::from("name")),
            index:Option::None,
            inner:FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None))
        })
    }

    #[test]
    fn should_parse_fillableforloop_for_arged_fillable_for_with_index(){
        let text=r#"names.for(name,index)=> name"#;
        let a=FillableForLoop::parser(text);
        assert_if(text,a,FillableForLoop{
            on:VariableReferenceName::from("names"),
            with:Option::Some(VariableReferenceName::from("name")),
            index:Option::Some(VariableReferenceName::from("index")),
            inner:FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None))
        })
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
    // #[test]
    // fn should_parse_fillable_obj_rhs_when_expression_with_operators(){
    //     let text=r#"name % 15 + 10"#;
    //     let a=fillable_obj_rhs_parser(text);
    //     assert_if(text,a,FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None)))
    // }

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