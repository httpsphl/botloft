//! A card as markdown, where it cannot be shown as itself (spec 8.11).

use crate::protocol::{CardBlock, CardItem};

/// The card as markdown, for where it cannot be shown as itself (the
/// phone): the title, the text and the tables, a chart as a table. Its
/// buttons and form are left out.
pub fn as_markdown(card: &CardItem) -> String {
    let mut parts = vec![format!("**{}**", card.title)];
    for block in &card.blocks {
        match block {
            CardBlock::Text { text } => parts.push(text.clone()),
            CardBlock::Table { columns, rows } => parts.push(markdown_table(columns, rows)),
            CardBlock::Chart {
                labels,
                series,
                unit,
                ..
            } => {
                let mut columns = vec![unit.clone().unwrap_or_default()];
                columns.extend(series.iter().map(|one| one.name.clone()));
                let rows: Vec<Vec<String>> = labels
                    .iter()
                    .enumerate()
                    .map(|(at, label)| {
                        let mut row = vec![label.clone()];
                        row.extend(series.iter().map(|one| {
                            one.values
                                .get(at)
                                .copied()
                                .flatten()
                                .map_or_else(String::new, |value| value.to_string())
                        }));
                        row
                    })
                    .collect();
                parts.push(markdown_table(&columns, &rows));
            }
            CardBlock::Buttons { .. } | CardBlock::Form { .. } => {}
        }
    }
    parts.join("\n\n")
}

fn markdown_table(columns: &[String], rows: &[Vec<String>]) -> String {
    let cell = |text: &str| text.replace('|', "\\|").replace('\n', " ");
    let line = |cells: Vec<String>| format!("| {} |", cells.join(" | "));
    let mut lines = vec![
        line(columns.iter().map(|one| cell(one)).collect()),
        line(columns.iter().map(|_| "---".to_owned()).collect()),
    ];
    lines.extend(
        rows.iter()
            .map(|row| line(row.iter().map(|one| cell(one)).collect())),
    );
    lines.join("\n")
}
