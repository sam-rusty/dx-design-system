//! Postgres error → user-facing message mapping.
//!
//! [`ErrorFormatting::sqlx`] classifies a [`sqlx::Error`] (via the Postgres error
//! code, falling back to message-text matching) and resolves a friendly message
//! from the [`phf`] constraint maps below, returning the appropriate [`AppError`]
//! variant. The status code is implied by the variant (see [`AppError::status_code`]).
//!
//! Map keys are the constraint / index names from `PgDatabaseError::constraint()`
//! (unique, foreign-key, check violations) or the `table.column` pair from
//! `PgDatabaseError::table()` / `column()` (not-null violations — Postgres does not
//! report a constraint name for those). Native enum columns reject invalid values at
//! the type level (not a CHECK violation), so those former `IN (...)` entries do not apply.

use phf::phf_map;
use sqlx::error::ErrorKind;
use validator::ValidationErrors;

use crate::AppError;

/// `constraint name → (column label, optional custom message)`.
///
/// When the custom message is `None`, the resolved message is `"{label} {suffix}"`
/// (the suffix is supplied per violation kind, e.g. `"already exists"`). When it is
/// `Some(msg)`, `msg` is used verbatim and the suffix is ignored.
type ErrorMessagesMap = phf::Map<&'static str, (&'static str, Option<&'static str>)>;

static UNIQUE_VIOLATIONS: ErrorMessagesMap = phf_map! {
    "integrations_workspace_id_application_account_id_key" =>
        ("", Some("This integration is already connected for this account")),
    "automation_step_branch_conditions_step_condition_key" =>
        ("", Some("This branch condition is already linked to this step")),
    "automation_trigger_conditions_automation_condition_key" =>
        ("", Some("This trigger condition is already on this automation")),
    "idx_executions_event_dedup" =>
        ("", Some("This automation has already run for this contact and event")),
    "workspace_users_user_id_workspace_id_key" =>
        ("", Some("This user is already a member of this workspace")),
    "workspace_users_owner_idx" => ("", Some("This workspace already has an owner")),
    "person_meta_hgi_client_id_key" => ("", Some("This HGI client ID is already linked to a contact")),
    "person_meta_hgi_code_key" => ("", Some("This HGI code is already linked to a contact")),
    "idx_pipeline_stages_initial" => ("", Some("This pipeline already has an initial stage")),
    "idx_person_addresses_person_id_unique" => ("", Some("This contact already has an address record")),
    "goals_workspace_id_user_id_kind_period_key" =>
        ("", Some("A goal for this metric and period already exists")),
    "users_email_key" => ("", Some("A user with this email already exists")),
    "users_username_key" => ("", Some("This username is already taken")),
    "users_code_key" => ("", Some("A user with this code already exists")),
    "idx_dispatches_one_scheduled" =>
        ("", Some("This automation already has a pending scheduled run")),
};

