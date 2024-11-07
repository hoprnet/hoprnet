use alloc::alloc::{dealloc, handle_alloc_error, Layout};
use cordyceps::{
    mpsc_queue::{Links, TryDequeueError},
    Linked, MpscQueue,
};
use core::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::{self, drop_in_place, NonNull},
    sync::atomic::{self, AtomicBool, AtomicUsize, Ordering},
    task::Waker,
};
use diatomic_waker::primitives::DiatomicWaker;

/// [`ArcSlice`] is a fun optimisation. For `FuturesUnorderedBounded`, we have `n` slots for futures,
/// and we create a separate context when polling each individual future to avoid having n^2 polling.
///
/// Originally, we pre-allocated `n` `Arc<Wake>` types and stored those along side the future slots.
/// These wakers would have a `Weak` pointing to some shared state, as well as an index for which slot this
/// waker is associated with.
///
/// [`RawWaker`] only gives us 1 pointer worth of data to play with - but we need 2. So unfortunately we needed
/// the extra allocations here
///
/// ... unless we hack around a little bit!
///
/// [`ArcSlice`] represents the shared state, as well as having a long tail of indices. The layout is as follows
/// ```text
/// [ strong_count | waker | head | tail | len | slot 0 | slot 1 | slot 2 | slot 3 | ... ]
/// ```
///
/// [`ArcSlot`] represents our [`RawWaker`]. It points to one of the numbers in the list.
/// Since the layouts of the internals are fixed (`repr(C)`) - we can count back from the index to find
/// the shared data.
///
/// For example, if we have an `ArcSlot` pointing at the number 2, we can count back to pointer 2 `usize`s + [`ArcSliceInnerMeta`] to find
/// the start of the [`ArcSlice`], and then we can insert `2` into the list of futures to poll, finally calling `waker.wake()`.
///
/// Each slot also forms part of a linked list.
pub(crate) struct ArcSlice {
    ptr: NonNull<ArcSliceInnerMeta>,
    phantom: PhantomData<ArcSliceInner>,
}

// This is repr(C) to future-proof against possible field-reordering, which
// would interfere with otherwise safe [into|from]_raw() of transmutable
// inner types.
#[repr(C)]
pub(crate) struct ArcSliceInner {
    meta: ArcSliceInnerMeta,
    slice: [ArcSlotInner],
}

pub(crate) struct ArcSliceInnerMeta {
    strong: AtomicUsize,
    waker: DiatomicWaker,
    len: usize,
    queue: MpscQueue<ArcSlotInner>,
}

// This is repr(C) to future-proof against possible field-reordering, which
// would interfere with otherwise safe [into|from]_raw() of transmutable
// inner types.
#[repr(C)]
pub(crate) struct ArcSlotInner {
    index: usize,
    wake_lock: AtomicBool,
    links: Links<Self>,
}

unsafe impl Linked<Links<Self>> for ArcSlotInner {
    type Handle = NonNull<Self>;

    fn into_ptr(r: Self::Handle) -> NonNull<Self> {
        r
    }

    unsafe fn from_ptr(ptr: NonNull<Self>) -> Self::Handle {
        ptr
    }

    unsafe fn links(ptr: NonNull<Self>) -> NonNull<Links<Self>> {
        NonNull::new_unchecked(ptr::addr_of_mut!((*ptr.as_ptr()).links))
    }
}

const fn __assert_send_sync<T: Send + Sync>() {}
const _: () = {
    // SyncUnsafeCell :ferrisPlead:
    // __assert_send_sync::<ArcSliceInnerMeta>();
    __assert_send_sync::<ArcSlotInner>();

    // SAFETY: The contents of the ArcSlice are Send+Sync
    unsafe impl Send for ArcSlice {}
    unsafe impl Sync for ArcSlice {}
};

impl ArcSlice {
    fn slice_start(&self) -> *mut ArcSlotInner {
        unsafe {
            let ptr = self.ptr.as_ptr().cast::<u8>();
            ptr.add(slice_offset()).cast::<ArcSlotInner>()
        }
    }

