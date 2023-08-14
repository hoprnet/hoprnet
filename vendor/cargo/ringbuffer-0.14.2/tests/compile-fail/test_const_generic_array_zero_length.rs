extern crate ringbuffer;

use ringbuffer::ConstGenericRingBuffer;

fn main() {
    let _ = ConstGenericRingBuffer::<i32, 0>::new();
    //~^ note: the above error was encountered while instantiating `fn ringbuffer::ConstGenericRingBuffer::<i32, 0>::new`
    // ringbuffer can't be zero length
}