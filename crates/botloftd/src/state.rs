//! State shared by every connection: the store, the supervisor, the
//! courier, the event bus and what the daemon knows about itself.

use crate::desktop::{Desktop, OwnerIdle};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use std::time::Instant;

use botloft_core::protocol::{
    AccountUsage, AutoBackupStatus, Bot, BotContextChanged, BotDeleted, BotDesktop, BotMcp,
    BotRules, BotStateChanged, BrowserAction, BrowserState, ChatDelta, ChatItemChanged,
    CloudProgress, CloudSignedIn, Crew, CrewDeleted, Delivery, DesktopAwayUse, DesktopState,
    FolderRecycled, McpOverview, Message, Question, ReactionChanged, Routine, RoutineRun,
    ScreenDraft, Task,
};
use botloft_store::Store;
use tokio::runtime::{Handle, RuntimeFlavor};
use tokio::sync::broadcast;

use crate::approvals::Approvals;
use crate::browser::{BrowserSettings, Browsers};
use crate::clock::Clock;
use crate::config::Config;
use crate::context::Contexts;
use crate::courier::{Courier, CourierSettings};
use crate::paths::Paths;
use crate::routines::Routines;
use crate::runtime::Runtime;
use crate::screens::Screens;
use crate::secrets::TokenHash;
use crate::service::tasks::TaskSettings;
use crate::settings::LiveSettings;
use crate::supervisor::{Supervisor, SupervisorSettings};
use crate::trash::Trash;
use crate::workspace::WorkspaceEnv;

/// Something that changed and every connected app should hear about.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    CrewChanged(Crew),
    CrewDeleted(CrewDeleted),
    BotChanged(Bot),
    BotDeleted(BotDeleted),
    FolderRecycled(FolderRecycled),
    BotState(BotStateChanged),
    BotContext(BotContextChanged),
    BotRules(BotRules),
    BotDesktop(BotDesktop),
    McpServers(McpOverview),
    BotMcp(BotMcp),
    DesktopChanged(DesktopState),
    DesktopAway(Vec<DesktopAwayUse>),
    ChatItem(ChatItemChanged),
    ChatDelta(ChatDelta),
    MessageCreated(Message),
    DeliveryChanged(Delivery),
    TaskChanged(Task),
    RoutineChanged(Routine),
    RoutineRun(RoutineRun),
    BrowserChanged(BrowserState),
    BrowserAction(BrowserAction),
    ScreenDraft(ScreenDraft),
    QuestionChanged(Question),
    ReactionChanged(ReactionChanged),
    CloudSignedIn(CloudSignedIn),
    CloudProgress(CloudProgress),
    CloudSigninExpired,
    AutoBackupChanged(AutoBackupStatus),
}

/// Events buffered per connection before a slow client is dropped.
const EVENT_BUFFER: usize = 4096;

/// Settings for what bots may ask of the owner (spec 6, `[bots]`). How
/// long a request waits for the owner is live: `LiveSettings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BotSettings {
    /// Largest attachment, per file (spec 9.5).
    pub attachment_max_bytes: u64,
    /// Most bots a chief's suggestions can bring a crew to (spec 10.2).
    pub max_per_crew: usize,
}

impl BotSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            attachment_max_bytes: config.bots.attachment_max_mb * 1024 * 1024,
            max_per_crew: config.bots.max_per_crew,
        }
    }
}

pub struct DaemonOptions {
    pub paths: Paths,
    /// Port the daemon listens on, written into each bot's `mcp.json`.
    pub port: u16,
    pub store: Store,
    pub owner_token: TokenHash,
    pub runtime: Arc<dyn Runtime>,
    pub supervisor: SupervisorSettings,
    pub clock: Arc<dyn Clock>,
    pub courier: CourierSettings,
    pub tasks: TaskSettings,
    pub bots: BotSettings,
    pub browser: BrowserSettings,
    /// What the owner changes in the app's Settings.
    pub settings: LiveSettings,
    /// Where a deleted bot's folder goes when the owner asks (spec 7.6).
    pub trash: Arc<dyn Trash>,
    /// How long ago the owner last used the computer (spec 24.8).
    pub owner_idle: OwnerIdle,
    /// The server of the account (spec 6, `[cloud]`).
    pub cloud: crate::cloud::CloudSettings,
    /// Where the automatic backup keeps its passphrase (spec 27.10).
    pub secrets: Arc<dyn crate::autobackup::SecretStore>,
}

