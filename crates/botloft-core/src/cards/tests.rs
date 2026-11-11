use std::collections::BTreeMap;

use super::*;
use crate::protocol::{CardButton, CardChartKind, CardItem, CardSeries};

fn button(id: &str, label: &str) -> CardButton {
    CardButton {
        id: id.into(),
        label: label.into(),
    }
}

fn field(id: &str, label: &str, kind: CardFieldKind) -> CardField {
    CardField {
        id: id.into(),
        label: label.into(),
        kind,
        options: Vec::new(),
        required: false,
    }
}

fn card(blocks: Vec<CardBlock>) -> CardItem {
    let (title, blocks) = check(" Weekly sales ", blocks).expect("valid");
    CardItem {
        version: 1,
        title,
        blocks,
        answer: None,
    }
}

#[test]
fn a_card_is_tidied_and_its_limits_hold() {
    let (title, blocks) = check(
        " Weekly sales ",
        vec![
            CardBlock::Text {
                text: "  **Up 8%**  ".into(),
            },
            CardBlock::Chart {
                kind: CardChartKind::Bar,
                labels: vec!["Mon".into(), "Tue".into()],
                series: vec![CardSeries {
                    name: "Bread".into(),
                    values: vec![Some(3.0), None],
                }],
                unit: Some("  ".into()),
            },
        ],
    )
    .expect("valid");
    assert_eq!(title, "Weekly sales");
    assert_eq!(
        blocks[0],
        CardBlock::Text {
            text: "**Up 8%**".into()
        }
    );
    let CardBlock::Chart { unit, .. } = &blocks[1] else {
        panic!("chart");
    };
    assert_eq!(unit, &None, "a blank unit is dropped");

    for (blocks, says) in [
        (vec![], "1 to 20 blocks"),
        (
            vec![CardBlock::Table {
                columns: vec!["A".into(), "B".into()],
                rows: vec![vec!["1".into()]],
            }],
            "one cell per column",
        ),
        (
            vec![CardBlock::Chart {
                kind: CardChartKind::Line,
                labels: vec!["Mon".into()],
                series: vec![CardSeries {
                    name: "Bread".into(),
                    values: vec![Some(1.0), Some(2.0)],
                }],
                unit: None,
            }],
            "one value per label",
        ),
        (
            vec![CardBlock::Buttons {
                buttons: vec![button("ok", "Yes"), button("ok", "No")],
            }],
            "used twice",
        ),
        (
            vec![CardBlock::Buttons {
                buttons: vec![button("no spaces", "Yes")],
            }],
            "letters, digits",
        ),
        (
            vec![
                CardBlock::Form {
                    fields: vec![field("a", "A", CardFieldKind::Text)],
                    submit: None,
                },
                CardBlock::Form {
                    fields: vec![field("b", "B", CardFieldKind::Text)],
                    submit: None,
                },
            ],
            "one form at most",
        ),
        (
            vec![CardBlock::Form {
                fields: vec![field("size", "Size", CardFieldKind::Choice)],
                submit: None,
            }],
            "2 to 10 options",
        ),
    ] {
        let err = check("Card", blocks).expect_err("refused");
        assert!(err.0.contains(says), "{err}");
    }
    let seven = (0..7).map(|n| button(&format!("b{n}"), "Go")).collect();
    let err = check("Card", vec![CardBlock::Buttons { buttons: seven }]).expect_err("too many");
    assert!(err.0.contains("1 to 6 buttons"), "{err}");
    let long = "x".repeat(CARD_MAX_BYTES);
    let blocks = (0..10)
        .map(|_| CardBlock::Text {
            text: long[..CARD_TEXT_MAX_CHARS].to_owned(),
        })
        .collect();
    let err = check("Card", blocks).expect_err("too big");
    assert!(err.0.contains("too big"), "{err}");
}

#[test]
fn a_button_answers_with_its_label() {
    let card = card(vec![CardBlock::Buttons {
        buttons: vec![button("publish", "Publish"), button("wait", "Wait a day")],
    }]);
    assert_eq!(
        answer(&card, Some("wait"), None).expect("pressed"),
        (Some("wait".into()), "Wait a day".into())
    );
    let err = answer(&card, Some("nope"), None).expect_err("unknown");
    assert!(err.0.contains("no button"), "{err}");
    let err = answer(&card, None, Some(&BTreeMap::new())).expect_err("no form");
    assert!(err.0.contains("no form"), "{err}");
    let err = answer(&card, None, None).expect_err("nothing");
    assert!(err.0.contains("press one button"), "{err}");
}

#[test]
fn a_form_answers_with_one_line_per_field() {
    let mut size = field("size", "Size", CardFieldKind::Choice);
    size.options = vec!["Small".into(), "Large".into()];
    let mut name = field("name", "Name", CardFieldKind::Text);
    name.required = true;
    let card = card(vec![CardBlock::Form {
        fields: vec![
            name,
            field("budget", "Budget", CardFieldKind::Number),
            size,
            field("urgent", "Urgent", CardFieldKind::Check),
            field("notes", "Notes", CardFieldKind::LongText),
        ],
        submit: Some("Create".into()),
    }]);
    let values: BTreeMap<String, String> = [
        ("name", "  Acme   Bakery "),
        ("budget", "5000,50"),
        ("size", "Large"),
        ("urgent", "true"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.to_owned()))
    .collect();
    let (pressed, text) = answer(&card, None, Some(&values)).expect("sent");
    assert_eq!(pressed, None);
    assert_eq!(
        text,
        "Name: Acme Bakery\nBudget: 5000,50\nSize: Large\nUrgent: ✓\nNotes: —"
    );

    for (key, value, says) in [
        ("name", "", "must be filled in"),
        ("budget", "lots", "takes a number"),
        ("size", "Medium", "not one of the options"),
        ("urgent", "maybe", "true or false"),
        ("color", "red", "no field \"color\""),
    ] {
        let mut wrong = values.clone();
        wrong.insert(key.to_owned(), value.to_owned());
        let err = answer(&card, None, Some(&wrong)).expect_err("refused");
        assert!(err.0.contains(says), "{key}: {err}");
    }
}

#[test]
fn the_phone_reads_a_card_as_markdown() {
    let card = card(vec![
        CardBlock::Text {
            text: "Up **8%**".into(),
        },
        CardBlock::Table {
            columns: vec!["Item".into(), "Sold".into()],
            rows: vec![vec!["Bread | rye".into(), "12".into()]],
        },
        CardBlock::Chart {
            kind: CardChartKind::Bar,
            labels: vec!["Mon".into(), "Tue".into()],
            series: vec![CardSeries {
                name: "Cakes".into(),
                values: vec![Some(2.5), None],
            }],
            unit: Some("kg".into()),
        },
        CardBlock::Buttons {
            buttons: vec![button("ok", "Publish")],
        },
    ]);
    assert_eq!(
        as_markdown(&card),
        "**Weekly sales**\n\nUp **8%**\n\n| Item | Sold |\n| --- | --- |\n| Bread \\| rye | 12 |\n\n\
         | kg | Cakes |\n| --- | --- |\n| Mon | 2.5 |\n| Tue |  |"
    );
}
