# Utils

Shared primitives, types, error handling, and SSR helpers used across the workspace.

## Key exports
- `AppError`, `Result<T>` — unified error handling
- `UserClaim`, `AuthState` — auth helpers
- `AppEnv` (`state.rs`) — JWT secret + optional env vars (`APP_PUBLIC_URL`, `GOOGLE_OAUTH_CLIENT_ID`, `GOOGLE_OAUTH_CLIENT_SECRET`); loaded via `envy` / `Env::load()`; available as Axum `Extension`; `AppEnv::take()` in server fns; `google_oauth_config()` returns OAuth triplet or `ServiceUnavailable`
- `types::*` — domain types: `Date`, `DateTime`, `Email`, `Phone`, `Code`, `PersonId`, `FnaId`, `AutomationId`, etc.
- SSR helpers (server feature): `DB`, `Connection`, `Env`, JWT, middleware, test harness
- Codec aliases: `GET`, `POST`, `PUT`, `PATCH`, `DELETE` for server_fn HTTP verbs
