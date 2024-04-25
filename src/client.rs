use std::os::unix::net::{UnixStream, SocketAncillary, AncillaryData};
use std::io::IoSliceMut;
use std::io::Read;
use std::io::Write;
use std::os::fd::RawFd;

use protobuf::Message;

use crate::subspace;
use crate::channel::*;

pub struct PublisherId(pub i32);

pub struct PublisherOptions {
    pub is_local: bool,
    pub is_fixed_size: bool,
    pub channel_options: ChannelOptions,
}

pub struct SubscriberOptions {
    pub channel_options: ChannelOptions,
}

pub struct ClientChannel {
    slot: MessageSlot,
    channel: Channel,
}

impl ClientChannel {
    pub fn new(channel_name: ChannelName, num_slots: NumSlots, opts: PublisherOptions, fds: FileDescriptors) -> ClientChannel {
        ClientChannel{
            channel: Channel::new(channel_name, num_slots, opts.channel_options, fds),
            slot: MessageSlot{..Default::default()},
        }
    }
}

pub struct Publisher {
    client_channel: ClientChannel,
    publisher_id: PublisherId,
    publisher_options: PublisherOptions,
}

pub struct Client {
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

    pub fn create_publisher(&mut self, channel_name: ChannelName, slot_size: SlotSize, num_slots: NumSlots, opts: PublisherOptions) -> Result<Publisher, std::io::Error> {
        let mut request = subspace::Request::new();
        request.set_create_publisher(subspace::CreatePublisherRequest{
            channel_name: channel_name.0.clone(),
            num_slots: num_slots.0,
            slot_size: slot_size.0,
            is_local: opts.is_local,
            is_reliable: opts.channel_options.is_reliable,
            is_bridge: opts.channel_options.is_bridge,
            type_: opts.channel_options.type_name.0,
            ..Default::default()
        });

        self.send(request.write_to_bytes().unwrap()).expect("initialization failed");
        let response = self.read()?.take_create_publisher();
        let fds = self.read_fds()?;

        Ok(Publisher{
            client_channel: ClientChannel::new(
                                channel_name,
                                ChannelId(response.channel_id),
                                num_slots,
                                opts,
                                FileDescriptors{
                                    fds: fds,
                                    buffers: response.buffers,
                                    scb_fd: self.scb_fd,
                                    ccb_fd_index: response.ccb_fd_index,
                                    trigger_fd_index: response.pub_trigger_fd_index,
                                    poll_fd_index: response.pub_poll_fd_index,
                                }),
            publisher_id: PublisherId(response.publisher_id),
            publisher_options: opts,
        })
    }
}
