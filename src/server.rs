use std::os::unix::net::UnixListener;
use std::io;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let listener = UnixListener::bind("/tmp/subspace")?;

    loop {
    match listener.accept() {
        Ok((mut socket, addr)) => {
            println!("Got a client: {:?} - {:?}", socket, addr);
            let mut buf = [0; 1024];
            let count = socket.read(&mut buf).unwrap();
            let response = String::from_utf8(buf[..count].to_vec()).unwrap();

            print!("got message {}", response);
         io::stdout().flush() ;

        },
        Err(e) => println!("accept function failed: {:?}", e),
    }
    }
}

