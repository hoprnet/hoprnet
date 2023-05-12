use futures::task::{self, ArcWake};
use parking_lot::Mutex;
use std::convert::TryFrom;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::time::Duration;
use wasm_bindgen::{JsCast, closure::Closure};

use crate::{Instant, Timer, TimerHandle};

/// Starts a background task, creates a `Timer`, and returns a handle to it.
///
/// > **Note**: Contrary to the original `futures-timer` crate, we don't have
/// >           any `forget()` method, as the task is automatically considered
/// >           as "forgotten".
pub(crate) fn run() -> TimerHandle {
    let timer = Timer::new();
    let handle = timer.handle();
    schedule_callback(Arc::new(Mutex::new(timer)), Duration::new(0, 0));
    handle
}

/// Calls `Window::setTimeout` with the given `Duration`. The callback wakes up the timer and
/// processes everything.
fn schedule_callback(timer: Arc<Mutex<Timer>>, when: Duration) {
    let window = web_sys::window().expect("Unable to access Window");
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        &Closure::once_into_js(move || {
            let mut timer_lock = timer.lock();

            // We start by polling the timer. If any new `Delay` is created, the waker will be used
            // to wake up this task pre-emptively. As such, we pass a `Waker` that calls
            // `schedule_callback` with a delay of `0`.
            let waker = task::waker(Arc::new(Waker { timer: timer.clone() }));
            let _ = Future::poll(Pin::new(&mut *timer_lock), &mut Context::from_waker(&waker));

            // Notify the timers that are ready.
            let now = Instant::now();
            timer_lock.advance_to(now);

            // Each call to `schedule_callback` calls `schedule_callback` again, but also leaves
            // the possibility for `schedule_callback` to be called in parallel. Since we don't
            // want too many useless callbacks, we...
            // TODO: ugh, that's a hack
            if Arc::strong_count(&timer) > 20 {
                return;
            }

            // We call `schedule_callback` again for the next event.
            let sleep_dur = timer_lock.next_event()
                .map(|next_event| {
                    if next_event > now {
                        next_event - now
                    } else {
                        Duration::new(0, 0)
                    }
                })
                .unwrap_or(Duration::from_secs(5));
            drop(timer_lock);
            schedule_callback(timer, sleep_dur);

        }).unchecked_ref(),
        i32::try_from(when.as_millis()).unwrap_or(0)
    ).unwrap();
}

struct Waker {
    timer: Arc<Mutex<Timer>>,
}

impl ArcWake for Waker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        schedule_callback(arc_self.timer.clone(), Duration::new(0, 0));
    }
}
