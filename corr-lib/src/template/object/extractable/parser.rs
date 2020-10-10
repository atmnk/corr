use crate::parser::{Parsable, ParseResult, ws};
use crate::template::object::extractable::{ExtractableObject, ExtractableMapObject, ExtractablePair, ExtractableForLoop};
use nom::branch::alt;
use nom::combinator::map;
use crate::template::VariableReferenceName;
use nom::sequence::{preceded, terminated, tuple};
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::multi::{separated_list0, separated_list1};
use crate::core::parser::string;

impl Parsable for ExtractableObject{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        preceded(ws(tag("object")),extractable_object_options)(input)
    }
}
fn extractable_object_options<'a>(input: &'a str) -> ParseResult<'a, ExtractableObject> {
    alt((
        map(ws(ExtractableForLoop::parser),|efl|ExtractableObject::WithForLoop(Box::new(efl))),
        map(ws(VariableReferenceName::parser),|var|ExtractableObject::WithVariableReference(var)),
        map(ws(ExtractableMapObject::parser),|mo|ExtractableObject::WithMapObject(mo)),
        map(ws(Vec::<ExtractableObject>::parser),|vec|ExtractableObject::WithFixedArray(vec))
    ))(input)
}
impl Parsable for ExtractableForLoop{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
            ws(VariableReferenceName::parser),
            preceded(tuple((ws(char('.')),ws(tag("for")),ws(char('(')))),terminated(ws(VariableReferenceName::parser),ws(char(')')))),
            preceded(ws(tag("=>")),ws(extractable_object_options))
            )),
        |(on,with,inner)| ExtractableForLoop{
            on,
            with,
            inner
        }
        )(input)
    }
}
impl Parsable for ExtractableMapObject{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(preceded(ws(char('{')),terminated(separated_list0(ws(char(',')),ExtractablePair::parser) ,ws(char('}')))),|pairs|ExtractableMapObject::WithPairs(pairs))(input)
    }
}
impl Parsable for ExtractablePair{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(string),ws(char(':')),extractable_object_options)),|(key,_,value)|ExtractablePair::WithKeyValue(key,value))(input)
    }
}
impl Parsable for Vec<ExtractableObject>{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        preceded(ws(char('[')),terminated(separated_list1(ws(char(',')),extractable_object_options),ws(char(']'))))(input)
    }
}

#[cfg(test)]
mod tests{
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    use crate::template::{VariableReferenceName};
    use crate::template::object::extractable::{ExtractablePair, ExtractableObject, ExtractableMapObject, ExtractableForLoop};
    use crate::template::object::extractable::parser::extractable_object_options;

    #[test]
    fn should_parse_extractablepair(){
        let text=r#""name":name"#;
        let a=ExtractablePair::parser(text);
        assert_if(text,a,ExtractablePair::WithKeyValue(format!("name"),ExtractableObject::WithVariableReference(VariableReferenceName::from("name"))));
    }

    #[test]
    fn should_parse_extractablemapobject(){
        let text=r#"{"name":var_name,"place":var_place}"#;
        let a=ExtractableMapObject::parser(text);
        assert_if(text,a,ExtractableMapObject::WithPairs(vec![
            ExtractablePair::WithKeyValue(format!("name"),ExtractableObject::WithVariableReference(VariableReferenceName::from("var_name"))),
            ExtractablePair::WithKeyValue(format!("place"),ExtractableObject::WithVariableReference(VariableReferenceName::from("var_place")))
        ]));
    }
    #[test]
    fn should_parse_extractableforloop(){
        let text=r#"names.for ( name ) => name"#;
        let a=ExtractableForLoop::parser(text);
        assert_if(text,a,ExtractableForLoop{
            on:VariableReferenceName::from("names"),
            with:VariableReferenceName::from("name"),
            inner:ExtractableObject::WithVariableReference(VariableReferenceName::from("name"))
        });
    }
    #[test]
    fn should_parse_vec_extractableobject(){
        let text=r#"[ name , place ]"#;
        let a=Vec::<ExtractableObject>::parser(text);
        assert_if(text,a,vec![
            ExtractableObject::WithVariableReference(VariableReferenceName::from("name")),
            ExtractableObject::WithVariableReference(VariableReferenceName::from("place")),
        ]);
    }
    #[test]
    fn should_parse_extractableobjectoptions_when_variablereference(){
        let text=r#"name"#;
        let a=extractable_object_options(text);
        assert_if(text,a,ExtractableObject::WithVariableReference(VariableReferenceName::from("name")));
    }

    #[test]
    fn should_parse_extractableobjectoptions_when_extractablemapobject(){
        let text=r#" { "name" : var_name , "place" : var_place } "#;
        let a=extractable_object_options(text);
        assert_if(text,a,ExtractableObject::WithMapObject(ExtractableMapObject::WithPairs(vec![
            ExtractablePair::WithKeyValue(format!("name"),ExtractableObject::WithVariableReference(VariableReferenceName::from("var_name"))),
            ExtractablePair::WithKeyValue(format!("place"),ExtractableObject::WithVariableReference(VariableReferenceName::from("var_place")))
        ])));
    }

    #[test]
    fn should_parse_extractableobjectoptions_when_extractableforloop(){
        let text=r#"names.for ( name ) => name"#;
        let a=extractable_object_options(text);
        assert_if(text,a,ExtractableObject::WithForLoop(Box::new(ExtractableForLoop{
            on:VariableReferenceName::from("names"),
            with:VariableReferenceName::from("name"),
            inner:ExtractableObject::WithVariableReference(VariableReferenceName::from("name"))
        })));
    }

    #[test]
    fn should_parse_extractableobjectoptions_when_fixedarray(){
        let text=r#"[ name , place ]"#;
        let a=extractable_object_options(text);
        assert_if(text,a,ExtractableObject::WithFixedArray(vec![
            ExtractableObject::WithVariableReference(VariableReferenceName::from("name")),
            ExtractableObject::WithVariableReference(VariableReferenceName::from("place")),
        ]));
    }

    #[test]
    fn should_parse_extractableobject(){
        let text=r#"object { "name" : var_name , "place" : var_place } "#;
        let a=ExtractableObject::parser(text);
        assert_if(text,a,ExtractableObject::WithMapObject(ExtractableMapObject::WithPairs(vec![
            ExtractablePair::WithKeyValue(format!("name"),ExtractableObject::WithVariableReference(VariableReferenceName::from("var_name"))),
            ExtractablePair::WithKeyValue(format!("place"),ExtractableObject::WithVariableReference(VariableReferenceName::from("var_place")))
        ])));
    }

}