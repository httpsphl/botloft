//! On Windows, `ssh` ends with the app even when the app does not end well:
//! it runs in a job that kills what is in it when the app's handle to the
//! job closes, which Windows does when the app's process ends.

use std::os::windows::io::AsRawHandle as _;
use std::process::Child;
use std::sync::OnceLock;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};
use windows::core::PCWSTR;

struct Job(HANDLE);

// SAFETY: a job's handle may be used from any thread; it is never closed.
unsafe impl Send for Job {}
// SAFETY: as above.
unsafe impl Sync for Job {}

fn job() -> Option<&'static Job> {
    static JOB: OnceLock<Option<Job>> = OnceLock::new();
    JOB.get_or_init(|| {
        // SAFETY: no security attributes and no name.
        let handle = unsafe { CreateJobObjectW(None, PCWSTR::null()) }.ok()?;
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `limits` is the struct this information class expects.
        unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        }
        .ok()?;
        Some(Job(handle))
    })
    .as_ref()
}

/// Puts `child` in the app's job, so it ends when the app does.
pub fn keep(child: &Child) {
    if let Some(job) = job() {
        // SAFETY: both handles are open; the child's lives as long as `child`.
        let _ = unsafe { AssignProcessToJobObject(job.0, HANDLE(child.as_raw_handle())) };
    }
}
