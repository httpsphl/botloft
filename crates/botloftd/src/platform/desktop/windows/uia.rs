//! A window's controls through UI Automation (spec 24.5), the interface
//! screen readers use: it works with the window behind others and never
//! moves the owner's cursor. One walk down the tree, a level at a time,
//! with the properties fetched in the same call; the same walk finds the
//! control a reading named, to act on it.

use std::sync::{Mutex, MutexGuard, PoisonError};

use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationCacheRequest, IUIAutomationCondition,
    IUIAutomationElement, TreeScope_Children, UIA_ControlTypePropertyId,
    UIA_ExpandCollapseExpandCollapseStatePropertyId, UIA_IsEnabledPropertyId,
    UIA_IsOffscreenPropertyId, UIA_IsPasswordPropertyId, UIA_IsValuePatternAvailablePropertyId,
    UIA_NamePropertyId, UIA_PROPERTY_ID, UIA_RuntimeIdPropertyId,
    UIA_SelectionItemIsSelectedPropertyId, UIA_ToggleToggleStatePropertyId,
    UIA_ValueValuePropertyId,
};

use super::super::DesktopError;
use super::super::read::Control;
use super::handle;
use super::uia_control::{control, kind_of};

/// The most controls one walk gathers, and how deep it goes: a huge
/// window is cut short, not read for minutes.
const CONTROLS_MAX: usize = 3000;
const DEPTH_MAX: usize = 40;

const PROPERTIES: [UIA_PROPERTY_ID; 11] = [
    UIA_NamePropertyId,
    UIA_ControlTypePropertyId,
    UIA_IsPasswordPropertyId,
    UIA_IsEnabledPropertyId,
    UIA_IsOffscreenPropertyId,
    UIA_IsValuePatternAvailablePropertyId,
    UIA_ValueValuePropertyId,
    UIA_ToggleToggleStatePropertyId,
    UIA_ExpandCollapseExpandCollapseStatePropertyId,
    UIA_SelectionItemIsSelectedPropertyId,
    UIA_RuntimeIdPropertyId,
];

/// COM on this thread while it lives, multithreaded, as UI Automation
/// clients should be.
struct Com {
    started: bool,
}

impl Com {
    fn start() -> Self {
        // SAFETY: balanced by `CoUninitialize` on drop when it started here.
        let started = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        Self { started }
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        if self.started {
            // SAFETY: this thread started COM in `start`.
            unsafe { CoUninitialize() };
        }
    }
}

pub(super) fn system(err: &windows::core::Error) -> DesktopError {
    DesktopError::System(err.message())
}

/// One UI Automation session at a time in this process: sessions on two
/// threads at once fail with "unspecified error" (seen with Windows 11).
static ONE: Mutex<()> = Mutex::new(());

/// UI Automation on this thread, ready to walk windows.
pub(super) struct Session {
    pub(super) automation: IUIAutomation,
    cache: IUIAutomationCacheRequest,
    shown: IUIAutomationCondition,
    // Last: COM ends after the interfaces above are released, then the
    // next session may start.
    _com: Com,
    _one: MutexGuard<'static, ()>,
}

impl Session {
    pub(super) fn start() -> Result<Self, DesktopError> {
        let one = ONE.lock().unwrap_or_else(PoisonError::into_inner);
        let com = Com::start();
        // SAFETY: COM calls on interfaces made here, on this thread.
        unsafe {
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                    .map_err(|err| system(&err))?;
            let cache = automation
                .CreateCacheRequest()
                .map_err(|err| system(&err))?;
            for property in PROPERTIES {
                cache.AddProperty(property).map_err(|err| system(&err))?;
            }
            // What a person sees: the control view, nothing scrolled away.
            let offscreen = automation
                .CreatePropertyCondition(UIA_IsOffscreenPropertyId, &VARIANT::from(true))
                .map_err(|err| system(&err))?;
            let shown = automation
                .CreateAndCondition(
                    &automation
                        .ControlViewCondition()
                        .map_err(|err| system(&err))?,
                    &automation
                        .CreateNotCondition(&offscreen)
                        .map_err(|err| system(&err))?,
                )
                .map_err(|err| system(&err))?;
            Ok(Self {
                automation,
                cache,
                shown,
                _com: com,
                _one: one,
            })
        }
    }

    /// Walks window `id`: every shown control, or until `target` is found.
    fn walk<'a>(&'a self, id: u64, target: Option<&'a [i32]>) -> Result<Walk<'a>, DesktopError> {
        // SAFETY: COM calls on this session's interfaces.
        let root = unsafe {
            self.automation
                .ElementFromHandleBuildCache(handle(id), &self.cache)
        }
        .map_err(|_| DesktopError::Gone)?;
        let mut walk = Walk {
            session: self,
            target,
            hit: None,
            found: Vec::new(),
        };
        // SAFETY: as above.
        unsafe { walk.children(&root, 0) };
        Ok(walk)
    }

    /// The control of window `id` whose `RuntimeId` is `target`, and how it
    /// reads.
    pub(super) fn find(
        &self,
        id: u64,
        target: &[i32],
    ) -> Result<(IUIAutomationElement, Control), DesktopError> {
        let walk = self.walk(id, Some(target))?;
        let control = walk.found.last().cloned();
        match (walk.hit, control) {
            (Some(element), Some(control)) => Ok((element, control)),
            _ => Err(DesktopError::NotThere),
        }
    }
}

/// One walk down a window.
struct Walk<'a> {
    session: &'a Session,
    /// The control looked for, by its `RuntimeId`.
    target: Option<&'a [i32]>,
    hit: Option<IUIAutomationElement>,
    found: Vec<Control>,
}

/// The controls of window `id`, in the order they sit in it.
pub fn read(id: u64) -> Result<Vec<Control>, DesktopError> {
    Ok(Session::start()?.walk(id, None)?.found)
}

impl Walk<'_> {
    /// Adds the shown children of `element`, and theirs.
    unsafe fn children(&mut self, element: &IUIAutomationElement, depth: usize) {
        if depth > DEPTH_MAX || self.found.len() >= CONTROLS_MAX || self.hit.is_some() {
            return;
        }
        let session = self.session;
        // SAFETY: COM calls on live interfaces of this walk.
        let Ok(children) = (unsafe {
            element.FindAllBuildCache(TreeScope_Children, &session.shown, &session.cache)
        }) else {
            return;
        };
        let count = unsafe { children.Length() }.unwrap_or(0);
        for index in 0..count {
            if self.found.len() >= CONTROLS_MAX || self.hit.is_some() {
                return;
            }
            let Ok(child) = (unsafe { children.GetElement(index) }) else {
                continue;
            };
            let kind = kind_of(unsafe { child.CachedControlType() }.unwrap_or_default());
            let Some(kind) = kind else {
                // Window furniture: scroll bars, the title bar.
                continue;
            };
            let control = unsafe { control(&child, depth, kind) };
            if self
                .target
                .is_some_and(|target| target == control.runtime_id.as_slice())
            {
                self.found.push(control);
                self.hit = Some(child);
                return;
            }
            // Unnamed panes and groups only hold others: their children
            // take their place.
            let holder = matches!(kind, "pane" | "group" | "custom") && control.name.is_empty();
            let next = if holder {
                depth
            } else {
                self.found.push(control);
                depth + 1
            };
            unsafe { self.children(&child, next) };
        }
    }
}
