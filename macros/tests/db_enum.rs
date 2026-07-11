//! Behavioral tests for the `DbEnum` derive. Compile-fail cases (typos in
//! `#[db_enum(...)]`, duplicate strings) are not covered here — they require a
//! `trybuild` harness which is heavier than the value adds for this macro.

use std::str::FromStr;

use macros::DbEnum;
use serde::{Deserialize, Serialize};
use utils::test::TestApp;
use utils::{AppError, Connection};

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

// ── sqlx round-trip ────────────────────────────────────────────────────────

/// A fresh template clone with the PG enum types `DbEnum` binds to. `DbEnum`
/// keys its `Type<Postgres>` to `snake_case(EnumName)`, so the type names below
/// must match (`Plain` → `plain`, `Renamed` → `renamed`). For `from_row` /
/// `from_row_opt` (which decode via `String`/`text`) the column is `text`.
async fn pool() -> Connection {
    let pool = TestApp::setup_db(env!("CARGO_MANIFEST_DIR"), "").await;
    sqlx::raw_sql(
        "CREATE TYPE plain AS ENUM ('Active', 'Inactive');
         CREATE TYPE renamed AS ENUM ('Phone Call', 'Email', 'SMS');",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn db_round_trip_via_sqlx() {
    let pool = pool().await;
    sqlx::raw_sql("CREATE TABLE t(status renamed NOT NULL);")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO t(status) VALUES ($1)")
        .bind(Renamed::PhoneCall)
        .execute(&pool)
        .await
        .unwrap();
    let got: Renamed = sqlx::query_scalar("SELECT status FROM t")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(got, Renamed::PhoneCall);
}

#[tokio::test]
async fn db_round_trip_handles_option_some_and_none() {
    let pool = pool().await;
    sqlx::raw_sql("CREATE TABLE t(id serial PRIMARY KEY, status plain);")
        .execute(&pool)
        .await
        .unwrap();
    let opt_some: Option<Plain> = Some(Plain::Active);
    let opt_none: Option<Plain> = None;
    sqlx::query("INSERT INTO t(status) VALUES ($1), ($2)")
        .bind(opt_some)
        .bind(opt_none)
        .execute(&pool)
        .await
        .unwrap();
    let got: Vec<Option<Plain>> = sqlx::query_scalar("SELECT status FROM t ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(got, vec![Some(Plain::Active), None]);
}

// ── from_row / from_row_opt ────────────────────────────────────────────────

#[tokio::test]
async fn from_row_reads_valid_value() {
    let pool = pool().await;
    sqlx::raw_sql("CREATE TABLE t (status text NOT NULL);")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO t VALUES ($1)")
        .bind(Plain::Active.to_string())
        .execute(&pool)
        .await
        .unwrap();
    let row = sqlx::query("SELECT status FROM t")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(Plain::from_row(&row, 0).unwrap(), Plain::Active);
}

#[tokio::test]
async fn from_row_errors_on_invalid_value() {
    let pool = pool().await;
    sqlx::raw_sql("CREATE TABLE t (status text NOT NULL); INSERT INTO t VALUES ('Bogus');")
        .execute(&pool)
        .await
        .unwrap();
    let row = sqlx::query("SELECT status FROM t")
        .fetch_one(&pool)
        .await
        .unwrap();
    let err = Plain::from_row(&row, 0).unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
}

#[tokio::test]
async fn from_row_opt_handles_null_some_and_invalid() {
    let pool = pool().await;
    sqlx::raw_sql(
        "CREATE TABLE t (id serial PRIMARY KEY, status text);
         INSERT INTO t (status) VALUES (NULL), ('Active'), ('WhoKnows');",
    )
    .execute(&pool)
    .await
    .unwrap();
    let rows = sqlx::query("SELECT status FROM t ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(Plain::from_row_opt(&rows[0], 0).unwrap(), None);
    assert_eq!(
        Plain::from_row_opt(&rows[1], 0).unwrap(),
        Some(Plain::Active)
    );
    let err = Plain::from_row_opt(&rows[2], 0).unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
}
