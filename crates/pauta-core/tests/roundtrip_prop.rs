//! Property tests: reading and re-writing never changes a file, and editing one
//! field leaves every other field as it was (sections 14.4 and 15.1).

use pauta_core::frontmatter::Doc;
use pauta_core::yaml::{Y, render_entry};
use proptest::prelude::*;

fn scalar() -> impl Strategy<Value = Y> {
    prop_oneof![
        Just(Y::Null),
        any::<bool>().prop_map(Y::Bool),
        any::<i32>().prop_map(|i| Y::Int(i.into())),
        "[a-zA-Z][a-zA-Z0-9_-]{0,10}".prop_map(Y::Str),
        // Anything printable, including YAML-hostile characters.
        "[ -~çãéíõú\t]{0,30}".prop_map(Y::Str),
        "[ -~]{0,20}".prop_map(Y::Quoted),
        Just(Y::Plain("2026-11-04T09:00:00-03:00".into())),
        Just(Y::Plain("2026-12-05".into())),
    ]
}

fn value() -> impl Strategy<Value = Y> {
    prop_oneof![
        4 => scalar(),
        1 => proptest::collection::vec(scalar(), 0..4).prop_map(Y::Seq),
        1 => proptest::collection::btree_map("[a-z]{1,6}", scalar(), 1..4).prop_map(|m| Y::Map(m.into_iter().collect())),
    ]
}

fn header() -> impl Strategy<Value = Vec<(String, Y)>> {
    proptest::collection::btree_map("[a-z][a-z_]{0,8}", value(), 1..8).prop_map(|m| m.into_iter().collect())
}

fn body() -> impl Strategy<Value = String> {
    prop_oneof![
        "(\n|[ -~]|## Notas\n|- \\[ \\] item\n|```\n---\n```\n|ção){0,40}",
        Just(String::new()),
    ]
}

fn build(fields: &[(String, Y)], comments: &[bool], body: &str) -> String {
    let mut text = String::from("---\n");
    for (i, (k, v)) in fields.iter().enumerate() {
        if comments.get(i).copied().unwrap_or(false) {
            text.push_str("# comentário à mão\n");
        }
        text.push_str(&render_entry(k, v, 0, "\n"));
    }
    text.push_str("---\n");
    text.push_str(body);
    text
}

proptest! {
    #[test]
    fn parse_render_is_identity(fields in header(), comments in proptest::collection::vec(any::<bool>(), 8), body in body()) {
        let text = build(&fields, &comments, &body);
        let doc = Doc::parse(&text);
        prop_assert_eq!(doc.render(), text.clone());
        prop_assert_eq!(doc.body(), body.as_str());
        // What we write is valid YAML.
        prop_assert!(doc.fields().is_ok(), "{}", text);
    }

    #[test]
    fn setting_one_key_keeps_the_others(
        fields in header(),
        comments in proptest::collection::vec(any::<bool>(), 8),
        body in body(),
        new_value in value(),
        pick in any::<prop::sample::Index>(),
    ) {
        let text = build(&fields, &comments, &body);
        let mut doc = Doc::parse(&text);
        let before = doc.fields().unwrap();
        let key = &fields[pick.index(fields.len())].0;
        doc.set(key, &new_value);
        let after = doc.fields().unwrap();
        for (k, v) in before.iter() {
            if k.as_str() != Some(key.as_str()) {
                prop_assert_eq!(after.get(k), Some(v));
            }
        }
        prop_assert_eq!(Y::from_value(after.get(key.as_str()).unwrap()), normalize(&new_value));
        prop_assert_eq!(doc.body(), body.as_str());
        prop_assert_eq!(after.len(), before.len());
    }
}

/// What a value looks like after being written and read back.
fn normalize(v: &Y) -> Y {
    match v {
        Y::Quoted(s) | Y::Str(s) => {
            if pauta_core::yaml::looks_like_date(s) {
                Y::Plain(s.clone())
            } else {
                Y::Str(s.clone())
            }
        }
        Y::Seq(items) => Y::Seq(items.iter().map(normalize).collect()),
        Y::Map(entries) => Y::Map(entries.iter().map(|(k, v)| (k.clone(), normalize(v))).collect()),
        other => other.clone(),
    }
}
