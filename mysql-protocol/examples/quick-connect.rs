use std::error::Error;

use mysql_binlog::{conn::Connection, packet::{Handshake, PacketReader, Raw}};
use mysql_common::packets::{HandshakePacket, parse_handshake_packet};
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
   let packet = conn.read_packet::<Raw>().await?.unwrap();
   let my_packet = Handshake::parse(&packet).unwrap();
   let their_packet = parse_handshake_packet(&packet).unwrap();
    println!("their: {:?}", their_packet);
    println!("my: {:?}", my_packet);

  Ok(())
}
