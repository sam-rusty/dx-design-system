# Utils

Shared error handling and formatting helpers used across the workspace (`ds-utils`).

## Key exports

- `DsError`, `Result<T>` — unified error type (`error.rs`): `InternalServer(String)`,
  `Other(String)`, `Validation(String, ValidationErrors)`. `From` impls for `String`, `&str`,
  `serde_json::Error` and `validator::ValidationErrors` let callers use `?` / `.into()`.
  `DsError::form_field_error(field, message)` builds a single-field `ValidationErrors`.
- `format::*` — `date`, `money`, `number`, `percent`, `phone`, `text` formatters
  - `money`: `major_to_minor("1,234.56", 2) -> Some("123456")`, `minor_to_major("2550", 2) -> "25.5"`
  - `number`: `format_number("1234567") -> "1,234,567"`, `parse_number`
  - `percent`: `format_percent`, `parse_percent`, `filter_percent`, `clamp_percent`
  - `phone`: `PhoneFormat`, `format_phone`, `format_phone_with`
  - `text`: `merge(&[..])`, `snake_to_title`
- `classes!` macro — joins non-empty class strings with spaces

## Features

- `web` — enables `dioxus/web` and `web-sys/Storage`
- `server` — enables `dioxus/server` and `http`