pub struct Daemon {
    pub paths: Paths,
    pub port: u16,
    pub supervisor: Supervisor,
    pub courier: Courier,
    pub tasks: TaskSettings,
    pub bots: BotSettings,
    pub settings: LiveSettings,
    pub approvals: Approvals,
    pub routines: Routines,
    pub browsers: Browsers,
    pub screens: Screens,
    pub desktop: Desktop,
    /// The account and the copies in the cloud (spec 27).
    pub cloud: crate::cloud::Cloud,
    /// The copies it sends by itself (spec 27.10).
    pub autobackup: crate::autobackup::AutoBackup,
    /// Other crews a bot may reach for its current turn (spec 10.4).
    pub crew_access: crate::service::crew_access::TurnAccess,
    /// What each bot's process had cost by its last turn (spec 8.7).
    pub costs: crate::service::plan::CostMeter,
    pub trash: Arc<dyn Trash>,
    pub contexts: Contexts,
    /// How each bot's connected tools stand since it started (spec 25.5).
    pub mcp_states: crate::service::mcp_state::McpStates,
    /// Time for everything stored or compared with stored times.
    pub clock: Arc<dyn Clock>,
    store: Mutex<Store>,
    owner_token: TokenHash,
    events: broadcast::Sender<Event>,
    usage: Mutex<Option<AccountUsage>>,
    started: Instant,
}

impl Daemon {
    /// The supervisor keeps a weak reference back to the daemon, so both are
    /// created together. Start them with [`crate::supervisor::run`] and
    /// [`crate::courier::run`].
    pub fn new(options: DaemonOptions) -> Arc<Self> {
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        Arc::new_cyclic(|daemon| Self {
            supervisor: Supervisor::new(
                daemon.clone(),
                options.runtime,
                options.supervisor,
                events.clone(),
            ),
            courier: Courier::new(options.courier),
            tasks: options.tasks,
            bots: options.bots,
            settings: options.settings,
            approvals: Approvals::default(),
            routines: Routines::default(),
            browsers: Browsers::new(
                options.browser,
                &options.paths.home,
                events.clone(),
                Arc::clone(&options.clock),
            ),
            screens: Screens::default(),
            desktop: Desktop::new(options.owner_idle),
            autobackup: crate::autobackup::AutoBackup::new(
                &options.paths.home,
                options.secrets,
                options.cloud.auto_tick,
            ),
            cloud: crate::cloud::Cloud::new(options.cloud),
            crew_access: Default::default(),
            costs: Default::default(),
            trash: options.trash,
            contexts: Contexts::default(),
            mcp_states: Default::default(),
            clock: options.clock,
            paths: options.paths,
            port: options.port,
            store: Mutex::new(options.store),
            owner_token: options.owner_token,
            events,
            usage: Mutex::new(None),
            started: Instant::now(),
        })
    }

    /// Locks the store. Hold the guard only for synchronous work, never
    /// across an `.await`.
    pub fn store(&self) -> MutexGuard<'_, Store> {
        // A panic while holding the lock cannot leave SQLite half-written
        // (every write is its own statement or transaction), so keep going.
        match self.store.try_lock() {
            Ok(store) => store,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => {
                off_runtime(|| self.store.lock()).unwrap_or_else(|poisoned| poisoned.into_inner())
            }
        }
    }

    pub fn workspace_env(&self) -> WorkspaceEnv<'_> {
        WorkspaceEnv {
            paths: &self.paths,
            port: self.port,
            approval_timeout: self.settings.approval_wait(),
        }
    }

    pub fn is_owner_token(&self, token: &str) -> bool {
        self.owner_token.matches(token)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }

    /// A sender for work that outlives the call that started it.
    pub(crate) fn events(&self) -> broadcast::Sender<Event> {
        self.events.clone()
    }

    pub(crate) fn emit(&self, event: Event) {
        // No receivers just means no app is connected.
        let _ = self.events.send(event);
    }

    /// The account usage Claude Code last reported (spec 8.1).
    pub fn usage(&self) -> Option<AccountUsage> {
        self.usage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub(crate) fn set_usage(&self, usage: AccountUsage) {
        *self
            .usage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(usage);
    }

    pub fn uptime_ms(&self) -> i64 {
        i64::try_from(self.started.elapsed().as_millis()).unwrap_or(i64::MAX)
    }
}

/// Runs `wait` so that it holds up no other task. A runtime thread stuck in
/// it would also stop the timers and the sockets until another thread woke
/// up (spec 11.1), so it hands its work to a new one first. Elsewhere, and
/// on the single-threaded runtime of some tests, it just waits.
fn off_runtime<T>(wait: impl FnOnce() -> T) -> T {
    match Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(wait)
        }
        _ => wait(),
    }
}
