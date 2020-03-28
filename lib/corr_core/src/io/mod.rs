use std::io;
pub trait StringIO {
    fn write(&mut self,value:String);
    fn read_raw(&mut self)->String;
    fn read(&mut self)->String{
        sanitize(self.read_raw())
    }
}
fn sanitize(mut line:String)->String {
    line.truncate(line.trim_end().len());
    line
}
pub struct StdStringIO {
}
impl StringIO for StdStringIO {
    fn write(&mut self,value: String) {
        println!("{}",value);
    }
    fn read_raw(&mut self) -> String {
        let mut line = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut line).unwrap();
        line.truncate(line.trim_end().len());
        line
    }
}