    /// The push function from the 1024cores intrusive MPSC queue algorithm.
    /// <https://www.1024cores.net/home/lock-free-algorithms/queues/intrusive-mpsc-node-based-queue>
    ///
    /// Safety: index must be within capacity
    pub(crate) unsafe fn push(&self, index: usize) {
        let queue = &*ptr::addr_of!((*self.ptr.as_ptr()).queue);
        let slot = self.slice_start().add(index);

        queue.enqueue(NonNull::new_unchecked(slot));
    }

    /// Register the waker
    pub(crate) fn register(&mut self, waker: &Waker) {
        // Safety:
        // Diatomic waker requires we do not concurrently run
        // "register", "unregister", and "wait_until".
        // we only call register with mut access, thus we are safe.
        let meta = unsafe { &*self.ptr.as_ptr() };
        unsafe { meta.waker.register(waker) }
    }

    fn get(&self, index: usize) -> ManuallyDrop<Waker> {
        // SAFETY: This cannot go through Deref::deref or RcBoxPtr::inner because
        // this is required to retain raw/mut provenance such that e.g. `get_mut` can
        // write through the pointer after the Rc is recovered through `from_raw`.
        let slot = unsafe { self.slice_start().add(index) };

        debug_assert_eq!(
            unsafe { (*slot).index },
            index,
            "the slot should point at our index"
        );
        slot::waker(slot)
    }

    /// The pop function from the 1024cores intrusive MPSC queue algorithm
    ///
    /// Note that this is unsafe as it required mutual exclusion (only one
    /// thread can call this) to be guaranteed elsewhere.
    pub(crate) unsafe fn pop(&self) -> ReadySlot<(usize, ManuallyDrop<Waker>)> {
        let queue = &*ptr::addr_of!((*self.ptr.as_ptr()).queue);
        match queue.try_dequeue_unchecked() {
            Ok(slot) => {
                (*slot.as_ptr()).wake_lock.store(false, Ordering::SeqCst);
                let i = (*slot.as_ptr()).index;
                ReadySlot::Ready((i, self.get(i)))
            }
            Err(TryDequeueError::Inconsistent) => ReadySlot::Inconsistent,
            Err(TryDequeueError::Empty) => ReadySlot::None,
            Err(TryDequeueError::Busy) => unreachable!(),
        }
    }
}

pub(crate) enum ReadySlot<T> {
    Ready(T),
    Inconsistent,
    None,
}

mod slot {
    use core::{
        mem::ManuallyDrop,
        ptr::NonNull,
        sync::atomic::Ordering,
        task::{RawWaker, RawWakerVTable, Waker},
    };

    use super::{slice_offset, ArcSliceInnerMeta, ArcSlotInner};

    /// Traverses back the [`ArcSlice`] to find the [`ArcSliceInnerMeta`] pointer
    ///
    /// # Safety:
    /// `ptr` must be from an `ArcSlot` originally
    unsafe fn meta_raw(ptr: *mut ArcSlotInner) -> *mut ArcSliceInnerMeta {
        let index = (*ptr).index;
        let slice_start = ptr.sub(index);

        unsafe { slice_start.cast::<u8>().sub(slice_offset()) }.cast::<ArcSliceInnerMeta>()
    }

    /// Traverses back the [`ArcSlice`] to find the [`ArcSliceInnerMeta`] pointer
    ///
    /// # Safety:
    /// * `ptr` must be from an `ArcSlot` originally
    /// * The original `ArcSlot` must outlive `'a`
    unsafe fn meta_ref<'a>(ptr: *const ArcSlotInner) -> &'a ArcSliceInnerMeta {
        unsafe { &*meta_raw(ptr.cast_mut()) }
    }

