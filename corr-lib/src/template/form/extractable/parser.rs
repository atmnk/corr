use crate::parser::{Parsable, ParseResult, ws};
use nom::combinator::map;
use crate::template::VariableReferenceName;
use nom::sequence::{preceded, terminated, tuple};
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::multi::{separated_list0};
use crate::core::parser::string;
use crate::template::form::extractable::ExtractableForm;

impl Parsable for ExtractableForm{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        preceded(ws(tag("form")),extractable_form_options)(input)
    }
}
fn extractable_form_options<'a>(input: &'a str) -> ParseResult<'a, ExtractableForm> {
    map(preceded(ws(char('{')),terminated(separated_list0(ws(char(',')),
        tuple((terminated(ws(string),ws(char(':'))),ws(VariableReferenceName::parser)))
    ),ws(char('}')))),|pairs|ExtractableForm::WithFields(pairs))(input)
    // map(preceded(ws(char('{')),terminated(separated_list0(ws(char(',')),tuple((ws(string),preceded(ws(tag(":")),ws(VariableReferenceName::parser))))) ,ws(char('}')))),|pairs|ExtractableForm::WithFields(pairs))(input)
}
#[cfg(test)]
mod tests{
    use crate::template::form::extractable::ExtractableForm;
    use crate::parser::Parsable;
    use crate::parser::util::assert_if;
    use crate::template::VariableReferenceName;

    #[test]
    fn should_parse_extractable_form(){
        let text=r#"form {"name":name }"#;
        let a= ExtractableForm::parser(text);
        assert_if(text, a, ExtractableForm::WithFields(vec![("name".to_string(),VariableReferenceName::from("name"))]));
    }


}
