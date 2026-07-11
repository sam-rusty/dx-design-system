use std::env::var;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::db::DB;

/// Name of the pre-built template database the harness clones from. Build it
/// with `make test-db-template` (re-run whenever a migration is added).
const TEMPLATE_DB: &str = "producer_pro_template";

/// Process-wide monotonic counter so each `setup_db` call gets a unique clone
/// name without touching random/time APIs.
static DB_SEQ: AtomicU64 = AtomicU64::new(0);

fn next_seq() -> u64 {
    DB_SEQ.fetch_add(1, Ordering::Relaxed)
}

/// Replace the path segment after the last `/` in a Postgres URL with `name`,
/// yielding a connection string for a sibling database on the same server.
fn swap_db_name(url: &str, name: &str) -> String {
    match url.rfind('/') {
        Some(i) => format!("{}/{}", &url[..i], name),
        None => name.to_string(),
    }
}

/// True when `s` contains only blank lines and SQL line comments — i.e. there
/// is no executable SQL to run.
fn is_blank(s: &str) -> bool {
    s.lines()
        .all(|l| l.trim().is_empty() || l.trim_start().starts_with("--"))
}

pub struct TestApp;

impl TestApp {
    /// Clone the `producer_pro_template` database into a fresh, uniquely-named
    /// Postgres database and return a pool connected to it. The schema comes
    /// from the template (built by `make test-db-template`); only the fixture
    /// data is applied here.
    ///
    /// Clones are intentionally left behind on test exit — per-test teardown
    /// would add a DROP round-trip to every test and races with pooled
    /// connections that may still be closing. Run `make reset-db` (or drop the
    /// `pp_test_*` databases) to reclaim them.
    pub async fn setup_db(fixture_path: &str, fixture: &str) -> crate::Connection {
        use sqlx::AssertSqlSafe;

        let base = var("DATABASE_URL").expect("DATABASE_URL must be set");
        let name = format!("pp_test_{}_{}", std::process::id(), next_seq());

        // The clone name is server-generated (never user input), so the raw-SQL
        // DDL is safe. `CREATE DATABASE` cannot bind parameters.
        let admin = DB::build_pool(&base).await.unwrap();
        sqlx::raw_sql(AssertSqlSafe(format!(
            "CREATE DATABASE \"{name}\" TEMPLATE {TEMPLATE_DB}"
        )))
        .execute(&admin)
        .await
        .unwrap_or_else(|e| panic!("create test db {name} from template {TEMPLATE_DB}: {e}"));
        admin.close().await;

        let pool = DB::build_pool(&swap_db_name(&base, &name)).await.unwrap();

        // An empty fixture name means "schema only" — used by unit tests that
        // build their own temp tables.
        if !fixture.trim().is_empty() {
            let fixture_sql = Self::read_fixture_file(fixture_path, fixture);
            if !is_blank(&fixture_sql) {
                sqlx::raw_sql(AssertSqlSafe(fixture_sql))
                    .execute(&pool)
                    .await
                    .unwrap();
            }
        }
        pool
    }

    fn read_fixture_file(fixture_path: &str, fixture: &str) -> String {
        let path = Path::new(fixture_path)
            .join("tests")
            .join("fixtures")
            .join(fixture)
            .with_extension("sql");
        fs::read_to_string(path).unwrap()
    }
}