/// Keyed by foreign-key constraint name (`<table>_<column>_fkey`). The label is the
/// referenced (parent) entity, reused for both directions: insert against a missing
/// parent → `"{label} not found"`; delete of a still-referenced parent →
/// `"{label} is referenced by another record and cannot be deleted"`.
static FOREIGN_KEY_VIOLATIONS: ErrorMessagesMap = phf_map! {
    "activities_person_id_fkey" => ("Contact", None),
    "activities_user_id_fkey" => ("User", None),
    "activities_workspace_id_fkey" => ("Workspace", None),
    "aristotle_activity_dates_person_id_fkey" => ("Contact", None),
    "collection_members_collection_id_fkey" => ("Collection", None),
    "collection_members_person_id_fkey" => ("Contact", None),
    "collections_workspace_id_fkey" => ("Workspace", None),
    "financial_analysis_person_id_fkey" => ("Contact", None),
    "financial_analysis_workspace_id_fkey" => ("Workspace", None),
    "goals_user_id_fkey" => ("User", None),
    "goals_workspace_id_fkey" => ("Workspace", None),
    "integrations_created_by_fkey" => ("User", None),
    "integrations_workspace_id_fkey" => ("Workspace", None),
    "person_addresses_person_id_fkey" => ("Contact", None),
    "person_meta_created_by_fkey" => ("User", None),
    "person_meta_person_id_fkey" => ("Contact", None),
    "person_meta_spouse_person_id_fkey" => ("Contact", None),
    "person_notes_person_id_fkey" => ("Contact", None),
    "person_notes_user_id_fkey" => ("User", None),
    "person_notes_workspace_id_fkey" => ("Workspace", None),
    "person_pipeline_stage_person_id_fkey" => ("Contact", None),
    "person_pipeline_stage_pipeline_id_fkey" => ("Collection", None),
    "person_pipeline_stage_stage_id_fkey" => ("Pipeline stage", None),
    "pipeline_stages_pipeline_id_fkey" => ("Collection", None),
    "tasks_person_id_fkey" => ("Contact", None),
    "tasks_user_id_fkey" => ("User", None),
    "tasks_workspace_id_fkey" => ("Workspace", None),
    "automation_audit_logs_user_id_fkey" => ("User", None),
    "automation_conditions_collection_id_fkey" => ("Collection", None),
    "automation_conditions_pipeline_stage_id_fkey" => ("Pipeline stage", None),
    "automation_conditions_automation_id_fkey" => ("Automation", None),
    "automation_step_branch_conditions_automation_condition_id_fkey" => ("Automation condition", None),
    "automation_step_branch_conditions_automation_step_id_fkey" => ("Automation step", None),
    "automation_steps_branch_if_false_next_step_id_fkey" => ("Automation step", None),
    "automation_steps_collection_id_fkey" => ("Collection", None),
    "automation_steps_integration_id_fkey" => ("Integration", None),
    "automation_steps_next_step_id_fkey" => ("Automation step", None),
    "automation_steps_pipeline_stage_id_fkey" => ("Pipeline stage", None),
    "automation_steps_automation_id_fkey" => ("Automation", None),
    "automation_trigger_conditions_automation_condition_id_fkey" => ("Automation condition", None),
    "automation_trigger_conditions_automation_id_fkey" => ("Automation", None),
    "automations_created_by_fkey" => ("User", None),
    "automations_template_id_fkey" => ("Automation template", None),
    "automations_workspace_id_fkey" => ("Workspace", None),
    "automation_dispatches_automation_id_fkey" => ("Automation", None),
    "automation_dispatch_people_dispatch_id_fkey" => ("Automation dispatch", None),
    "automation_dispatch_people_person_id_fkey" => ("Contact", None),
    "automation_dispatch_steps_dispatch_id_fkey" => ("Automation dispatch", None),
    "automation_step_executions_dispatch_id_fkey" => ("Automation dispatch", None),
    "automation_step_executions_dispatch_person_id_fkey" => ("Automation dispatch recipient", None),
    "automation_event_dedup_automation_id_fkey" => ("Automation", None),
    "automation_event_dedup_person_id_fkey" => ("Contact", None),
    "workspace_users_user_id_fkey" => ("User", None),
    "workspace_users_workspace_id_fkey" => ("Workspace", None),
};

/// Keyed by `table.column`, matched against `PgDatabaseError::table()` + `column()`.
/// The first tuple element holds the full message (the suffix template is unused for
/// not-null violations, mirroring the upstream behaviour).
static NOT_NULL_VIOLATIONS: ErrorMessagesMap = phf_map! {
    "people.first_name" => ("Contact first name is required", None),
    "people.last_name" => ("Contact last name is required", None),
    "users.username" => ("Username is required", None),
    "financial_analysis.workspace_id" => ("Analysis must be linked to a workspace", None),
    "financial_analysis.person_id" => ("Analysis must be linked to a contact", None),
    "financial_analysis.status" => ("Status is required", None),
    "financial_analysis.analysis_type" => ("Analysis type is required", None),
    "financial_analysis.payload" => ("Analysis data is required", None),
};

