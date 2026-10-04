//! Acting on a control through UI Automation's patterns (spec 24.5): no
//! mouse, no keyboard, the window may stay behind others. A click is the
//! first of Invoke, Toggle, Select and Expand/Collapse the control has.

use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    ExpandCollapseState_Collapsed, IUIAutomation2, IUIAutomationElement,
    IUIAutomationExpandCollapsePattern, IUIAutomationInvokePattern,
    IUIAutomationLegacyIAccessiblePattern, IUIAutomationScrollItemPattern,
    IUIAutomationScrollPattern, IUIAutomationSelectionItemPattern, IUIAutomationTogglePattern,
    IUIAutomationValuePattern, ScrollAmount_LargeDecrement, ScrollAmount_LargeIncrement,
    ScrollAmount_NoAmount, TreeScope_Descendants, UIA_ExpandCollapsePatternId, UIA_InvokePatternId,
    UIA_LegacyIAccessiblePatternId, UIA_NamePropertyId, UIA_ScrollItemPatternId,
    UIA_ScrollPatternId, UIA_SelectionItemPatternId, UIA_TogglePatternId, UIA_ValuePatternId,
};
use windows::core::{BSTR, Interface};

use super::super::{Acted, Action, DesktopError, Scroll};
use super::uia::{Session, system};

/// How long a call into the app may take: one that opens a dialog may not
/// come back until the dialog closes.
const TRANSACTION_MS: u32 = 5000;
/// UI Automation's "the app took too long".
const TIMEOUT: windows::core::HRESULT = windows::core::HRESULT(0x8013_1505_u32 as i32);
/// "No scroll" for one direction of `SetScrollPercent`.
const NO_SCROLL: f64 = -1.0;

pub fn act(id: u64, target: &[i32], action: &Action) -> Result<Acted, DesktopError> {
    let session = Session::start()?;
    // SAFETY: a COM call on this session's interface.
    if let Ok(quick) = session.automation.cast::<IUIAutomation2>() {
        let _ = unsafe { quick.SetTransactionTimeout(TRANSACTION_MS) };
    }
    let (element, control) = session.find(id, target)?;
    if control.password {
        return Err(DesktopError::Password);
    }
    if !control.enabled {
        return Err(DesktopError::Cannot("used while it is disabled"));
    }
    // SAFETY: COM calls on the element just found, on this thread.
    let done = unsafe {
        match action {
            Action::Click => click(&element),
            Action::Type(text) => type_text(&element, text),
            Action::Select(option) => select(&session, &element, option),
            Action::Scroll(to) => scroll(&element, *to),
        }
    };
    match done {
        Err(DesktopError::System(_)) if waited(&done) => Ok(Acted::Waiting),
        Err(err) => Err(err),
        Ok(()) => Ok(Acted::Done),
    }
}

/// Whether the call failed only because the app took too long.
fn waited(done: &Result<(), DesktopError>) -> bool {
    matches!(done, Err(DesktopError::System(message)) if message.contains("0x80131505"))
}

/// Turns a failed pattern call into an error, keeping a time-out apart.
fn called(result: windows::core::Result<()>) -> Result<(), DesktopError> {
    result.map_err(|err| {
        if err.code() == TIMEOUT {
            DesktopError::System(format!("0x80131505 {}", err.message()))
        } else {
            system(&err)
        }
    })
}

unsafe fn pattern<T: Interface>(
    element: &IUIAutomationElement,
    id: windows::Win32::UI::Accessibility::UIA_PATTERN_ID,
) -> Option<T> {
    // SAFETY: a COM call on a live element; a missing pattern is an error
    // or a null, both `None` here.
    unsafe { element.GetCurrentPatternAs::<T>(id) }.ok()
}

unsafe fn click(element: &IUIAutomationElement) -> Result<(), DesktopError> {
    // SAFETY: COM calls on a live element and its patterns.
    unsafe {
        if let Some(invoke) = pattern::<IUIAutomationInvokePattern>(element, UIA_InvokePatternId) {
            return called(invoke.Invoke());
        }
        if let Some(toggle) = pattern::<IUIAutomationTogglePattern>(element, UIA_TogglePatternId) {
            return called(toggle.Toggle());
        }
        if let Some(item) =
            pattern::<IUIAutomationSelectionItemPattern>(element, UIA_SelectionItemPatternId)
        {
            return called(item.Select());
        }
        if let Some(expand) =
            pattern::<IUIAutomationExpandCollapsePattern>(element, UIA_ExpandCollapsePatternId)
        {
            let state = expand
                .CurrentExpandCollapseState()
                .map_err(|err| system(&err))?;
            return called(if state == ExpandCollapseState_Collapsed {
                expand.Expand()
            } else {
                expand.Collapse()
            });
        }
    }
    Err(DesktopError::Cannot("clicked"))
}

