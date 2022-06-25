#![feature(generators, generator_trait)]
#![feature(async_closure)]
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Result};
use tokio_tungstenite::tungstenite::Message;
use url::Url;
#[tokio::main]
async fn main()->Result<()>{
    let (mut socket, _)=connect_async(Url::parse("ws://localhost:9002").unwrap()).await?;
    let _ = socket.send(Message::Text("{\"name\":\"Atmaram\"}".to_string())).await;
    let msg=socket.next().await.unwrap()?;
    println!("{:?}",msg);
    Ok(())
    // let journey = r#"`PrintCategories`(){
    //     let c_str = "host=localhost user=postgres dbname=dellstore password=postgres port=5436"
    //     let category_table = "categories"
    //     let connection = connect postgres c_str
    //     let sql = text `insert into <%category_table%>(categoryname) values($1)`
    //     categories.for(category)=>{
    //         let category = object [{"value":fake("CompanyName"),"type":"string"}]
    //     }
    //     on connection execute sql with multiple categories
    //     let category = object [{"value":fake("CompanyName"),"type":"string"}]
    //     let query = "select * from categories"
    //     on connection execute sql with category
    //     fetch query on connection matching categories.for(category)=>[{"i32":category.catgory},{"string":category.categoryname}]
    // }
    // "#;
    // let input = vec![ Input::new_continue("categories::length".to_string(),"10".to_string(),DataType::PositiveInteger)];
    // let buffer = Arc::new(Mutex::new(vec![]));
    // let context= Context::mock(input,buffer.clone());
    // let (_,jrn) = Journey::parser(journey).unwrap();
    // jrn.execute(&context).await;
    // let buf= buffer.lock().unwrap();
    // for out in buf.iter() {
    //     println!("{:?}",out)
    // }
    // // let dr = PostgresDriver;
    // // let conn = dr.connect("host=localhost user=postgres dbname=dellstore password=postgres port=5436").await?;
    // // let query = conn.prepare("SELECT * from public.categories where category = 1 order by category ASC").await?;
    // // let mut results=query.execute_query(vec![]).await?;
    // // while results.next().await {
    // //     println!("{}",results.get_string(1).unwrap())
    // // }
    // // Ok(())
}
