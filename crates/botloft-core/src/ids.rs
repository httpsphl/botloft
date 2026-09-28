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
