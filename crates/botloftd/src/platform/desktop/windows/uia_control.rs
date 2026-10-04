//! One control as a line of a reading (spec 24.5): its properties, fetched
//! with the walk, read out of UI Automation's variants.

use windows::Win32::System::Variant::{VARIANT, VT_ARRAY, VT_BSTR, VT_I4};
use windows::Win32::UI::Accessibility::{
    ExpandCollapseState_Collapsed, ExpandCollapseState_Expanded,
    ExpandCollapseState_PartiallyExpanded, IUIAutomationElement, UIA_CONTROLTYPE_ID,
    UIA_ExpandCollapseExpandCollapseStatePropertyId, UIA_IsEnabledPropertyId,
    UIA_IsPasswordPropertyId, UIA_IsValuePatternAvailablePropertyId, UIA_PROPERTY_ID,
    UIA_RuntimeIdPropertyId, UIA_SelectionItemIsSelectedPropertyId,
    UIA_ToggleToggleStatePropertyId, UIA_ValueValuePropertyId,
};

use super::super::read::Control;

unsafe fn property(element: &IUIAutomationElement, id: UIA_PROPERTY_ID) -> Option<VARIANT> {
    // SAFETY: a cached property of a live element.
    unsafe { element.GetCachedPropertyValue(id) }.ok()
}

pub(super) unsafe fn control(
    element: &IUIAutomationElement,
    depth: usize,
    kind: &'static str,
) -> Control {
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
pub(super) fn kind_of(kind: UIA_CONTROLTYPE_ID) -> Option<&'static str> {
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
