use std::error::Error;


use mysql_protocol::conn::Connection;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();

    let addr = if args.len() >= 2 {
        args[1].to_owned()
    } else {
        "127.0.0.1:3306".to_owned()
    };

    let client = TcpStream::connect(addr).await?;
    let mut conn = Connection::new(client, None);
    conn.connect().await?;

    println!("Connected: {:?}", conn);

    Ok(())
}
