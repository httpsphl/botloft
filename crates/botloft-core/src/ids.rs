//! Prefixed ULID identifiers such as `bot_01J9Z3K8...`. The prefix makes IDs
//! readable in logs; the ULID keeps them unique and ordered by creation time.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ulid::Ulid;

/// A string that is not a valid identifier of the expected kind.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid {kind} id: expected `{prefix}` followed by a ULID")]
pub struct InvalidId {
    kind: &'static str,
    prefix: &'static str,
}

macro_rules! prefixed_id {
    ($(#[$doc:meta])* $name:ident, $kind:literal, $prefix:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(test, derive(ts_rs::TS))]
        pub struct $name(String);

        impl $name {
            pub const PREFIX: &'static str = $prefix;

            /// Creates a new identifier ordered after every earlier one.
            pub fn generate() -> Self {
                Self(format!("{}{}", $prefix, Ulid::generate()))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl FromStr for $name {
            type Err = InvalidId;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let invalid = InvalidId { kind: $kind, prefix: $prefix };
                let ulid = s.strip_prefix($prefix).ok_or(invalid.clone())?;
                let ulid = Ulid::from_string(ulid).map_err(|_| invalid)?;
                // Re-encode so lowercase input maps to the canonical form.
                Ok(Self(format!("{}{}", $prefix, ulid)))
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw = String::deserialize(deserializer)?;
                raw.parse().map_err(serde::de::Error::custom)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

prefixed_id!(
    /// Identifies a crew.
    CrewId, "crew", "crw_"
);
prefixed_id!(
    /// Identifies a bot.
    BotId, "bot", "bot_"
);
prefixed_id!(
    /// Identifies a message.
    MessageId, "message", "msg_"
);
prefixed_id!(
    /// Identifies a delivery attempt record.
    DeliveryId, "delivery", "dlv_"
);
prefixed_id!(
    /// Identifies a task between bots.
    TaskId, "task", "tsk_"
);
prefixed_id!(
    /// Identifies an item of a bot's chat.
    ChatItemId, "chat item", "cht_"
);
prefixed_id!(
    /// Identifies a permission request waiting for the owner.
    ApprovalId, "approval", "apr_"
);
prefixed_id!(
    /// Identifies a file the owner attached to a message.
    AttachmentId, "attachment", "att_"
);
prefixed_id!(
    /// Identifies a routine (spec 20).
    RoutineId, "routine", "rtn_"
);
prefixed_id!(
    /// Identifies one run of a routine.
    RoutineRunId, "routine run", "rrn_"
);
prefixed_id!(
    /// Identifies a request a bot may make without asking (spec 10.1).
    RuleId, "rule", "rul_"
);
prefixed_id!(
    /// Identifies a question a bot asked the owner (spec 23).
    QuestionId, "question", "qst_"
);
prefixed_id!(
    /// Identifies what the owner let a bot see or do on their desktop
    /// (spec 24.2).
    DesktopGrantId, "desktop grant", "dsk_"
);
prefixed_id!(
    /// Identifies a connected tool, an MCP server of the owner (spec 25).
    McpServerId, "MCP server", "msv_"
);
prefixed_id!(
    /// Identifies what the owner let a bot reach in another crew (spec 10.4).
    CrewAccessId, "crew access", "cxa_"
);

/// A random version 4 UUID, for Claude Code session ids and the uuid of
/// each message written to a bot (spec 7.3, 9.2).
pub fn random_uuid() -> String {
    let mut value = Ulid::generate().0;
    // Version 4 and the RFC 4122 variant, as UUID validators expect.
    value = (value & !(0xf << 76)) | (0x4 << 76);
    value = (value & !(0x3 << 62)) | (0x2 << 62);
    let hex = format!("{value:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ids_carry_their_prefix_and_parse_back() {
        let id = BotId::generate();
        assert!(id.as_str().starts_with("bot_"));
        assert_eq!(id.as_str().len(), 4 + 26);
        assert_eq!(id.as_str().parse::<BotId>(), Ok(id));
    }

    #[test]
    fn parsing_rejects_wrong_prefix_or_garbage() {
        let crew = CrewId::generate();
        assert!(crew.as_str().parse::<BotId>().is_err());
        assert!("bot_not-a-ulid".parse::<BotId>().is_err());
        assert!("".parse::<TaskId>().is_err());
    }

    #[test]
    fn lowercase_input_is_normalized() {
        let id = MessageId::generate();
        let lower = format!("msg_{}", id.as_str()["msg_".len()..].to_lowercase());
        assert_eq!(lower.parse::<MessageId>(), Ok(id));
    }

    #[test]
    fn ids_are_ordered_by_creation() {
        let first = DeliveryId::generate();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = DeliveryId::generate();
        assert!(first < second);
    }

    #[test]
    fn random_uuids_are_version_4() {
        let uuid = random_uuid();
        assert_eq!(uuid.len(), 36);
        assert_eq!(&uuid[14..15], "4");
        assert!(matches!(&uuid[19..20], "8" | "9" | "a" | "b"), "{uuid}");
        assert_ne!(random_uuid(), uuid);
    }

    #[test]
    fn serde_uses_the_plain_string_and_validates_it() {
        let id = CrewId::generate();
        let json = serde_json::to_string(&id).expect("serialize");
        assert_eq!(json, format!("\"{id}\""));
        assert_eq!(
            serde_json::from_str::<CrewId>(&json).expect("deserialize"),
            id
        );
        assert!(serde_json::from_str::<CrewId>("\"bot_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0\"").is_err());
    }
}
