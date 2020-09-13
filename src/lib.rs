#![no_std]

use core::fmt::Write;
use core::marker::PhantomPinned;
use core::mem::{transmute, MaybeUninit};
use core::ops::DerefMut;
use core::panic::PanicInfo;
use core::pin::Pin;
use core::ptr::null_mut;

static mut PANIC_HANDLER_GETTER: Option<unsafe fn(handler: *mut (), info: &PanicInfo)> = None;
static mut PANIC_HANDLER: *mut () = null_mut();

/// Use monomorphization to "save" the type parameter of the static pointer
unsafe fn trampoline<W: Write>(ptr: *mut (), info: &PanicInfo) {
    let handler: &mut PanicHandler<W> = transmute(ptr);

    let _ = write!(handler.deref_mut(), "{}", info);
}

pub struct PanicHandler<W: Write> {
    writer: MaybeUninit<W>,
    _pin: PhantomPinned,
}

impl<W: Write> PanicHandler<W> {
    /// Create a panic handler from a `core::fmt::Write`
    ///
    /// Note that the returned handler is detached when it goes out of scope so in most cases it's
    /// desired to keep the handler in scope for the full duration of the program.
    ///
    /// Additionally, the panic handler implements `Deref` for the provided `Write` and can be used
    /// in place of the original `Write` throughout the app.
    #[must_use = "the panic handler must be kept in scope"]
    pub fn new(writer: W) -> Pin<Self> {
        let handler = unsafe {
            Pin::new_unchecked(PanicHandler {
                writer: MaybeUninit::new(writer),
                _pin: PhantomPinned,
            })
        };
        unsafe {
            PANIC_HANDLER_GETTER = Some(trampoline::<W>);
            PANIC_HANDLER = transmute(&handler);
        }
        handler
    }

    /// Detach this panic handler and return the underlying writer
    pub fn detach(handler: Pin<Self>) -> W {
        unsafe {
            PANIC_HANDLER_GETTER = None;
            PANIC_HANDLER = null_mut();

            // unpin is safe because the pointer to the handler is removed
            let mut handler = Pin::into_inner_unchecked(handler);
            let writer = core::mem::replace(&mut handler.writer, MaybeUninit::uninit());

            // safe because self.writer is only uninit during drop
            writer.assume_init()
        }
    }
}

impl<W: Write> Drop for PanicHandler<W> {
    fn drop(&mut self) {
        unsafe {
            PANIC_HANDLER_GETTER = None;
            PANIC_HANDLER = null_mut();
        }
    }
}

impl<W: Write> core::ops::Deref for PanicHandler<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        // safe because self.writer is only uninit during drop
        unsafe { &*self.writer.as_ptr() }
    }
}

impl<W: Write> core::ops::DerefMut for PanicHandler<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // safe because self.writer is only uninit during drop
        unsafe { &mut *self.writer.as_mut_ptr() }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        if let Some(trampoline) = PANIC_HANDLER_GETTER {
            trampoline(PANIC_HANDLER, info);
        }
    }
    loop {}
}
