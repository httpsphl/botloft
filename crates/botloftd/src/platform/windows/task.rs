//! The per-user scheduled task that starts the daemon at logon (spec 14),
//! through the Task Scheduler COM API. `schtasks.exe` would do the same,
//! but its messages are localized and cannot be parsed.

use std::io;

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, RPC_E_CHANGED_MODE};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::TaskScheduler::{
    IRegisteredTask, ITaskFolder, ITaskService, TASK_CREATE_OR_UPDATE,
    TASK_LOGON_INTERACTIVE_TOKEN, TASK_STATE_DISABLED, TASK_STATE_QUEUED, TASK_STATE_READY,
    TASK_STATE_RUNNING, TaskScheduler,
};
use windows::Win32::System::Variant::VARIANT;
use windows::core::BSTR;

use super::CurrentUser;
use super::task_xml;
use crate::platform::{TaskDefinition, TaskInfo, TaskState};

/// COM for the current thread, released on drop.
struct Com {
    uninit: bool,
}

impl Com {
    fn init() -> io::Result<Self> {
        // SAFETY: plain COM initialization for this thread.
        let status = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if status == RPC_E_CHANGED_MODE {
            // Already initialized with another model; that works too.
            return Ok(Self { uninit: false });
        }
        status.ok()?;
        Ok(Self { uninit: true })
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        if self.uninit {
            // SAFETY: pairs with the successful CoInitializeEx above.
            unsafe { CoUninitialize() };
        }
    }
}

/// The root folder of the current user's Task Scheduler.
struct Scheduler {
    // Declared first so the folder is released before COM goes away.
    folder: ITaskFolder,
    _com: Com,
}

impl Scheduler {
    fn connect() -> io::Result<Self> {
        let com = Com::init()?;
        // SAFETY: COM is initialized on this thread; empty VARIANTs mean the
        // local machine and the current user.
        let folder = unsafe {
            let service: ITaskService =
                CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
            let empty = VARIANT::default();
            service.Connect(&empty, &empty, &empty, &empty)?;
            service.GetFolder(&BSTR::from("\\"))?
        };
        Ok(Self { folder, _com: com })
    }

    fn task(&self, name: &str) -> io::Result<Option<IRegisteredTask>> {
        // SAFETY: `folder` is a live COM object.
        match unsafe { self.folder.GetTask(&BSTR::from(name)) } {
            Ok(task) => Ok(Some(task)),
            Err(err) if err.code() == ERROR_FILE_NOT_FOUND.to_hresult() => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    fn existing(&self, name: &str) -> io::Result<IRegisteredTask> {
        self.task(name)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("the scheduled task {name} does not exist"),
            )
        })
    }
}

/// Creates the task, or replaces its definition if it exists. It runs as
/// the current user, only while they are logged on.
pub fn register_task(name: &str, task: &TaskDefinition) -> io::Result<()> {
    let xml = task_xml::render(task, &CurrentUser::query()?.sid_string()?);
    let scheduler = Scheduler::connect()?;
    let empty = VARIANT::default();
    // SAFETY: `folder` is a live COM object and every argument outlives the call.
    unsafe {
        scheduler.folder.RegisterTask(
            &BSTR::from(name),
            &BSTR::from(xml.as_str()),
            TASK_CREATE_OR_UPDATE.0,
            &empty,
            &empty,
            TASK_LOGON_INTERACTIVE_TOKEN,
            &empty,
        )
    }?;
    Ok(())
}

/// The task's state and command line, or `None` if it does not exist.
pub fn find_task(name: &str) -> io::Result<Option<TaskInfo>> {
    let scheduler = Scheduler::connect()?;
    let Some(task) = scheduler.task(name)? else {
        return Ok(None);
    };
    // SAFETY: `task` is a live COM object.
    let (state, xml) = unsafe { (task.State()?, task.Xml()?) };
    let state = match state {
        TASK_STATE_RUNNING => TaskState::Running,
        TASK_STATE_READY | TASK_STATE_QUEUED => TaskState::Ready,
        TASK_STATE_DISABLED => TaskState::Disabled,
        _ => TaskState::Unknown,
    };
    Ok(Some(TaskInfo {
        state,
        command: task_xml::command_line(&xml.to_string()),
    }))
}

/// Starts the task now. A running task ignores it.
pub fn run_task(name: &str) -> io::Result<()> {
    let scheduler = Scheduler::connect()?;
    let task = scheduler.existing(name)?;
    // SAFETY: `task` is a live COM object.
    unsafe { task.Run(&VARIANT::default()) }?;
    Ok(())
}

/// Ends every running instance of the task, which kills its process.
pub fn stop_task(name: &str) -> io::Result<()> {
    let scheduler = Scheduler::connect()?;
    let task = scheduler.existing(name)?;
    // SAFETY: `task` is a live COM object.
    unsafe { task.Stop(0) }?;
    Ok(())
}

/// Deletes the task. Returns whether it existed.
pub fn delete_task(name: &str) -> io::Result<bool> {
    let scheduler = Scheduler::connect()?;
    // SAFETY: `folder` is a live COM object.
    match unsafe { scheduler.folder.DeleteTask(&BSTR::from(name), 0) } {
        Ok(()) => Ok(true),
        Err(err) if err.code() == ERROR_FILE_NOT_FOUND.to_hresult() => Ok(false),
        Err(err) => Err(err.into()),
    }
}
