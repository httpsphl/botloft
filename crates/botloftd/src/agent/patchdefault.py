root = r'C:\Users\codez\Documents\Coding\botloft\crates'


def edit(rel, fn):
    p = root + '\\' + rel
    s = open(p, encoding='utf8', newline='').read()
    nl = '\r\n' if '\r\n' in s else '\n'
    s = fn(s.replace('\r\n', '\n'))
    open(p, 'w', encoding='utf8', newline='').write(s.replace('\n', nl))


def rep(s, a, b):
    assert a in s, a
    return s.replace(a, b, 1)


# core: Settings.default_agent
def core_settings(s):
    s = rep(s, '''    pub approval_wait_minutes: u32,
}

/// The settings to change''', '''    pub approval_wait_minutes: u32,
    /// The agent bots are made for when nothing says otherwise: new bots,
    /// the chief of a new crew, the bots of a template (spec 30).
    pub default_agent: AgentKind,
}

/// The settings to change''')
    s = rep(s, '''    pub approval_wait_minutes: Option<u32>,
}''', '''    pub approval_wait_minutes: Option<u32>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub default_agent: Option<AgentKind>,
}''')
    return s


edit('botloft-core\\src\\protocol\\settings.rs', core_settings)


def config(s):
    s = rep(s, '''    /// Agents not meant for everyone yet''', '''    /// The agent for new bots: `"claude"`, or an enabled experimental one
    /// (spec 30). Changed in the app's Settings.
    pub default_agent: String,
    /// Agents not meant for everyone yet''')
    s = rep(s, '            agy_path: String::new(),\n', '            agy_path: String::new(),\n            default_agent: "claude".to_owned(),\n')
    return s


edit('botloftd\\src\\config.rs', config)


def settings(s):
    s = rep(s, '''            approval_wait_minutes: minutes,
        };''', '''            approval_wait_minutes: minutes,
            // A name this build does not know is Claude Code.
            default_agent: config.default_agent.parse().unwrap_or(AgentKind::Claude),
        };''')
    s = rep(s, '''        if let Some(minutes) = update.approval_wait_minutes {
            current.settings.approval_wait_minutes = minutes;
            current.approval_wait = minutes_to_wait(minutes);
        }''', '''        if let Some(minutes) = update.approval_wait_minutes {
            current.settings.approval_wait_minutes = minutes;
            current.approval_wait = minutes_to_wait(minutes);
        }
        if let Some(agent) = update.default_agent {
            current.settings.default_agent = agent;
        }''')
    s = rep(s, '''    if let Some(minutes) = update.approval_wait_minutes {
        let bots = config["bots"]''', '''    if let Some(agent) = update.default_agent {
        set(config.as_table_mut(), "default_agent", agent.as_str());
    }
    if let Some(minutes) = update.approval_wait_minutes {
        let bots = config["bots"]''')
    return s


edit('botloftd\\src\\settings.rs', settings)


# the service: one place that says which agent a bot is made for
def svc_bots(s):
    s = rep(s, '''pub fn create(daemon: &Daemon, params: BotsCreateParams) -> ApiResult<Bot> {''', '''/// The agent a bot is made for when nobody chose: the owner's setting, if
/// that agent can run bots, and Claude Code otherwise (spec 30).
pub(crate) fn default_agent(daemon: &Daemon) -> AgentKind {
    let agent = daemon.settings.get().default_agent;
    if daemon.supervisor.agent_enabled(agent) {
        agent
    } else {
        AgentKind::Claude
    }
}

pub fn create(daemon: &Daemon, params: BotsCreateParams) -> ApiResult<Bot> {''')
    s = rep(s, '''        params.model,
        params.agent,
    )?;''', '''        params.model,
        Some(params.agent.unwrap_or_else(|| default_agent(daemon))),
    )?;''')
    return s


edit('botloftd\\src\\service\\bots.rs', svc_bots)
edit('botloftd\\src\\service\\catalog.rs', lambda s: rep(s,
     '        Some(template.model),\n        None,\n    )?;',
     '        Some(template.model),\n        Some(bots::default_agent(daemon)),\n    )?;'))
edit('botloftd\\src\\service\\crews.rs', lambda s: rep(s,
     '&lead.instructions, None, lead.model, None))',
     '&lead.instructions, None, lead.model, Some(bots::default_agent(daemon))))'))
edit('botloftd\\src\\tools\\suggest.rs', lambda s: rep(s,
     '        None,\n        model,\n        None,\n    )\n    .map_err(explain)',
     '        None,\n        model,\n        Some(crate::service::bots::default_agent(daemon)),\n    )\n    .map_err(explain)'))
