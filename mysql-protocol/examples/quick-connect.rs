use std::error::Error;


use mysql_binlog::{
    conn::Connection,
    packet::{Handshake},
};
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
    let mut conn = Connection::new(client);
    //   let packet = conn.read_packet::<Handshake>().await?;
    let handshake = conn.read_packet::<Handshake>().await?.unwrap();
    println!("Handshake: {{");
    println!("  protocol_version: {:?},", handshake.protocol_version);
    println!("  server_version: {:?},", handshake.server_version);
    println!("  connection_id: {:?},", handshake.connection_id);
    println!("  auth_plugin_data: {:?},", handshake.auth_plugin_data);
    println!("  capability_flags: {:?},", handshake.capability_flags);
    println!("  character_set: {:?},", handshake.character_set);
    println!("  status_flags: {:?},", handshake.status_flags);
    println!("  auth_plugin_name: {:?},", handshake.auth_plugin_name);
    println!("}}");

    Ok(())
}
