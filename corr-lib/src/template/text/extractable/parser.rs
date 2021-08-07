use crate::core::Variable;
use crate::parser::{Parsable, ParseResult, ws};
use nom::branch::alt;
use nom::combinator::{map, opt};
use nom::sequence::tuple;

use nom::bytes::complete::{  tag};
use crate::template::text::extractable::{ ExtractableText};
use crate::template::text::parser::text_block;
use nom::multi::many0;
use nom::sequence::{ terminated, preceded};


impl Parsable for ExtractableText{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            terminated(preceded(
                tuple((ws(tag("text")),tag("`"))),terminated(preceded(
                    tag("<%"),map(Variable::parser,|var|ExtractableText::Single(var))),
                                                             tag("%>"))),
                       tag("`")),
            terminated(preceded(
                tuple((ws(tag("text")),tag("`"))),
                map(tuple((opt(text_block),many0(tuple((
                    terminated(preceded(
                    tag("<%"),Variable::parser),
                               tag("%>")
                    ),text_block))),opt(terminated(preceded(
                    tag("<%"),Variable::parser),
                                                   tag("%>")
                )))),|(first,vars,last)|ExtractableText::Multi(first,vars,last))
            ),
                       tag("`")),
        ))(input)
    }
}
pub fn dynamic_tag<'a>(input: &'a str,tag_value:String) -> ParseResult<'a, String> {
    map(tag(tag_value.as_str()),|val:&str| val.to_string())(input)
}
#[cfg(test)]
mod tests{
    
    use crate::parser::util::assert_if;
    use crate::parser::Parsable;
    
    use crate::core::{Variable};
    use crate::template::text::extractable::ExtractableText;

    #[test]
    fn should_parse_extractable_text(){
        let text=r#"text `Hello`"#;
        let a=ExtractableText::parser(text);
        assert_if(text,a,ExtractableText::Multi(Option::Some(format!("Hello")),vec![],Option::None))
    }
    #[test]
    fn should_parse_extractable_text_with_one_variable(){
        let text=r#"text `<%name%>`"#;
        let a=ExtractableText::parser(text);
        assert_if(text,a,ExtractableText::Single(Variable::new("name")))
    }
    // #[test]
    // fn should_parse_extractable_text_with_multiple_variable(){
    //     let text=r#"text `Hello<%name%>World`"#;
    //     let a=ExtractableText::parser(text);
    //     assert_if(text,a,ExtractableText::Single(Variable::new("name")))
    // }
}