pub mod runtime;
pub mod io;
use std::result::Result;

pub fn break_on(path:String,chr:char)->Option<(String,String)>{
    let spl:Vec<&str>=path.rsplitn(2,chr).collect();
    if spl.len() == 2{
        Option::Some((spl[1].to_string(),spl[0].to_string()))
    }
    else {
        Option::None
    }

}
#[cfg(test)]
mod tests{
    use crate::break_on;

    #[test]
    fn should_split_on_dot_if_possible_when_one_dot() {
        let (left,right) = break_on(format!("abc.pqr"),'.').unwrap();
        assert_eq!(left,format!("abc"));
        assert_eq!(right,format!("pqr"))
    }
    #[test]
    fn should_split_on_dot_if_possible_when_multiple_dot() {
        let (left,right) = break_on(format!("abc.pqr.xyz"),'.').unwrap();
        assert_eq!(left,format!("abc.pqr"));
        assert_eq!(right,format!("xyz"))
    }
}