    pub(super) fn waker(ptr: *const ArcSlotInner) -> ManuallyDrop<Waker> {
        static VTABLE: &RawWakerVTable =
            &RawWakerVTable::new(clone_waker, wake, wake_by_ref, drop_waker);

        // Increment the reference count of the arc to clone it.
        unsafe fn clone_waker(waker: *const ()) -> RawWaker {
            meta_ref(waker.cast()).inc_strong();
            RawWaker::new(waker, VTABLE)
        }

        // We don't need ownership. Just wake_by_ref and drop the waker
        unsafe fn wake(waker: *const ()) {
            wake_by_ref(waker);
            drop_waker(waker);
        }

        // Find the `ArcSliceInnerMeta` and push the current index value into it,
        // then call the stored waker to trigger a poll
        unsafe fn wake_by_ref(waker: *const ()) {
            let slot = waker.cast::<ArcSlotInner>();

            let node = &*slot;
            // node.woken.store(true, Ordering::Relaxed);
            let prev = node.wake_lock.swap(true, Ordering::SeqCst);

            if !prev {
                let meta = meta_ref(slot);
                meta.queue.enqueue(NonNull::new_unchecked(slot.cast_mut()));
                meta.waker.notify();
            }
        }

        // Decrement the reference count of the Arc on drop
        unsafe fn drop_waker(waker: *const ()) {
            let meta = meta_ref(waker.cast());
            if meta.dec_strong() {
                unsafe {
                    super::drop_inner(meta_raw(waker.cast::<ArcSlotInner>().cast_mut()), meta.len);
                }
            }
        }

        let raw_waker = RawWaker::new(ptr.cast(), VTABLE);
        unsafe { ManuallyDrop::new(Waker::from_raw(raw_waker)) }
    }
}

impl ArcSliceInnerMeta {
    fn inc_strong(&self) {
        // Using a relaxed ordering is alright here, as knowledge of the
        // original reference prevents other threads from erroneously deleting
        // the object.
        //
        // As explained in the [Boost documentation][1], Increasing the
        // reference counter can always be done with memory_order_relaxed: New
        // references to an object can only be formed from an existing
        // reference, and passing an existing reference from one thread to
        // another must already provide any required synchronization.
        //
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        let old_size = self.strong.fetch_add(1, Ordering::Relaxed);

        // However we need to guard against massive refcounts in case someone is `mem::forget`ing
        // Arcs. If we don't do this the count can overflow and users will use-after free. This
        // branch will never be taken in any realistic program. We abort because such a program is
        // incredibly degenerate, and we don't care to support it.
        //
        // This check is not 100% water-proof: we error when the refcount grows beyond `isize::MAX`.
        // But we do that check *after* having done the increment, so there is a chance here that
        // the worst already happened and we actually do overflow the `usize` counter. However, that
        // requires the counter to grow from `isize::MAX` to `usize::MAX` between the increment
        // above and the `abort` below, which seems exceedingly unlikely.
        if old_size > (isize::MAX) as usize {
            abort("too many arc clones");
        }
    }
    fn dec_strong(&self) -> bool {
        // Because `fetch_sub` is already atomic, we do not need to synchronize
        // with other threads unless we are going to delete the object. This
        // same logic applies to the below `fetch_sub` to the `weak` count.
        let old_size = self.strong.fetch_sub(1, Ordering::Release);
        if old_size != 1 {
            return false;
        }

        // This fence is needed to prevent reordering of use of the data and
        // deletion of the data.  Because it is marked `Release`, the decreasing
        // of the reference count synchronizes with this `Acquire` fence. This
        // means that use of the data happens before decreasing the reference
        // count, which happens before this fence, which happens before the
        // deletion of the data.
        //
        // As explained in the [Boost documentation][1],
        //
        // > It is important to enforce any possible access to the object in one
        // > thread (through an existing reference) to *happen before* deleting
        // > the object in a different thread. This is achieved by a "release"
        // > operation after dropping a reference (any access to the object
        // > through this reference must obviously happened before), and an
        // > "acquire" operation before deleting the object.
        //
        // In particular, while the contents of an Arc are usually immutable, it's
        // possible to have interior writes to something like a Mutex<T>. Since a
        // Mutex is not acquired when it is deleted, we can't rely on its
        // synchronization logic to make writes in thread A visible to a destructor
        // running in thread B.
        //
        // Also note that the Acquire fence here could probably be replaced with an
        // Acquire load, which could improve performance in highly-contended
        // situations. See [2].
        //
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        // [2]: (https://github.com/rust-lang/rust/pull/41714)
        atomic::fence(Ordering::Acquire);
        true
    }
}