unsafe fn type_text(element: &IUIAutomationElement, text: &str) -> Result<(), DesktopError> {
    // SAFETY: COM calls on a live element and its pattern.
    unsafe {
        let Some(value) = pattern::<IUIAutomationValuePattern>(element, UIA_ValuePatternId) else {
            return Err(DesktopError::Cannot("typed in"));
        };
        if value
            .CurrentIsReadOnly()
            .is_ok_and(|read_only| read_only.as_bool())
        {
            return Err(DesktopError::ReadOnly);
        }
        called(value.SetValue(&BSTR::from(text)))
    }
}

/// Picks the option named `option` in a list or combo box. Selected where
/// it sits, when it shows without opening anything; a combo box keeps its
/// options in a list that only shows open, and a classic one only takes
/// the choice from there by the item's default action (a double click),
/// which also closes it. Closing it any other way would cancel the choice,
/// as Esc does.
unsafe fn select(
    session: &Session,
    element: &IUIAutomationElement,
    option: &str,
) -> Result<(), DesktopError> {
    // SAFETY: COM calls on a live element, its patterns and descendants.
    unsafe {
        let named = session
            .automation
            .CreatePropertyCondition(UIA_NamePropertyId, &VARIANT::from(option))
            .map_err(|err| system(&err))?;
        let find = || element.FindFirst(TreeScope_Descendants, &named).ok();
        let value = pattern::<IUIAutomationValuePattern>(element, UIA_ValuePatternId);
        let shows = || {
            value
                .as_ref()
                .and_then(|value| value.CurrentValue().ok())
                .is_some_and(|shown| shown == option)
        };
        let selection = |item: &IUIAutomationElement| {
            pattern::<IUIAutomationSelectionItemPattern>(item, UIA_SelectionItemPatternId)
        };
        if let Some(item) = find()
            && let Some(selection) = selection(&item)
        {
            called(selection.Select())?;
            // A list, or a combo box that took it: done.
            if value.is_none() || shows() {
                return Ok(());
            }
        }
        let Some(expand) =
            pattern::<IUIAutomationExpandCollapsePattern>(element, UIA_ExpandCollapsePatternId)
        else {
            return Err(DesktopError::NoOption(option.to_owned()));
        };
        called(expand.Expand())?;
        let Some(item) = find() else {
            let _ = expand.Collapse();
            return Err(DesktopError::NoOption(option.to_owned()));
        };
        if let Some(legacy) =
            pattern::<IUIAutomationLegacyIAccessiblePattern>(&item, UIA_LegacyIAccessiblePatternId)
        {
            return called(legacy.DoDefaultAction());
        }
        // Without a default action, selected in the open list, and the
        // list left open for the owner to see.
        match selection(&item) {
            Some(selection) => called(selection.Select()),
            None => Err(DesktopError::Cannot("chosen")),
        }
    }
}

unsafe fn scroll(element: &IUIAutomationElement, to: Scroll) -> Result<(), DesktopError> {
    // SAFETY: COM calls on a live element and its patterns.
    unsafe {
        if let Some(scroll) = pattern::<IUIAutomationScrollPattern>(element, UIA_ScrollPatternId) {
            return called(match to {
                Scroll::Down => scroll.Scroll(ScrollAmount_NoAmount, ScrollAmount_LargeIncrement),
                Scroll::Up => scroll.Scroll(ScrollAmount_NoAmount, ScrollAmount_LargeDecrement),
                Scroll::Top => scroll.SetScrollPercent(NO_SCROLL, 0.0),
                Scroll::Bottom => scroll.SetScrollPercent(NO_SCROLL, 100.0),
            });
        }
        // An item in a list that scrolls: brought into view.
        if let Some(item) =
            pattern::<IUIAutomationScrollItemPattern>(element, UIA_ScrollItemPatternId)
        {
            return called(item.ScrollIntoView());
        }
    }
    Err(DesktopError::Cannot("scrolled"))
}
