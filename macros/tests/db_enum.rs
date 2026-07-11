//! Behavioral tests for the `DbEnum` derive. Compile-fail cases (typos in
//! `#[db_enum(...)]`, duplicate strings) are not covered here — they require a
//! `trybuild` harness which is heavier than the value adds for this macro.

use std::str::FromStr;

use macros::DbEnum;
use serde::{Deserialize, Serialize};
use utils::AppError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DbEnum)]
enum Plain {
    Active,
    Inactive,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DbEnum)]
enum Renamed {
    #[db_enum(rename = "Phone Call")]
    PhoneCall,
    Email,
    #[db_enum(rename = "SMS", alias = "sms", alias = "Sms")]
    Sms,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DbEnum)]
#[db_enum(rename_all = "snake_case")]
enum SnakeAll {
    BigBang,
    IRSPayment,
    Plain,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DbEnum)]
#[db_enum(rename_all = "kebab-case")]
enum KebabAll {
    LongName,
    Short,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DbEnum)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SerdeFallback {
    OneTwo,
    Three,
}

// ── Display / AsRef / FromStr ──────────────────────────────────────────────

#[test]
fn display_uses_primary_string() {
    assert_eq!(Plain::Active.to_string(), "Active");
    assert_eq!(Renamed::PhoneCall.to_string(), "Phone Call");
    assert_eq!(Renamed::Sms.to_string(), "SMS");
    assert_eq!(SnakeAll::BigBang.to_string(), "big_bang");
    assert_eq!(KebabAll::LongName.to_string(), "long-name");
    assert_eq!(SerdeFallback::OneTwo.to_string(), "ONE_TWO");
}

#[test]
fn snake_case_keeps_consecutive_acronym_together() {
    // IRSPayment → "irs_payment", not "i_r_s_payment".
    assert_eq!(SnakeAll::IRSPayment.to_string(), "irs_payment");
}

#[test]
fn as_ref_returns_primary_string() {
    let s: &str = Plain::Active.as_ref();
    assert_eq!(s, "Active");
    let s: &str = Renamed::Sms.as_ref();
    assert_eq!(s, "SMS");
}

#[test]
fn from_str_accepts_primary_and_aliases() {
    assert_eq!(Plain::from_str("Active").unwrap(), Plain::Active);
    assert_eq!(Renamed::from_str("Phone Call").unwrap(), Renamed::PhoneCall);
    assert_eq!(Renamed::from_str("SMS").unwrap(), Renamed::Sms);
    assert_eq!(Renamed::from_str("sms").unwrap(), Renamed::Sms);
    assert_eq!(Renamed::from_str("Sms").unwrap(), Renamed::Sms);
    assert_eq!(SnakeAll::from_str("big_bang").unwrap(), SnakeAll::BigBang);
    assert_eq!(
        SerdeFallback::from_str("ONE_TWO").unwrap(),
        SerdeFallback::OneTwo
    );
}

#[test]
fn from_str_rejects_unknown_with_bad_request() {
    let err = Plain::from_str("Nope").unwrap_err();
    match err {
        AppError::BadRequest(msg) => {
            assert!(msg.contains("Plain"), "msg = {msg}");
            assert!(msg.contains("Nope"), "msg = {msg}");
        }
        other => panic!("expected BadRequest, got {other:?}"),
    }
}

#[test]
fn from_str_rejects_alias_pattern_for_non_alias_variant() {
    // "phone_call" was never registered as primary or alias.
    assert!(Renamed::from_str("phone_call").is_err());
}
