#![feature(unix_socket_ancillary_data)]
use std::os::unix::net::{UnixStream, SocketAncillary, AncillaryData};
use std::io::IoSliceMut;
use std::io::Read;
use std::io::Write;
use std::os::fd::RawFd;

use protobuf::Message;
mod subspace;

struct ClientChannel {
}

struct Client {
    name: String,
    socket: UnixStream,
    scb_fd: RawFd,
    buffer: Vec<u8>,
    channels: Vec<ClientChannel>,
}

impl Client {

    pub fn new(server_socket : Option<String>, name: Option<String>) -> Result<Client, std::io::Error> {
        let mut client = Client{
            name: name.unwrap_or("".to_string()),
            socket: UnixStream::connect(server_socket.unwrap_or("/tmp/subspace".to_string()))?,
            scb_fd: -1,
            buffer: vec![],
            channels: vec![],
        };
        client.init()?;
        return Ok(client);
    }


    fn send(&mut self, mut data : Vec<u8>) -> std::io::Result<()> {
        let mut packet: Vec<u8> = vec![];
        let num_bytes:u32 = data.len().try_into().unwrap();
        packet.append(&mut num_bytes.to_be_bytes().to_vec());
        packet.append(&mut data);
        self.socket.write_all(&packet)
    }

    fn read(&mut self) -> Result<subspace::Response, protobuf::Error> {
        let mut buffer = [0; 4];
        self.socket.read_exact(&mut buffer)?;

        let response_length = u32::from_be_bytes(buffer);

        self.buffer.resize(response_length.try_into().unwrap(), 0u8);
        self.socket.read_exact(self.buffer.as_mut_slice())?;
        return Message::parse_from_bytes(&self.buffer);
    }

    fn read_fds(&mut self) -> Result<Vec<RawFd>, std::io::Error> {
        let mut fds: Vec<RawFd> = vec![];
        // wtf are these buffers for?
        let mut buf1 = [1; 8];
        let bufs = &mut [
            IoSliceMut::new(&mut buf1),
        ][..];

        let mut ancillary_buffer = [0; 100];
        let mut ancillary = SocketAncillary::new(&mut ancillary_buffer[..]);
        let _ = self.socket.recv_vectored_with_ancillary(bufs, &mut ancillary)?;
        for ancillary_result in ancillary.messages() {
            if let AncillaryData::ScmRights(scm_rights) = ancillary_result.unwrap() {
                for fd in scm_rights {
                    fds.push(fd);
                }
            }
        }
        Ok(fds)
    }

    fn init(&mut self) -> std::io::Result<()> {
        let mut request = subspace::Request::new();
        request.mut_init().client_name = "client_name".to_string();
        self.send(request.write_to_bytes().unwrap()).expect("initialization failed");

        self.read().expect("error reading socket response");
        self.scb_fd = *self.read_fds()?.first().unwrap();

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let _client = Client::new(None, None);
    Ok(())
}
