use async_trait::async_trait;
#[async_trait]
pub trait JourneyController{
    async fn write_message(&self,message:String);
    async fn start(&mut self,journies:&Vec<Journey>,filter:String);
    async fn execute(&mut self,journey:Journey);
}
#[derive(Debug, Clone)]
pub struct Journey{
    pub name:String,
}
