pub mod parser;
#[derive( Clone,PartialEq,Debug)]
pub struct WorkLoad{
    pub name:String,
    pub journey:String,
    pub concurrentUsers:usize,
    pub duration:u128,
    pub perUserRampUp:u64,
}
