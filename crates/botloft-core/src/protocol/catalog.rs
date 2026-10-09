//! The bot catalog (spec 26): ready-made roles the owner adds to a crew and
//! the chief suggests from.

use serde::{Deserialize, Serialize};

use crate::ids::CrewId;

text_enum!(
    /// The group a role is listed under.
    BotTemplateCategory, "template category" {
        Code => "code",
        Design => "design",
        Content => "content",
        Research => "research",
        Business => "business",
        Product => "product",
        Marketing => "marketing",
        Learning => "learning",
        Finance => "finance",
        People => "people",
    }
);

/// A role as the list shows it, in English. The app writes the name, role
/// and summary the owner reads in the owner's language (spec 26.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotTemplate {
    /// The sheet's file name: `code-reviewer`.
    pub id: String,
    pub category: BotTemplateCategory,
    pub name: String,
    pub role: String,
    pub summary: String,
}

/// A role with what a bot made from it starts with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotTemplateFull {
    pub id: String,
    pub category: BotTemplateCategory,
    pub name: String,
    pub role: String,
    pub summary: String,
    pub model: super::BotModel,
    pub effort: super::BotEffort,
    pub instructions: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CatalogListParams {
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub category: Option<BotTemplateCategory>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CatalogGetParams {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CatalogAddParams {
    pub crew_id: CrewId,
    pub template_id: String,
    /// In the owner's language, as the app writes it.
    pub name: String,
    pub role: String,
}
