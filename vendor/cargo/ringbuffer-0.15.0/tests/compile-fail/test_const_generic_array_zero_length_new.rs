extern crate ringbuffer;

use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

fn main() {
    let mut buf = ConstGenericRingBuffer::new::<0>();
    //~^ note: the above error was encountered while instantiating `fn ringbuffer::ConstGenericRingBuffer::<i32, 0>::new::<0>`
    // ringbuffer can't be zero length
    buf.push(5);
}
