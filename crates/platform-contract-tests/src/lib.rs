#![forbid(unsafe_code)]

//! Portable assertions and provider fakes shared by adapter and application tests.

mod privacy;
mod selection;
mod translation;

pub use privacy::assert_error_is_content_free;
pub use selection::{FakeSelectionProvider, assert_selection_contract};
pub use translation::{UnavailableTranslationProvider, assert_translation_contract};

#[cfg(test)]
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    use std::{
        sync::Arc,
        task::{Context, Poll, Wake, Waker},
        thread,
    };

    struct ThreadWake(thread::Thread);

    impl Wake for ThreadWake {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::park(),
        }
    }
}