fn slice_offset() -> usize {
    fn padding_needed_for(layout: &Layout, align: usize) -> usize {
        let len = layout.size();

        // Rounded up value is:
        //   len_rounded_up = (len + align - 1) & !(align - 1);
        // and then we return the padding difference: `len_rounded_up - len`.
        //
        // We use modular arithmetic throughout:
        //
        // 1. align is guaranteed to be > 0, so align - 1 is always
        //    valid.
        //
        // 2. `len + align - 1` can overflow by at most `align - 1`,
        //    so the &-mask with `!(align - 1)` will ensure that in the
        //    case of overflow, `len_rounded_up` will itself be 0.
        //    Thus the returned padding, when added to `len`, yields 0,
        //    which trivially satisfies the alignment `align`.
        //
        // (Of course, attempts to allocate blocks of memory whose
        // size and padding overflow in the above manner should cause
        // the allocator to yield an error anyway.)

        let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
        len_rounded_up.wrapping_sub(len)
    }

    let layout = Layout::new::<ArcSliceInnerMeta>();
    layout.size() + padding_needed_for(&layout, core::mem::align_of::<ArcSlotInner>())
}

/// Drops the internals of the [`ArcSlice`].
///
/// # Safety:
/// The pointer must point to a currently allocated [`ArcSlice`].
unsafe fn drop_inner(p: *mut ArcSliceInnerMeta, capacity: usize) {
    let layout = ArcSlice::layout(capacity);

    // SAFETY: the pointer points to an aligned and init instance of `ArcSliceInnerMeta`
    drop_in_place(p);

    // SAFETY: this pointer has been allocated in the global allocator with the given layout
    dealloc(p.cast(), layout);
}

impl Drop for ArcSlice {
    fn drop(&mut self) {
        let meta = unsafe { &*self.ptr.as_ptr() };
        if meta.dec_strong() {
            unsafe { drop_inner(self.ptr.as_ptr().cast(), meta.len) }
        }
    }
}

impl ArcSlice {
    /// Allocates an `ArcInner<T>` with sufficient space for
    /// a possibly-unsized inner value where the value has the layout provided.
    pub(crate) fn new(cap: usize) -> Self {
        // code taken and modified from `Arc::allocate_for_layout`

        let arc_slice_layout = Self::layout(cap);

        // safety: layout size is > 0 because it has at least 7 usizes
        // in the metadata alone
        debug_assert!(arc_slice_layout.size() > 0);
        let ptr = unsafe { alloc::alloc::alloc(arc_slice_layout) };
        if ptr.is_null() {
            handle_alloc_error(arc_slice_layout)
        }

        // meta should be the first item in the alloc
        let meta = ptr.cast::<ArcSliceInnerMeta>();
        let slice = unsafe { ptr.add(slice_offset()).cast::<ArcSlotInner>() };

        // SAFETY:
        // The inner pointer is allocated and aligned, they just need to be initialised
        unsafe {
            let stub = slice.add(cap);

            for i in 0..cap {
                ptr::write(
                    slice.add(i),
                    ArcSlotInner {
                        index: i,
                        wake_lock: AtomicBool::new(false),
                        links: Links::new(),
                    },
                );
            }
            ptr::write(
                slice.add(cap),
                ArcSlotInner {
                    index: cap,
                    wake_lock: AtomicBool::new(false),
                    links: Links::new_stub(),
                },
            );

            ptr::write(
                meta,
                ArcSliceInnerMeta {
                    strong: AtomicUsize::new(1),
                    len: cap,
                    waker: DiatomicWaker::new(),
                    queue: MpscQueue::new_with_stub(NonNull::new_unchecked(stub)),
                },
            );
        }

        Self {
            ptr: unsafe { NonNull::new_unchecked(meta) },
            phantom: PhantomData,
        }
    }

    fn layout(cap: usize) -> Layout {
        let padded = Layout::new::<ArcSlotInner>().pad_to_align();
        let alloc_size = padded.size().checked_mul(cap + 1).unwrap();
        let slice_layout =
            Layout::from_size_align(alloc_size, Layout::new::<ArcSlotInner>().align()).unwrap();

        Layout::new::<ArcSliceInnerMeta>()
            .extend(slice_layout)
            .unwrap()
            .0
            .pad_to_align()
    }
}

fn abort(s: &str) -> ! {
    struct DoublePanic;

    impl Drop for DoublePanic {
        fn drop(&mut self) {
            panic!("panicking twice to abort the program");
        }
    }

    let _bomb = DoublePanic;
    panic!("{}", s);
}