/// Check violations carry a complete message directly — no label/suffix template.
static CHECK_VIOLATIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "automation_conditions_collection_id_check" =>
        "Automation condition must reference a collection where required",
    "automation_conditions_pipeline_stage_id_check" =>
        "Automation condition must reference a pipeline stage where required",
    "automation_steps_step_type_shape" => "Automation step fields do not match its step type",
    "automation_steps_collection_id_check" => "This automation action requires a collection",
    "automation_steps_pipeline_stage_id_check" => "Move-to-stage actions require a pipeline stage",
    "automation_steps_send_email_check" => "Send-email steps require a recipient",
    "automation_steps_meta_check" => "This action is missing required settings",
    "automations_check" => "Non-template automations must belong to a workspace",
    "automations_check1" => "Templates cannot reference another template",
    "goals_target_count_check" => "Goal target must be greater than zero",
};

pub struct ErrorMessages;

impl ErrorMessages {
    fn unique_violations(key: &str) -> Option<String> {
        Self::error_message(key, &UNIQUE_VIOLATIONS, "already exists")
    }

    fn foreign_key_violations(key: &str) -> Option<String> {
        Self::error_message(key, &FOREIGN_KEY_VIOLATIONS, "not found")
    }

    fn foreign_key_violations_on_delete(key: &str) -> Option<String> {
        Self::error_message(
            key,
            &FOREIGN_KEY_VIOLATIONS,
            "is referenced by another record and cannot be deleted",
        )
    }

    fn check_violations(key: &str) -> Option<String> {
        CHECK_VIOLATIONS.get(key).map(|message| message.to_string())
    }

    fn not_null_violations(key: &str) -> Option<String> {
        NOT_NULL_VIOLATIONS
            .get(key)
            .map(|(message, _)| message.to_string())
    }

    /// Parse a SQLx not-null constraint error message into a `table.column` lookup key
    /// for [`NOT_NULL_VIOLATIONS`]. The key keeps the raw snake_case column so it matches
    /// the map entries.
    /// Example input: 'null value in column "first_name" of relation "people" violates not-null constraint'
    /// Example output: Some("people.first_name")
    pub fn parse_not_null_error_to_key(error_message: &str) -> Option<String> {
        if !error_message.contains("violates not-null constraint") {
            return None;
        }

        let mut parts = error_message.split('"');
        let column = parts.nth(1)?;
        let table = parts.nth(1)?;

        Some(format!("{table}.{column}"))
    }

    /// Turn a snake_case column into a sentence-case label: `first_name` → `First name`.
    fn humanize_column(column: &str) -> String {
        let spaced = column.replace('_', " ");
        let mut chars = spaced.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().chain(chars).collect(),
            None => String::new(),
        }
    }

    /// Look up `key`; render `custom` verbatim, or `"{label} {suffix}"` otherwise.
    fn error_message(key: &str, map: &ErrorMessagesMap, suffix: &str) -> Option<String> {
        map.get(key).map(|(label, custom)| match custom {
            Some(message) => message.to_string(),
            None => format!("{label} {suffix}"),
        })
    }
}

pub struct ErrorFormatting;

