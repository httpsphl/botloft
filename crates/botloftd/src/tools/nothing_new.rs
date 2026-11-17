//! `nothing_new` (spec 20.5): in a routine run whose check found nothing
//! new, the bot ends the run quietly instead of writing a report the owner
//! would have to read.

use serde_json::{Value, json};

use crate::seat::Seat;
use crate::state::Daemon;

pub const NOTHING_NEW: &str = "nothing_new";

pub(super) fn tool() -> Value {
    json!({
        "name": NOTHING_NEW,
        "title": "Nothing new",
        "description": "Only in a routine run: when the check found nothing new since the last \
            run, call this, then end your turn without a reply. The run ends quietly, with no \
            report for the owner to read. If you found something, report it in your reply \
            instead.",
        "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
    })
}

pub(super) fn call(daemon: &Daemon, bot: &Seat) -> Result<Value, String> {
    if crate::routines::mark_nothing_new(daemon, bot) {
        Ok(json!({
            "done": true,
            "note": "This run ends quietly. End your turn now, without a reply.",
        }))
    } else {
        Err(
            "No routine run is open in this turn: nothing_new only works when a routine \
             asked you to do something. Reply as usual."
                .to_owned(),
        )
    }
}
