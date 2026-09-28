//! One Job Object per bot process (spec 7.3). With KILL_ON_JOB_CLOSE, the
//! bot and every process it started die when the job handle closes: when
//! the daemon stops the bot, and also when the daemon itself dies.

use std::io;
use std::os::windows::io::RawHandle;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject,
};
use windows::core::PCWSTR;

#[derive(Debug)]
pub struct ProcessJob(HANDLE);

// SAFETY: a job handle can be used and closed from any thread.
unsafe impl Send for ProcessJob {}
// SAFETY: the Win32 job functions used here are thread-safe.
unsafe impl Sync for ProcessJob {}

impl ProcessJob {
    pub fn new() -> io::Result<Self> {
        // SAFETY: anonymous job with default security.
        let handle = unsafe { CreateJobObjectW(None, PCWSTR::null()) }?;
        let job = Self(handle);
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `limits` is the struct this information class expects.
        unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        }?;
        Ok(job)
    }

    /// Puts a process (and whatever it starts from now on) in the job.
    pub fn assign(&self, process: RawHandle) -> io::Result<()> {
        // SAFETY: `process` is a live process handle owned by the caller.
        unsafe { AssignProcessToJobObject(self.0, HANDLE(process)) }?;
        Ok(())
    }

    /// Kills every process in the job.
    pub fn terminate(&self) -> io::Result<()> {
        // SAFETY: the handle is valid until drop.
        unsafe { TerminateJobObject(self.0, 1) }?;
        Ok(())
    }
}

impl Drop for ProcessJob {
    fn drop(&mut self) {
        // SAFETY: closed exactly once; KILL_ON_JOB_CLOSE ends the processes.
        let _ = unsafe { CloseHandle(self.0) };
    }
}