impl ErrorFormatting {
    /// Convert a `sqlx::Error` into the most specific `AppError` we can. Known
    /// constraint violations map to friendly, client-safe messages; everything else
    /// collapses to `InternalServer` so raw database text is never leaked.
    pub fn sqlx(db_error: sqlx::Error) -> AppError {
        let msg = db_error.to_string();
        match db_error.into_database_error() {
            None => AppError::InternalServer(msg),
            Some(error) => {
                let error_message = error.to_string();
                let kind = error.kind();
                let error_code = error.code().map(|c| c.to_string());

                match error.constraint() {
                    Some(key) => {
                        match Self::determine_error_type(&error_message, kind, &error_code) {
                            ErrorKind::UniqueViolation => AppError::Conflict(
                                ErrorMessages::unique_violations(key).unwrap_or(error_message),
                            ),
                            ErrorKind::ForeignKeyViolation => {
                                if error_message.contains("delete on table")
                                    || error_message.contains("update or delete on table")
                                {
                                    AppError::BadRequest(
                                        ErrorMessages::foreign_key_violations_on_delete(key)
                                            .unwrap_or(error_message),
                                    )
                                } else {
                                    AppError::NotFound(
                                        ErrorMessages::foreign_key_violations(key)
                                            .unwrap_or(error_message),
                                    )
                                }
                            }
                            ErrorKind::CheckViolation => AppError::BadRequest(
                                ErrorMessages::check_violations(key).unwrap_or(error_message),
                            ),
                            ErrorKind::NotNullViolation => AppError::BadRequest(
                                ErrorMessages::not_null_violations(key).unwrap_or(error_message),
                            ),
                            _ => AppError::InternalServer(error_message),
                        }
                    }
                    None => {
                        // we don't get constraint name for not null violation we have to handle it separately
                        if kind == ErrorKind::NotNullViolation
                            || error_code.as_deref() == Some("23502")
                        {
                            let error_message = error.to_string();

                            if let Some(key) =
                                ErrorMessages::parse_not_null_error_to_key(&error_message)
                                && let Some(custom_error) = ErrorMessages::not_null_violations(&key)
                            {
                                return AppError::Validation(
                                    custom_error,
                                    ValidationErrors::default(),
                                );
                            }

                            // Fallback to extracting column name from error message
                            let mut parts = error_message.split('"');
                            if let Some(column) = parts.nth(1) {
                                return AppError::Validation(
                                    format!(
                                        "{} is required",
                                        ErrorMessages::humanize_column(column)
                                    ),
                                    ValidationErrors::default(),
                                );
                            }
                        }
                        // No friendlier mapping: keep the real Postgres message in the variant.
                        // The HTTP `into_response` still masks it on the wire, but logs and
                        // non-HTTP callers (e.g. the dispatcher) get the real cause.
                        AppError::InternalServer(error_message)
                    }
                }
            }
        }
    }

