#![feature(unix_socket_ancillary_data)]
mod client;
mod channel;
mod subspace;

use crate::client::*;
use crate::channel::*;


fn main() -> std::io::Result<()> {
    let mut client = Client::new(None, None)?;
    let _pub = client.create_publisher(ChannelName("foo".to_string()), SlotSize(32), NumSlots(4), PublisherOptions{..Default::default()})?;
    Ok(())
}
