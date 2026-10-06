//! The bot catalog (spec 26): ready-made roles, one sheet each in
//! `crates/botloftd/catalog/<id>.toml`, built into the daemon.

#[cfg(test)]
mod lint;

use std::sync::OnceLock;

use botloft_core::protocol::{
    BotEffort, BotModel, BotTemplate, BotTemplateCategory, BotTemplateFull,
};
use serde::Deserialize;
use tracing::error;

/// One sheet of the catalog: its id and its file's text.
macro_rules! sheet {
    ($id:literal) => {
        ($id, include_str!(concat!("../../catalog/", $id, ".toml")))
    };
}

/// Every sheet, in the order the catalog lists them. A test makes sure this
/// is the folder's content.
const SHEETS: &[(&str, &str)] = &[
    sheet!("developer"),
    sheet!("code-reviewer"),
    sheet!("qa-tester"),
    sheet!("designer"),
    sheet!("writer"),
    sheet!("social-media"),
    sheet!("translator"),
    sheet!("researcher"),
    sheet!("data-analyst"),
    sheet!("sales-prospector"),
    sheet!("customer-support"),
    sheet!("personal-assistant"),
    sheet!("product-manager"),
    sheet!("project-manager"),
    sheet!("business-analyst"),
    sheet!("ux-researcher"),
    sheet!("agile-facilitator"),
    sheet!("goals-coach"),
    sheet!("meeting-secretary"),
    sheet!("process-analyst"),
    sheet!("operations-manager"),
    sheet!("recruiter"),
    sheet!("onboarding-coach"),
    sheet!("customer-success"),
    sheet!("event-planner"),
    sheet!("travel-planner"),
    sheet!("seo-specialist"),
    sheet!("email-marketer"),
    sheet!("content-strategist"),
    sheet!("copywriter"),
    sheet!("ads-manager"),
    sheet!("video-scriptwriter"),
    sheet!("newsletter-editor"),
    sheet!("community-manager"),
    sheet!("ecommerce-manager"),
    sheet!("brand-strategist"),
    sheet!("competitor-analyst"),
    sheet!("growth-marketer"),
    sheet!("conversion-optimizer"),
    sheet!("pr-writer"),
    sheet!("reputation-manager"),
    sheet!("proposal-writer"),
    sheet!("sales-coach"),
    sheet!("pricing-analyst"),
];

/// What a sheet file holds.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Sheet {
    pub category: BotTemplateCategory,
    pub name: String,
    pub role: String,
    pub summary: String,
    pub model: BotModel,
    pub effort: BotEffort,
    pub instructions: String,
}

impl Sheet {
    /// The multi-line string ends with a newline; a bot's instructions are
    /// trimmed, so the sheet is too.
    fn parse(text: &str) -> Result<Self, toml::de::Error> {
        let mut sheet: Self = toml::from_str(text)?;
        sheet.instructions = sheet.instructions.trim().to_owned();
        Ok(sheet)
    }

    fn template(&self, id: &str) -> BotTemplate {
        BotTemplate {
            id: id.to_owned(),
            category: self.category,
            name: self.name.clone(),
            role: self.role.clone(),
            summary: self.summary.clone(),
        }
    }

    fn full(&self, id: &str) -> BotTemplateFull {
        BotTemplateFull {
            id: id.to_owned(),
            category: self.category,
            name: self.name.clone(),
            role: self.role.clone(),
            summary: self.summary.clone(),
            model: self.model,
            effort: self.effort,
            instructions: self.instructions.clone(),
        }
    }
}

/// The sheets, read once. A sheet that does not read is left out and
/// logged; `catalog_lint` keeps that from reaching a release.
fn sheets() -> &'static [(&'static str, Sheet)] {
    static SHEETS_READ: OnceLock<Vec<(&'static str, Sheet)>> = OnceLock::new();
    SHEETS_READ.get_or_init(|| {
        SHEETS
            .iter()
            .filter_map(|(id, text)| match Sheet::parse(text) {
                Ok(sheet) => Some((*id, sheet)),
                Err(err) => {
                    error!(template = id, "a catalog sheet does not read: {err}");
                    None
                }
            })
            .collect()
    })
}

/// The roles, without their instructions, of `category` or of all.
pub fn list(category: Option<BotTemplateCategory>) -> Vec<BotTemplate> {
    sheets()
        .iter()
        .filter(|(_, sheet)| category.is_none_or(|wanted| sheet.category == wanted))
        .map(|(id, sheet)| sheet.template(id))
        .collect()
}

/// One role with its instructions, if `id` names one.
pub fn get(id: &str) -> Option<BotTemplateFull> {
    sheets()
        .iter()
        .find(|(sheet_id, _)| *sheet_id == id)
        .map(|(id, sheet)| sheet.full(id))
}
