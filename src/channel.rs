use bit_set::BitSet;
use std::os::fd::RawFd;

use crate::subspace;

pub struct SlotSize(pub i32);
pub struct NumSlots(pub i32);
pub struct ChannelName(pub String);
#[derive(Default)]
pub struct TypeName(pub Vec<u8>);
pub struct ChannelId(pub i32);

struct MessagePrefix {
    padding: i32, // Padding for Socket::SendMessage.
    message_size: i32,
    ordinal: i64,
    timestamp: u64,
    flags: u64,
}
// Flag for flags field in MessagePrefix.
const K_MESSAGE_ACTIVATE: i32  = 1; // This is a reliable activation message.
const K_MESSAGE_BRIDGE: i32  = 2;  // This message came from the bridge.
const K_MESSAGE_SEE: i32  = 4;     // Message has been seen.

// We need a max channels number because the size of things in
// shared memory needs to be fixed.
const K_MAX_CHANNELS: usize  = 1024;

// Maximum number of owners for a lot.  One per subscriber reference
// and publisher reference.  Best if it's a multiple of 64 because
// it's used as the size in a BitSet.
const K_MAX_SLOT_OWNERS: i32 = 1024;

// Max length of a channel name in shared memory.  A name longer
// this this will be truncated but the full name will be available
// in process memory.
const K_MAX_CHANNEL_NAME : usize = 64;

// Message slots are held in a double linked list, each element of
// which is a SlotListElement (embedded at offset 0 in the MessageSlot
// struct in shared memory).  The linked lists do not use pointers
// because this is in shared memory mapped at different virtual
// addresses in each client.  Instead they use an offset from the
// start of the ChannelControlBlock (CCB) as a pointer.
#[derive(Default)]
pub struct SlotListElement {
    prev: i32,
    next: i32,
}

//Double linked list header in shared memory
pub struct SlotList {
    first: i32,
    last: i32,
}

//TODO remove default
#[derive(Default)]
pub struct MessageSlot {
    element: SlotListElement,
    id: i32,                 // Unique ID for slot (0...num_slots-1).
    ref_count: i16,          // Number of subscribers referring to this slot.
    reliable_ref_count: i16, // Number of reliable subscriber references.
    ordinal: i64,            // Message ordinal held currently in slot.
    message_size: i64,       // Size of message held in slot.
    buffer_index: i32,       // Index of buffer.
    owners: BitSet,          // One bit per publisher/subscriber.
}

// This is a global (to a server) structure in shared memory that holds
// counts for the number of updates to publishers and subscribers on
// that server.  The server updates these counts when a publisher or
// subscriber is created or deleted.  The purpose is to allow a client
// to check if it needs to update its local information about the
// channel by contacting the server.  Things such as trigger file
// descriptors are distributed by the server to clients.
//
// This is in shared memory, but it is only ever written by the
// server so there is no lock required to access it in the clients.
pub struct ChannelCounters {
  num_pub_updates: u16,
  num_sub_updates: u16,
  num_pubs: u16,
  num_reliable: u16,
  num_subs: u16,
  num_reliable_subs: u16,
}

pub struct ChannelControlBlock {
    channel_name: [char; K_MAX_CHANNEL_NAME],
    num_slots: NumSlots,
    next_ordinal: i64, // Next ordinal to use.
    buffer_index: i32,         // Which buffer in buffers array to use.
    num_buffers: i32,          // Size of buffers array in shared memory.
    // Statistics counters.
    total_bytes: i64,
    total_messages: i64,

    // Slot lists.
    // Active list: slots with active messages in them.
    // Busy list: slots allocated to publishers
    // Free list: slots not allocated.
    active_list: SlotList,
    busy_list: SlotList,
    free_list: SlotList,
}

pub struct SystemControlBlock {
    counters: [ChannelCounters; K_MAX_CHANNELS],
}

pub struct BufferSet {
    slot_size: SlotSize,
    buffer: Vec<char>,
}

pub struct ChannelOptions {
    pub is_reliable: bool,
    pub is_bridge: bool,
    pub type_name: TypeName,
    pub is_debug: bool,
}

struct SlotBuffer {
    slot_size: SlotSize,
    fd: RawFd,
}

struct FileDescriptors {
    buffer_infos: Vec<subspace::BufferInfo>,
    fds: Vec<RawFd>,
    scb_fd: RawFd
    trigger_fd_index: i32,
    poll_fd_index: i32,
}

pub struct Channel {
    pub name: ChannelName,
    pub num_slots: NumSlots,
    pub channel_id: ChannelId,
    pub type_name: TypeName,
    pub num_updates: i16,
    pub scb: SystemControlBlock,
    pub ccb: ChannelControlBlock,
    pub buffers: Vec<BufferSet>,
    pub is_debug: bool,
}

struct SharedMemory {
}

impl Channel {
    pub fn new(channel_name: ChannelName, channel_id: ChannelId, num_slots: NumSlots, opts: ChannelOptions, fds: FileDescriptors)-> Channel {
        Channel{
            name: channel_name,
            num_slots: num_slots,
            type_name: opts.type_name,
            num_updates: 0,
            is_debug: opts.is_debug,
            scb: ,
            ccb: ,
            buffers: ,
        }
    }

    fn MapSharedMemory(fds: FileDescriptors) -> SharedMemory {
        let buffers = Vec::with_capacity(fds.buffer_infos.len());
        for info in buffer_infos.into_iter() {
            buffers.push(SlotBuffer{ slot_size: info.slot_size, fd: fds[.fd_index]});
        }

    }
}
