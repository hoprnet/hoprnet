#[cfg(test)]
mod test {
    use crate::{suspend, AtomicOwned, AtomicShared, Guard, Owned, Ptr, Shared, Tag};
    use std::ops::Deref;
    use std::panic::UnwindSafe;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::sync::atomic::{AtomicBool, AtomicUsize};

    static_assertions::assert_impl_all!(Shared<String>: Send, Sync, UnwindSafe);
    static_assertions::assert_impl_all!(AtomicShared<String>: Send, Sync, UnwindSafe);
    static_assertions::assert_impl_all!(Ptr<String>: UnwindSafe);
    static_assertions::assert_not_impl_all!(Shared<*const u8>: Send, Sync, UnwindSafe);
    static_assertions::assert_not_impl_all!(AtomicShared<*const u8>: Send, Sync, UnwindSafe);
    static_assertions::assert_not_impl_all!(Ptr<String>: Send, Sync);
    static_assertions::assert_not_impl_all!(Ptr<*const u8>: Send, Sync, UnwindSafe);
    static_assertions::assert_impl_all!(Guard: UnwindSafe);
    static_assertions::assert_not_impl_all!(Guard: Send, Sync);

    struct A(AtomicUsize, usize, &'static AtomicBool);
    impl Drop for A {
        fn drop(&mut self) {
            self.2.swap(true, Relaxed);
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn deferred() {
        static EXECUTED: AtomicBool = AtomicBool::new(false);

        let guard = Guard::new();
        guard.defer_execute(|| EXECUTED.store(true, Relaxed));
        drop(guard);

        while !EXECUTED.load(Relaxed) {
            drop(Guard::new());
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn shared() {
        static DESTROYED: AtomicBool = AtomicBool::new(false);

        let mut shared = Shared::new(A(AtomicUsize::new(10), 10, &DESTROYED));
        if let Some(mut_ref) = unsafe { shared.get_mut() } {
            mut_ref.1 += 1;
        }
        shared.0.fetch_add(1, Relaxed);
        assert_eq!(shared.deref().0.load(Relaxed), 11);
        assert_eq!(shared.deref().1, 11);

        let mut shared_clone = shared.clone();
        assert!(unsafe { shared_clone.get_mut().is_none() });
        shared_clone.0.fetch_add(1, Relaxed);
        assert_eq!(shared_clone.deref().0.load(Relaxed), 12);
        assert_eq!(shared_clone.deref().1, 11);

        let mut shared_clone_again = shared_clone.clone();
        assert!(unsafe { shared_clone_again.get_mut().is_none() });
        assert_eq!(shared_clone_again.deref().0.load(Relaxed), 12);
        assert_eq!(shared_clone_again.deref().1, 11);

        drop(shared);
        assert!(!DESTROYED.load(Relaxed));
        assert!(unsafe { shared_clone_again.get_mut().is_none() });

        drop(shared_clone);
        assert!(!DESTROYED.load(Relaxed));
        assert!(unsafe { shared_clone_again.get_mut().is_some() });

        drop(shared_clone_again);
        while !DESTROYED.load(Relaxed) {
            drop(Guard::new());
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn owned() {
        static DESTROYED: AtomicBool = AtomicBool::new(false);

        let mut owned = Owned::new(A(AtomicUsize::new(10), 10, &DESTROYED));
        unsafe {
            *owned.get_mut().0.get_mut() += 2;
            owned.get_mut().1 += 2;
        }
        assert_eq!(owned.deref().0.load(Relaxed), 12);
        assert_eq!(owned.deref().1, 12);

        let guard = Guard::new();
        let ptr = owned.get_guarded_ptr(&guard);
        assert!(ptr.get_shared().is_none());

        drop(owned);
        assert!(!DESTROYED.load(Relaxed));

        drop(guard);

        while !DESTROYED.load(Relaxed) {
            drop(Guard::new());
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn sendable() {
        static DESTROYED: AtomicBool = AtomicBool::new(false);

        let shared = Shared::new(A(AtomicUsize::new(14), 14, &DESTROYED));
        let owned = Owned::new(A(AtomicUsize::new(15), 15, &DESTROYED));
        let shared_clone = shared.clone();
        let thread = std::thread::spawn(move || {
            assert_eq!(shared_clone.0.load(Relaxed), shared_clone.1);
            assert_eq!(owned.1, 15);
        });
        assert!(thread.join().is_ok());
        assert_eq!(shared.0.load(Relaxed), shared.1);
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn shared_send() {
        static DESTROYED: AtomicBool = AtomicBool::new(false);

        let shared = Shared::new(A(AtomicUsize::new(14), 14, &DESTROYED));
        let shared_clone = shared.clone();
        let thread = std::thread::spawn(move || {
            assert_eq!(shared_clone.0.load(Relaxed), 14);
            unsafe {
                assert!(!shared_clone.drop_in_place());
            }
        });
        assert!(thread.join().is_ok());
        assert_eq!(shared.0.load(Relaxed), 14);

        unsafe {
            assert!(shared.drop_in_place());
        }

        assert!(DESTROYED.load(Relaxed));
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn shared_nested() {
        static DESTROYED: AtomicBool = AtomicBool::new(false);

        let nested_shared = Shared::new(Shared::new(A(AtomicUsize::new(10), 10, &DESTROYED)));
        assert!(!DESTROYED.load(Relaxed));
        drop(nested_shared);

        while !DESTROYED.load(Relaxed) {
            drop(Guard::new());
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn atomic_shared() {
        static DESTROYED: AtomicBool = AtomicBool::new(false);

        let atomic_shared = AtomicShared::new(A(AtomicUsize::new(10), 10, &DESTROYED));
        assert!(!DESTROYED.load(Relaxed));

        let guard = Guard::new();
        let atomic_shared_clone = atomic_shared.clone(Relaxed, &guard);
        assert_eq!(
            atomic_shared_clone
                .load(Relaxed, &guard)
                .as_ref()
                .unwrap()
                .1,
            10
        );

        drop(atomic_shared);
        assert!(!DESTROYED.load(Relaxed));

        atomic_shared_clone.update_tag_if(Tag::Second, |_| true, Relaxed, Relaxed);

        drop(atomic_shared_clone);
        drop(guard);

        while !DESTROYED.load(Relaxed) {
            drop(Guard::new());
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn atomic_owned() {
        static DESTROYED: AtomicBool = AtomicBool::new(false);

        let atomic_owned = AtomicOwned::new(A(AtomicUsize::new(10), 10, &DESTROYED));
        assert!(!DESTROYED.load(Relaxed));

        let guard = Guard::new();
        let ptr = atomic_owned.load(Relaxed, &guard);
        assert_eq!(ptr.as_ref().map(|a| a.1), Some(10));

        atomic_owned.update_tag_if(Tag::Second, |_| true, Relaxed, Relaxed);

        drop(atomic_owned);
        assert_eq!(ptr.as_ref().map(|a| a.1), Some(10));

        drop(guard);

        while !DESTROYED.load(Relaxed) {
            drop(Guard::new());
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn atomic_shared_send() {
        static DESTROYED: AtomicBool = AtomicBool::new(false);

        let atomic_shared = AtomicShared::new(A(AtomicUsize::new(17), 17, &DESTROYED));
        assert!(!DESTROYED.load(Relaxed));

        let thread = std::thread::spawn(move || {
            let guard = Guard::new();
            let ptr = atomic_shared.load(Relaxed, &guard);
            assert_eq!(ptr.as_ref().unwrap().0.load(Relaxed), 17);
        });
        assert!(thread.join().is_ok());

        while !DESTROYED.load(Relaxed) {
            drop(Guard::new());
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn atomic_shared_creation() {
        static DESTROYED: AtomicBool = AtomicBool::new(false);

        let atomic_shared = AtomicShared::new(A(AtomicUsize::new(11), 11, &DESTROYED));
        assert!(!DESTROYED.load(Relaxed));

        let guard = Guard::new();

        let shared = atomic_shared.get_shared(Relaxed, &guard);

        drop(atomic_shared);
        assert!(!DESTROYED.load(Relaxed));

        if let Some(shared) = shared {
            assert_eq!(shared.1, 11);
            assert!(!DESTROYED.load(Relaxed));
        }
        drop(guard);

        while !DESTROYED.load(Relaxed) {
            drop(Guard::new());
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn atomic_shared_conversion() {
        static DESTROYED: AtomicBool = AtomicBool::new(false);

        let atomic_shared = AtomicShared::new(A(AtomicUsize::new(11), 11, &DESTROYED));
        assert!(!DESTROYED.load(Relaxed));

        let guard = Guard::new();

        let shared = atomic_shared.into_shared(Relaxed);
        assert!(!DESTROYED.load(Relaxed));

        if let Some(shared) = shared {
            assert_eq!(shared.1, 11);
            assert!(!DESTROYED.load(Relaxed));
        }
        drop(guard);

        while !DESTROYED.load(Relaxed) {
            drop(Guard::new());
        }
    }

    #[cfg_attr(miri, ignore)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 16)]
    async fn atomic_shared_parallel() {
        let atomic_shared: Shared<AtomicShared<String>> =
            Shared::new(AtomicShared::new(String::from("How are you?")));
        let mut task_handles = Vec::new();
        for _ in 0..16 {
            let atomic_shared = atomic_shared.clone();
            task_handles.push(tokio::task::spawn(async move {
                for _ in 0..64 {
                    let guard = Guard::new();
                    let mut ptr = (*atomic_shared).load(Acquire, &guard);
                    assert!(ptr.tag() == Tag::None || ptr.tag() == Tag::Second);
                    if let Some(str_ref) = ptr.as_ref() {
                        assert!(str_ref == "How are you?" || str_ref == "How can I help you?");
                    }
                    let converted: Result<Shared<String>, _> = Shared::try_from(ptr);
                    if let Ok(shared) = converted {
                        assert!(*shared == "How are you?" || *shared == "How can I help you?");
                    }
                    while let Err((passed, current)) = atomic_shared.compare_exchange(
                        ptr,
                        (
                            Some(Shared::new(String::from("How can I help you?"))),
                            Tag::Second,
                        ),
                        Release,
                        Relaxed,
                        &guard,
                    ) {
                        if let Some(shared) = passed {
                            assert!(*shared == "How can I help you?");
                        }
                        ptr = current;
                        if let Some(str_ref) = ptr.as_ref() {
                            assert!(str_ref == "How are you?" || str_ref == "How can I help you?");
                        }
                        assert!(ptr.tag() == Tag::None || ptr.tag() == Tag::Second);
                    }
                    assert!(!suspend());
                    drop(guard);

                    assert!(suspend());

                    atomic_shared.update_tag_if(Tag::None, |_| true, Relaxed, Relaxed);

                    let guard = Guard::new();
                    ptr = (*atomic_shared).load(Acquire, &guard);
                    assert!(ptr.tag() == Tag::None || ptr.tag() == Tag::Second);
                    if let Some(str_ref) = ptr.as_ref() {
                        assert!(str_ref == "How are you?" || str_ref == "How can I help you?");
                    }
                    drop(guard);

                    let (old, _) = atomic_shared.swap(
                        (Some(Shared::new(String::from("How are you?"))), Tag::Second),
                        Release,
                    );
                    if let Some(shared) = old {
                        assert!(*shared == "How are you?" || *shared == "How can I help you?");
                    }
                }
            }));
        }
        for r in futures::future::join_all(task_handles).await {
            assert!(r.is_ok());
        }
    }

    #[cfg_attr(miri, ignore)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 16)]
    async fn atomic_shared_clone() {
        let atomic_shared: Shared<AtomicShared<String>> =
            Shared::new(AtomicShared::new(String::from("How are you?")));
        let mut task_handles = Vec::new();
        for t in 0..4 {
            let atomic_shared = atomic_shared.clone();
            task_handles.push(tokio::task::spawn(async move {
                for i in 0..256 {
                    if t == 0 {
                        let tag = if i % 3 == 0 {
                            Tag::First
                        } else if i % 2 == 0 {
                            Tag::Second
                        } else {
                            Tag::None
                        };
                        let (old, _) = atomic_shared.swap(
                            (Some(Shared::new(String::from("How are you?"))), tag),
                            Release,
                        );
                        assert!(old.is_some());
                        if let Some(shared) = old {
                            assert!(*shared == "How are you?");
                        }
                    } else {
                        let (shared_clone, _) = (*atomic_shared)
                            .clone(Acquire, &Guard::new())
                            .swap((None, Tag::First), Release);
                        assert!(shared_clone.is_some());
                        if let Some(shared) = shared_clone {
                            assert!(*shared == "How are you?");
                        }
                        let shared_clone = atomic_shared.get_shared(Acquire, &Guard::new());
                        assert!(shared_clone.is_some());
                        if let Some(shared) = shared_clone {
                            assert!(*shared == "How are you?");
                        }
                    }
                }
            }));
        }
        for r in futures::future::join_all(task_handles).await {
            assert!(r.is_ok());
        }
    }
}
