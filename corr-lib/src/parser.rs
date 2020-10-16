use nom::error::{VerboseError};
use nom::IResult;
use nom::bytes::complete::{take_while, tag, escaped_transform, is_not};
use nom::sequence::{preceded, terminated, pair};
use nom::combinator::{map, verify, recognize, opt};
use nom::branch::alt;
use nom::multi::{ many0_count};
use nom::character::complete::{alphanumeric1, alpha1, char};
use crate::core::parser::boolean;
use nom::error::convert_error;
use crate::{get_keywords};
use crate::template::functions::function_names;

pub trait Parsable:Sized{
    fn parser<'a>(input:&'a str)->ParseResult<'a,Self>;
}
pub type ParseResult<'a,O> = IResult<&'a str,O,VerboseError<&'a str>>;
pub fn sp<'a>(i: &'a str) -> ParseResult<'a, &'a str> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(i)
}
pub fn ws<'a, O, F>(inner: F) -> impl FnMut(&'a str) -> ParseResult<'a, O>
    where
        F: Fn(&'a str) -> ParseResult<'a, O>
{
    preceded(sp,terminated(inner,sp))
}
pub fn parse<'a>(input : &'a str)->ParseResult<'a, bool> {
    ws(boolean)(input)
}

pub fn non_back_quote<'a>(input:&'a str) ->ParseResult<'a,String>{
    map(escaped_transform(is_not("\\`"), '\\', |i: &'a str| alt((tag("`"),tag("\\")))(i)),|val| val.to_string())(input)

}
pub fn identifier_part<'a>(input: &'a str) -> ParseResult<'a,&str> {
    verify(recognize(
        pair(
            alt((alpha1,tag("_"))),
            many0_count(preceded(opt(char('_')),alphanumeric1)))),|val:&&str|{!get_keywords().contains(val)})(input)
}
pub fn function_name<'a>(input: &'a str) -> ParseResult<'a,&str> {
    verify(recognize(
        pair(
            alt((alpha1,tag("_"))),
            many0_count(preceded(opt(char('_')),alphanumeric1)))),|val:&&str|{function_names().contains(val)})(input)
}
pub fn result_option<'a,T>(contents:&str,result:ParseResult<'a,T>)->Option<T>{
    match result {
        Ok((_,val))=>{
            Option::Some(val)
        },
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e))=>{
            eprintln!("{}",readable_error(contents,e));
            Option::None
        },
        _=>{
            Option::None
        }
    }
}
pub fn readable_error(contents:&str,e: VerboseError<&str>)->String{
    return format!("Unable to parse following errors {}",convert_error(contents,e))
}
#[cfg(test)]
pub mod util{
    use crate::parser::ParseResult;
    use nom::error::convert_error;

    pub fn assert_if<'a,T>(text:&'a str, result:ParseResult<'a,T>, to:T) where T:PartialEq+std::fmt::Debug{
        match result {
            Ok((_i,val))=>{
                assert_eq!(val,to)
            },
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e))=>{
                assert!(false,format!("Unable to parse following errors {}",convert_error(text,e)))
            },
            _=>{}
        }
    }
    pub fn assert_no_error<'a,T>(text:&'a str, result:ParseResult<'a,T>) where T:PartialEq+std::fmt::Debug{
        match result {
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e))=>{
                assert!(false,format!("Unable to parse following errors {}",convert_error(text,e)))
            },
            _=>{}
        }
    }
}
#[cfg(test)]
mod tests{
    use crate::parser::function_name;

    #[test]
    fn should_recognize_function_names(){
        let txt = r#"concat"#;
        let (_,name) = function_name(txt).unwrap();
        assert_eq!(name,"concat")
    }
}