    fn determine_error_type(
        error_message: &str,
        kind: ErrorKind,
        error_code: &Option<String>,
    ) -> ErrorKind {
        // First check PostgreSQL error code if available
        if let Some(code_str) = error_code {
            match code_str.as_ref() {
                "23505" => return ErrorKind::UniqueViolation,
                "23503" => return ErrorKind::ForeignKeyViolation,
                "23514" => return ErrorKind::CheckViolation,
                "23502" => return ErrorKind::NotNullViolation,
                _ => {} // Continue to next check
            }
        }

        if kind != ErrorKind::Other {
            return kind;
        }

        if error_message.contains("duplicate key") || error_message.contains("unique constraint") {
            ErrorKind::UniqueViolation
        } else if error_message.contains("foreign key constraint") {
            ErrorKind::ForeignKeyViolation
        } else if error_message.contains("check constraint") {
            ErrorKind::CheckViolation
        } else if error_message.contains("not-null constraint") {
            ErrorKind::NotNullViolation
        } else {
            ErrorKind::Other
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorFormatting;
    use crate::Connection;
    use crate::test::TestApp;

    const WS: &str = "11111111-1111-1111-1111-111111111111";
    const USER: &str = "22222222-2222-2222-2222-222222222222";

    /// A fresh template clone seeded with one workspace + user so each test can
    /// trigger a real Postgres constraint violation against the actual schema.
    async fn pool() -> Connection {
        let pool = TestApp::setup_db(env!("CARGO_MANIFEST_DIR"), "").await;
        sqlx::query!("INSERT INTO workspaces (id) VALUES (($1::text)::uuid)", WS)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query!(
            "INSERT INTO users (id, first_name, last_name, email, username, phone)
             VALUES (($1::text)::uuid, 'Alice', 'A', 'alice@test.com', 'alice', '5555550123')",
            USER
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn unique_constraint_returns_conflict() {
        let pool = pool().await;
        let err = sqlx::query!(
            "INSERT INTO users (id, first_name, last_name, email, username, phone)
             VALUES (gen_random_uuid(), 'Bob', 'B', 'other@test.com', 'alice', '5555550123')"
        )
        .execute(&pool)
        .await
        .unwrap_err();

        let app_err = ErrorFormatting::sqlx(err);
        assert_eq!(app_err.to_string(), "This username is already taken");
        assert_eq!(app_err.status_code(), http::StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn unique_email_nocase_returns_conflict() {
        let pool = pool().await;
        // `email` is `citext`, so a different-cased duplicate still collides.
        let err = sqlx::query!(
            "INSERT INTO users (id, first_name, last_name, email, username, phone)
             VALUES (gen_random_uuid(), 'Eve', 'E', 'ALICE@TEST.COM', 'eve', '5555550123')"
        )
        .execute(&pool)
        .await
        .unwrap_err();

        let app_err = ErrorFormatting::sqlx(err);
        assert_eq!(app_err.to_string(), "A user with this email already exists");
        assert_eq!(app_err.status_code(), http::StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn not_null_constraint_returns_bad_request() {
        let pool = pool().await;
        let err = sqlx::query!(
            "INSERT INTO users (id, first_name, last_name, email, username, phone)
             VALUES (gen_random_uuid(), NULL, 'X', 'x@test.com', 'xuser', '5555550123')"
        )
        .execute(&pool)
        .await
        .unwrap_err();

        let app_err = ErrorFormatting::sqlx(err);
        assert_eq!(app_err.to_string(), "First name is required");
        assert_eq!(app_err.status_code(), http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn check_constraint_returns_bad_request() {
        let pool = pool().await;
        // `goals.target_count` carries `CHECK (target_count > 0)`, mapped to
        // `goals_target_count_check`.
        let err = sqlx::query!(
            "INSERT INTO goals (id, workspace_id, user_id, kind, period, target_count)
             VALUES (gen_random_uuid(), ($1::text)::uuid, ($2::text)::uuid, 'Email', 'Daily', 0)",
            WS,
            USER
        )
        .execute(&pool)
        .await
        .unwrap_err();

        let app_err = ErrorFormatting::sqlx(err);
        assert_eq!(app_err.to_string(), "Goal target must be greater than zero");
        assert_eq!(app_err.status_code(), http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn fk_insert_returns_not_found() {
        let pool = pool().await;
        // References a workspace that does not exist → `goals_workspace_id_fkey`.
        let err = sqlx::query!(
            "INSERT INTO goals (id, workspace_id, user_id, kind, period, target_count)
             VALUES (
                gen_random_uuid(),
                '33333333-3333-3333-3333-333333333333'::uuid,
                ($1::text)::uuid, 'Email', 'Daily', 5
             )",
            USER
        )
        .execute(&pool)
        .await
        .unwrap_err();

        let app_err = ErrorFormatting::sqlx(err);
        assert_eq!(app_err.to_string(), "Workspace not found");
        assert_eq!(app_err.status_code(), http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn fk_on_delete_returns_bad_request() {
        let pool = pool().await;
        // `automation_audit_logs.user_id REFERENCES users(id)` has no ON DELETE clause
        // (NO ACTION), so deleting a still-referenced user violates
        // `automation_audit_logs_user_id_fkey` on the delete side.
        sqlx::query!(
            "INSERT INTO automation_audit_logs (id, workspace_id, user_id, action, target_id)
             VALUES (gen_random_uuid(), ($1::text)::uuid, ($2::text)::uuid, 'note', gen_random_uuid())",
            WS,
            USER
        )
        .execute(&pool)
        .await
        .unwrap();

        let err = sqlx::query!("DELETE FROM users WHERE id = ($1::text)::uuid", USER)
            .execute(&pool)
            .await
            .unwrap_err();

        let app_err = ErrorFormatting::sqlx(err);
        assert_eq!(
            app_err.to_string(),
            "User is referenced by another record and cannot be deleted"
        );
        assert_eq!(app_err.status_code(), http::StatusCode::BAD_REQUEST);
    }

    // #[tokio::test]
    // async fn unknown_error_returns_internal_server() {
    //     let pool = pool().await;
    //     // Stays a runtime `query()`: `nonexistent` is not in the schema, so the
    //     // compile-time `query!` macro cannot describe it. The whole point of this
    //     // test is to exercise the unknown-table error path.
    //     let err = sqlx::query("INSERT INTO nonexistent VALUES ('x')")
    //         .execute(&pool)
    //         .await
    //         .unwrap_err();

    //     let app_err = ErrorFormatting::sqlx(err);
    //     assert_eq!(app_err.to_string(), "Internal Server Error");
    //     assert_eq!(
    //         app_err.status_code(),
    //         http::StatusCode::INTERNAL_SERVER_ERROR
    //     );
    // }
}
