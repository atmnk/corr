use nom::error::{VerboseError};
use nom::IResult;
use nom::bytes::complete::{take_while, tag, escaped_transform, is_not};
use nom::sequence::{preceded, terminated, pair};
use nom::combinator::{value, map};
use nom::branch::alt;
use nom::multi::many0;
use nom::character::complete::{alphanumeric1, alpha1};
pub trait Parsable:Sized{
    fn parser<'a>(input:&'a str)->ParseResult<'a,Self>;
}
pub type ParseResult<'a,O> = IResult<&'a str,O,VerboseError<&'a str>>;
pub fn sp<'a>(i: &'a str) -> ParseResult<'a, &'a str> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(i)
}
pub fn boolean<'a>(input: &'a str) -> ParseResult<'a, bool> {
    // This is a parser that returns `true` if it sees the string "true", and
    // an error otherwise
    let parse_true = value(true, tag("true"));

    // This is a parser that returns `true` if it sees the string "true", and
    // an error otherwise
    let parse_false = value(false, tag("false"));

    // `alt` combines the two parsers. It returns the result of the first
    // successful parser, or an error
    alt((parse_true, parse_false))(input)
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
pub fn identifier<'a>(input: &'a str) -> ParseResult<'a,String> {
    map(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_"))))),
        |(start,remain)| {
            let mut buf="".to_string();
            buf.push_str(start);
            buf.push_str(remain.join("").as_str());
            return buf
        })(input)
}
#[cfg(test)]
mod tests{
    use crate::parser::parse;
    use nom::error::{convert_error};

    #[test]
    fn test_parse(){
        match parse("true") {
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                // here we use the `convert_error` function, to transform a `VerboseError<&str>`
                // into a printable trace.
                //
                // This will print:
                // verbose errors - `root::<VerboseError>(data)`:
                // 0: at line 2:
                //   "c": { 1"hello" : "world"
                //          ^
                // expected '}', found 1
                //
                // 1: at line 2, in map:
                //   "c": { 1"hello" : "world"
                //        ^
                //
                // 2: at line 0, in map:
                //   { "a" : 42,
                //   ^
                println!(
                    "verbose errors - `root::<VerboseError>(data)`:\n{}",
                    convert_error("hellotrue", e)
                );
            },
            Ok((_input,val)) => {
                assert_eq!(val,true)
            },
            _=>{}
        }
    }
}
