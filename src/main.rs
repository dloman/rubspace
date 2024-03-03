use std::os::unix::net::{SocketAddr, UnixStream};
use std::io::Write;

use protobuf::Message;
mod subspace;

fn main() -> std::io::Result<()> {
    let addr = SocketAddr::from_pathname("/tmp/subspace")?;

    let mut socket = match UnixStream::connect_addr(&addr) {
        Ok(sock) => sock,
        Err(e) => {
            println!("Couldn't connect: {e:?}");
            return Err(e)
        }
    };

    let mut request = subspace::Request::new();
    request.mut_init().client_name = "client_name".to_string();
    let mut bytes = request.write_to_bytes().unwrap();

    let mut packet: Vec<u8> = vec![];
    packet.append(&mut bytes.len().to_be_bytes().to_vec());

    packet.append(&mut bytes);

    let num_bytes = packet.len();
    println!("sending {request} for init {num_bytes}");
    socket.write_all(&packet)?;
    //println!("sent {bytes_written} ");
//    let response = subspace::Response::merge_from(server.read())?;
//    println!("{response}");
    Ok(())
}
