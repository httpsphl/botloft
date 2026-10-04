//! The owner's own hands while a bot uses the real mouse and keyboard
//! (spec 24.7): low-level hooks see every input, and what the bot sends
//! comes marked as injected. Any other input is the owner's, and the bot
//! stops at once.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::thread::JoinHandle;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, HHOOK, KBDLLHOOKSTRUCT, LLKHF_INJECTED, LLMHF_INJECTED, MSG,
    MSLLHOOKSTRUCT, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL,
    WH_MOUSE_LL, WM_QUIT,
};

use super::super::DesktopError;

/// Set by the hooks when input that is not the bot's arrives.
static OWNER_MOVED: AtomicBool = AtomicBool::new(false);
/// The tick of the last input the bot sent, and of the owner's last one
/// before it: the owner's idle time leaves the bot's input out (spec 24.8).
pub(super) static BOT_TICK: AtomicU32 = AtomicU32::new(0);
pub(super) static OWNER_TICK: AtomicU32 = AtomicU32::new(0);

unsafe extern "system" fn mouse(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        // SAFETY: for a mouse hook, `lparam` is an MSLLHOOKSTRUCT.
        let event = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        if event.flags & LLMHF_INJECTED == 0 {
            OWNER_MOVED.store(true, Ordering::SeqCst);
        }
    }
    // SAFETY: passes the event on, as every hook must.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

unsafe extern "system" fn keyboard(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        // SAFETY: for a keyboard hook, `lparam` is a KBDLLHOOKSTRUCT.
        let event = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        if event.flags.0 & LLKHF_INJECTED.0 == 0 {
            OWNER_MOVED.store(true, Ordering::SeqCst);
        }
    }
    // SAFETY: passes the event on, as every hook must.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Watches the owner's hands while it lives: the hooks run on a thread of
/// their own, which pumps their messages.
pub struct OwnerWatch {
    thread: Option<JoinHandle<()>>,
    id: u32,
}

impl OwnerWatch {
    pub fn start() -> Result<Self, DesktopError> {
        OWNER_MOVED.store(false, Ordering::SeqCst);
        let (told, started) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            // SAFETY: hooks for this process, removed before the thread ends.
            let hooks = unsafe {
                (
                    SetWindowsHookExW(WH_MOUSE_LL, Some(mouse), None, 0),
                    SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard), None, 0),
                )
            };
            let ok = hooks.0.is_ok() && hooks.1.is_ok();
            // SAFETY: a plain query.
            let id = unsafe { GetCurrentThreadId() };
            let _ = told.send(ok.then_some(id));
            if ok {
                let mut message = MSG::default();
                // SAFETY: this thread's own queue, until WM_QUIT.
                while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {}
            }
            for hook in [hooks.0, hooks.1].into_iter().flatten() {
                let hook: HHOOK = hook;
                // SAFETY: a hook this thread set.
                let _ = unsafe { UnhookWindowsHookEx(hook) };
            }
        });
        match started.recv() {
            Ok(Some(id)) => Ok(Self {
                thread: Some(thread),
                id,
            }),
            _ => {
                let _ = thread.join();
                Err(DesktopError::System(
                    "Botloft could not watch the owner's mouse and keyboard".to_owned(),
                ))
            }
        }
    }

    /// Whether the owner touched the mouse or keyboard since it started.
    pub fn moved(&self) -> bool {
        OWNER_MOVED.load(Ordering::SeqCst)
    }
}

impl Drop for OwnerWatch {
    fn drop(&mut self) {
        // SAFETY: ends the hooks' thread's message loop.
        let _ = unsafe { PostThreadMessageW(self.id, WM_QUIT, WPARAM(0), LPARAM(0)) };
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
