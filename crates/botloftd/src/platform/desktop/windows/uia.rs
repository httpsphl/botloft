//! A window's controls through UI Automation (spec 24.5), the interface
//! screen readers use: it works with the window behind others and never
//! moves the owner's cursor. One walk down the tree, a level at a time,
//! with the properties fetched in the same call.

use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::Variant::{VARIANT, VT_ARRAY, VT_BSTR, VT_I4};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, ExpandCollapseState_Collapsed, ExpandCollapseState_Expanded,
    ExpandCollapseState_PartiallyExpanded, IUIAutomation, IUIAutomationCacheRequest,
    IUIAutomationCondition, IUIAutomationElement, TreeScope_Children, UIA_CONTROLTYPE_ID,
    UIA_ControlTypePropertyId, UIA_ExpandCollapseExpandCollapseStatePropertyId,
    UIA_IsEnabledPropertyId, UIA_IsOffscreenPropertyId, UIA_IsPasswordPropertyId,
    UIA_IsValuePatternAvailablePropertyId, UIA_NamePropertyId, UIA_PROPERTY_ID,
    UIA_RuntimeIdPropertyId, UIA_SelectionItemIsSelectedPropertyId,
    UIA_ToggleToggleStatePropertyId, UIA_ValueValuePropertyId,
};

use super::super::DesktopError;
use super::super::read::Control;
use super::handle;

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

fn system(err: &windows::core::Error) -> DesktopError {
    DesktopError::System(err.message())
}

/// Everything a walk needs, made once.
struct Walk {
    cache: IUIAutomationCacheRequest,
    shown: IUIAutomationCondition,
    found: Vec<Control>,
}

/// The controls of window `id`, in the order they sit in it.
pub fn read(id: u64) -> Result<Vec<Control>, DesktopError> {
    let _com = Com::start();
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
        let root = automation
            .ElementFromHandleBuildCache(handle(id), &cache)
            .map_err(|_| DesktopError::Gone)?;
        let mut walk = Walk {
            cache,
            shown,
            found: Vec::new(),
        };
        walk.children(&root, 0);
        Ok(walk.found)
    }
}

impl Walk {
    /// Adds the shown children of `element`, and theirs.
    unsafe fn children(&mut self, element: &IUIAutomationElement, depth: usize) {
        if depth > DEPTH_MAX || self.found.len() >= CONTROLS_MAX {
            return;
        }
        // SAFETY: COM calls on live interfaces of this walk.
        let Ok(children) =
            (unsafe { element.FindAllBuildCache(TreeScope_Children, &self.shown, &self.cache) })
        else {
            return;
        };
        let count = unsafe { children.Length() }.unwrap_or(0);
        for index in 0..count {
            if self.found.len() >= CONTROLS_MAX {
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

unsafe fn property(element: &IUIAutomationElement, id: UIA_PROPERTY_ID) -> Option<VARIANT> {
    // SAFETY: a cached property of a live element.
    unsafe { element.GetCachedPropertyValue(id) }.ok()
}

unsafe fn control(element: &IUIAutomationElement, depth: usize, kind: &'static str) -> Control {
    // SAFETY: cached properties of a live element.
    unsafe {
        let flag = |id| property(element, id).and_then(|value| bool::try_from(&value).ok());
        let number = |id| property(element, id).and_then(|value| i32::try_from(&value).ok());
        let password = flag(UIA_IsPasswordPropertyId).unwrap_or(false);
        let value = if !password && flag(UIA_IsValuePatternAvailablePropertyId) == Some(true) {
            property(element, UIA_ValueValuePropertyId).and_then(|value| text(&value))
        } else {
            None
        };
        let expanded = number(UIA_ExpandCollapseExpandCollapseStatePropertyId).and_then(|state| {
            if state == ExpandCollapseState_Expanded.0
                || state == ExpandCollapseState_PartiallyExpanded.0
            {
                Some(true)
            } else if state == ExpandCollapseState_Collapsed.0 {
                Some(false)
            } else {
                None
            }
        });
        Control {
            depth,
            kind,
            name: element
                .CachedName()
                .map(|name| name.to_string())
                .unwrap_or_default(),
            value,
            password,
            enabled: flag(UIA_IsEnabledPropertyId).unwrap_or(true),
            // On, off; "indeterminate" says nothing.
            checked: number(UIA_ToggleToggleStatePropertyId).and_then(|state| match state {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            }),
            expanded,
            selected: flag(UIA_SelectionItemIsSelectedPropertyId).unwrap_or(false),
            runtime_id: property(element, UIA_RuntimeIdPropertyId)
                .map(|value| runtime_id(&value))
                .unwrap_or_default(),
        }
    }
}

/// A string value.
fn text(value: &VARIANT) -> Option<String> {
    // SAFETY: the union is read as a string only when its type says so.
    unsafe {
        let inner = &value.Anonymous.Anonymous;
        (inner.vt == VT_BSTR).then(|| inner.Anonymous.bstrVal.to_string())
    }
}

/// The numbers of a runtime id: a one-dimension array of 32-bit integers.
fn runtime_id(value: &VARIANT) -> Vec<i32> {
    // SAFETY: the union is read as an array only when its type says so,
    // and the array's own bounds limit the read.
    unsafe {
        let inner = &value.Anonymous.Anonymous;
        if inner.vt != (VT_ARRAY | VT_I4) {
            return Vec::new();
        }
        let array = inner.Anonymous.parray;
        if array.is_null() || (*array).cDims != 1 || (*array).pvData.is_null() {
            return Vec::new();
        }
        let length = (*array).rgsabound[0].cElements as usize;
        std::slice::from_raw_parts((*array).pvData.cast::<i32>(), length).to_vec()
    }
}

/// A control type in words; `None` for what is left out.
fn kind_of(kind: UIA_CONTROLTYPE_ID) -> Option<&'static str> {
    Some(match kind.0 {
        50000 => "button",
        50001 => "calendar",
        50002 => "check box",
        50003 => "combo box",
        50004 => "edit",
        50005 => "link",
        50006 => "image",
        50007 => "list item",
        50008 => "list",
        50009 => "menu",
        50010 => "menu bar",
        50011 => "menu item",
        50012 => "progress bar",
        50013 => "radio button",
        50015 => "slider",
        50016 => "spinner",
        50017 => "status bar",
        50018 => "tab list",
        50019 => "tab",
        50020 => "text",
        50021 => "toolbar",
        50023 => "tree",
        50024 => "tree item",
        50026 => "group",
        50028 => "grid",
        50029 => "cell",
        50030 => "document",
        50031 => "split button",
        50032 => "window",
        50033 => "pane",
        50034 => "header",
        50035 => "header item",
        50036 => "table",
        50040 => "app bar",
        // Scroll bars, tool tips, thumbs, title bars, separators.
        50014 | 50022 | 50027 | 50037 | 50038 => return None,
        _ => "custom",
    })
}
