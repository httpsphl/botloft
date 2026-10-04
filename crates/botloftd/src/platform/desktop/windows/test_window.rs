//! A window of the tests' own, painted red, on a thread with its message
//! loop: a Save button that renames the window, a field, a password field,
//! a ticked check box, a combo box and a long list, on its left side, so
//! its middle stays red.

use std::sync::{Mutex, MutexGuard, Once, PoisonError, mpsc};
use std::thread::JoinHandle;
use std::time::Duration;

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateSolidBrush, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    BM_SETCHECK, BS_AUTOCHECKBOX, BS_PUSHBUTTON, CB_ADDSTRING, CBS_DROPDOWNLIST, CW_USEDEFAULT,
    CreateWindowExW, DefWindowProcW, DispatchMessageW, ES_PASSWORD, GetMessageW, HMENU,
    LB_ADDSTRING, LBS_NOTIFY, MSG, PostMessageW, PostQuitMessage, RegisterClassW, SendMessageW,
    SetWindowTextW, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_COMMAND,
    WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
};
use windows::core::{PCWSTR, w};

const CLASS: PCWSTR = w!("BotloftDesktopTest");
/// The Save button's id: its click renames the window "Saved".
const SAVE: usize = 1;
/// How many items the long list has: more than show at once.
pub const ITEMS: usize = 50;

unsafe extern "system" fn procedure(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_DESTROY {
        // SAFETY: ends this thread's message loop.
        unsafe { PostQuitMessage(0) };
        return LRESULT(0);
    }
    // The low word is the control, the high word what happened: 0, a click.
    if message == WM_COMMAND && wparam.0 & 0xFFFF == SAVE && wparam.0 >> 16 == 0 {
        // SAFETY: renames our own window, on its thread.
        let _ = unsafe { SetWindowTextW(hwnd, w!("Saved")) };
        return LRESULT(0);
    }
    // SAFETY: everything else as Windows does by default.
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

/// Adds `text` to a combo box or list with `message`.
///
/// # Safety
///
/// `control` is a window of this thread.
unsafe fn add(control: HWND, message: u32, text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    // SAFETY: the string lives through the call.
    unsafe { SendMessageW(control, message, None, Some(LPARAM(wide.as_ptr() as isize))) };
}

/// # Safety
///
/// `parent` is a window of this thread.
unsafe fn controls(parent: HWND) {
    let child = |class: PCWSTR, text: PCWSTR, style: u32, id: usize, place: (i32, i32)| {
        // SAFETY: a child of `parent`, on its thread.
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class,
                text,
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(style),
                10,
                place.0,
                160,
                place.1,
                Some(parent),
                Some(HMENU(id as *mut std::ffi::c_void)),
                None,
                None,
            )
            .expect("a control")
        }
    };
    child(
        w!("BUTTON"),
        w!("Save"),
        BS_PUSHBUTTON as u32,
        SAVE,
        (10, 24),
    );
    child(w!("EDIT"), w!("hello"), WS_BORDER.0, 2, (40, 24));
    let password = WS_BORDER.0 | ES_PASSWORD as u32;
    child(w!("EDIT"), w!("secret"), password, 3, (70, 24));
    let check = child(
        w!("BUTTON"),
        w!("Remember"),
        BS_AUTOCHECKBOX as u32,
        4,
        (100, 24),
    );
    // SAFETY: our own controls, on this thread.
    unsafe {
        SendMessageW(check, BM_SETCHECK, Some(WPARAM(1)), None);
        let size = child(
            w!("COMBOBOX"),
            w!(""),
            CBS_DROPDOWNLIST as u32,
            5,
            (130, 120),
        );
        add(size, CB_ADDSTRING, "Small");
        add(size, CB_ADDSTRING, "Large");
        let list = WS_BORDER.0 | WS_VSCROLL.0 | LBS_NOTIFY as u32;
        let items = child(w!("LISTBOX"), w!(""), list, 6, (160, 90));
        for item in 0..ITEMS {
            add(items, LB_ADDSTRING, &format!("Item {item}"));
        }
    }
}

/// One test window at a time: a new window takes the focus, and an open
/// combo box list closes when its window loses it.
static ONE: Mutex<()> = Mutex::new(());

pub struct TestWindow {
    pub id: u64,
    thread: Option<JoinHandle<()>>,
    _one: MutexGuard<'static, ()>,
}

impl TestWindow {
    pub fn open(title: &str) -> Self {
        let one = ONE.lock().unwrap_or_else(PoisonError::into_inner);
        static CLASS_ONCE: Once = Once::new();
        CLASS_ONCE.call_once(|| {
            // SAFETY: a window class with a static name and procedure.
            unsafe {
                let class = WNDCLASSW {
                    lpfnWndProc: Some(procedure),
                    hInstance: GetModuleHandleW(None).expect("module").into(),
                    lpszClassName: CLASS,
                    // Pure red: 0x00BBGGRR.
                    hbrBackground: HBRUSH(CreateSolidBrush(COLORREF(0x0000_00FF)).0),
                    ..Default::default()
                };
                assert_ne!(RegisterClassW(&class), 0, "register the class");
            }
        });
        let title: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
        let (sent, opened) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            // SAFETY: a window on this thread, with its message loop.
            unsafe {
                let hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    CLASS,
                    PCWSTR(title.as_ptr()),
                    WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    420,
                    320,
                    None,
                    None,
                    None,
                    None,
                )
                .expect("create the window");
                controls(hwnd);
                sent.send(hwnd.0 as usize as u64).expect("send");
                let mut message = MSG::default();
                while GetMessageW(&mut message, None, 0, 0).as_bool() {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        });
        let id = opened
            .recv_timeout(Duration::from_secs(5))
            .expect("the window opened");
        // Time to paint.
        std::thread::sleep(Duration::from_millis(300));
        Self {
            id,
            thread: Some(thread),
            _one: one,
        }
    }

    pub fn close(&mut self) {
        // SAFETY: asks the window's own thread to close it.
        let _ =
            unsafe { PostMessageW(Some(super::handle(self.id)), WM_CLOSE, WPARAM(0), LPARAM(0)) };
        if let Some(thread) = self.thread.take() {
            thread.join().expect("the window's thread");
        }
    }
}

impl Drop for TestWindow {
    fn drop(&mut self) {
        self.close();
    }
}
