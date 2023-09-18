use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::drop_in_place;
use core::sync::atomic::{AtomicUsize, Ordering::{Acquire, AcqRel}};
use core::task::Waker;

#[derive(Debug)]
pub(crate) struct Inner<T> {
    // This one is easy.
    state: AtomicUsize,
    // This is where it all starts to go a bit wrong.
    value: UnsafeCell<MaybeUninit<T>>,
    // Yes, these are subtly different from the last just to confuse you.
    send: UnsafeCell<MaybeUninit<Waker>>,
    recv: UnsafeCell<MaybeUninit<Waker>>,
}

const CLOSED: usize = 0b1000;
const SEND: usize   = 0b0100;
const RECV: usize   = 0b0010;
const READY: usize  = 0b0001;

impl<T> Inner<T> {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Inner {
            state: AtomicUsize::new(0),
            value: UnsafeCell::new(MaybeUninit::uninit()),
            send: UnsafeCell::new(MaybeUninit::uninit()),
            recv: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    // Gets the current state
    #[inline(always)]
    pub(crate) fn state(&self) -> State { State(self.state.load(Acquire)) }

    // Gets the receiver's waker. You *must* check the state to ensure
    // it is set. This would be unsafe if it were public.
    #[inline(always)]
    pub(crate) fn recv(&self) -> &Waker { // MUST BE SET
        debug_assert!(self.state().recv());
        unsafe { &*(*self.recv.get()).as_ptr() }
    }

    // Sets the receiver's waker.
    #[inline(always)]
    pub(crate) fn set_recv(&self, waker: Waker) -> State {
        let recv = self.recv.get();
        unsafe { (*recv).as_mut_ptr().write(waker) } // !
        State(self.state.fetch_or(RECV, AcqRel))
    }

    // Gets the sender's waker. You *must* check the state to ensure
    // it is set. This would be unsafe if it were public.
    #[inline(always)]
    pub(crate) fn send(&self) -> &Waker {
        debug_assert!(self.state().send());
        unsafe { &*(*self.send.get()).as_ptr() }
    }

    // Sets the sender's waker.
    #[inline(always)]
    pub(crate) fn set_send(&self, waker: Waker) -> State {
        let send = self.send.get();
        unsafe { (*send).as_mut_ptr().write(waker) } // !
        State(self.state.fetch_or(SEND, AcqRel))
    }

    #[inline(always)]
    pub(crate) fn take_value(&self) -> T { // MUST BE SET
        debug_assert!(self.state().ready());
        unsafe { (*self.value.get()).as_ptr().read() }
    }

    #[inline(always)]
    pub(crate) fn set_value(&self, value: T) -> State {
        debug_assert!(!self.state().ready());
        let val = self.value.get();
        unsafe { (*val).as_mut_ptr().write(value) }
        State(self.state.fetch_or(READY, AcqRel))
    }

    #[inline(always)]
    pub(crate) fn close(&self) -> State {
        State(self.state.fetch_or(CLOSED, AcqRel))
    }
}

impl<T> Drop for Inner<T> {
    #[inline(always)]
    fn drop(&mut self) {
        let state = State(*self.state.get_mut());
        // Drop the wakers if they are present
        if state.recv() {
            unsafe { drop_in_place((&mut *self.recv.get()).as_mut_ptr()); }
        }
        if state.send() {
            unsafe { drop_in_place((&mut *self.send.get()).as_mut_ptr()); }
        }
    }
}

unsafe impl<T: Send> Send for Inner<T> {}
unsafe impl<T: Sync> Sync for Inner<T> {}

#[derive(Clone, Copy)]
pub(crate) struct State(usize);

impl State {
    #[inline(always)]
    pub(crate) fn closed(&self) -> bool { (self.0 & CLOSED) == CLOSED }
    #[inline(always)]
    pub(crate) fn ready(&self)  -> bool { (self.0 & READY ) == READY  }
    #[inline(always)]
    pub(crate) fn send(&self)   -> bool { (self.0 & SEND  ) == SEND   }
    #[inline(always)]
    pub(crate) fn recv(&self)   -> bool { (self.0 & RECV  ) == RECV   }
}
