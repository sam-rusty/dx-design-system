# Macros

Procedural macros for the FNA project. Provides derive macros and a route definition macro used by the `components` crate.

## Exports

| Macro | Kind | Purpose |
|---|---|---|
| `FormFields` | Derive | Generates typed field name constants for structs |
| `FormOptions` | Derive | Generates `OPTIONS` constant for unit enums |
| `Steps` | Derive | Generates step metadata for multi-step form enums |
| `on_server!` | Function | Applies `#[cfg(feature = "server")]` to every item inside the block |
| `on_web!` | Function | Applies `#[cfg(feature = "web")]` to every item inside the block |

---

## `FormFields`

Generates typed field constants on a struct. Scalar/nested fields produce `FieldName` (supports `.dot()` for nesting), `Vec<_>` fields produce `FieldArray` (supports `.at(index)`).

### Plain fields

```rust
#[derive(FormFields)]
struct LoginForm {
    email: String,
    password: String,
}

// Generates:
// LoginForm::email    -> FieldName("email")
// LoginForm::password -> FieldName("password")

view! { <FormItem name=LoginForm::email /> }
```

### Nested fields

```rust
#[derive(FormFields)]
struct Address {
    street: String,
    zip: String,
}

#[derive(FormFields)]
struct UserForm {
    name: String,
    address: Address,
}

// Flat access:
// UserForm::name -> FieldName("name")

// Nested dot-path via .dot():
// UserForm::address.dot(Address::street) -> FieldPath("address.street")

view! { <FormItem name=UserForm::address.dot(Address::street) /> }
```

### Array fields

```rust
#[derive(FormFields)]
struct Item {
    name: String,
    qty: i32,
}

#[derive(FormFields)]
struct OrderForm {
    title: String,
    items: Vec<Item>,
}

// .dot() on a Vec field is a compile error — use .at(index) first:
// OrderForm::items.at(0).dot(Item::name) -> FieldPath("items.0.name")
// OrderForm::items.at(2).dot(Item::qty)  -> FieldPath("items.2.qty")

for i in 0..item_count {
    view! {
        <FormItem name=OrderForm::items.at(i).dot(Item::name) />
        <FormItem name=OrderForm::items.at(i).dot(Item::qty) />
    }
}
```

### Custom labels

Labels are auto-generated from field names (`snake_case` -> `Title Case`). Override with `#[field(label = "...")]`:

```rust
#[derive(FormFields)]
struct Dime {
    #[field(label = "First Name")]
    first_name: String,
    date_of_birth: String,
}

// Dime::first_name.label    -> "First Name"
// Dime::date_of_birth.label -> "Date Of Birth"  (auto-generated)
```

---

## `FormOptions`

Generates `const OPTIONS: &[(&str, &str)]` on unit enums. The first element is the serialized value (respects `#[serde(rename_all)]` and per-variant `#[serde(rename)]`). The second is a human-readable label: per-variant `#[strum(to_string = "...")]` when present, otherwise `PascalCase` -> `Title Case`.

### Basic usage

```rust
#[derive(FormOptions)]
#[serde(rename_all = "lowercase")]
enum Role {
    Admin,
    User,
    SuperAdmin,
}

// Role::OPTIONS -> &[("admin", "Admin"), ("user", "User"), ("superadmin", "Super Admin")]

view! { <Select options=Role::OPTIONS /> }
view! { <RadioGroup options=Role::OPTIONS /> }
```

### Per-variant rename

```rust
#[derive(FormOptions)]
#[serde(rename_all = "kebab-case")]
enum Status {
    Active,
    #[serde(rename = "on_hold")]
    OnHold,
}

// Status::OPTIONS -> &[("active", "Active"), ("on_hold", "On Hold")]
```

### Supported `rename_all` rules

`lowercase`, `UPPERCASE`, `camelCase`, `PascalCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE`

---

## `Steps`

Derives step metadata for multi-step form enums. Generates `ALL`, `COUNT`, `TITLES`, and `DESCRIPTIONS` constants. Used with the stepper/multi-step form system in `components`.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Steps)]
pub enum MyStep {
    #[step(title = "First", description = "The first step.")]
    StepOne,
    #[step(title = "Second")]
    StepTwo,
}

// MyStep::ALL         -> &[MyStep::StepOne, MyStep::StepTwo]
// MyStep::COUNT       -> 2
// MyStep::TITLES      -> &["First", "Second"]
// MyStep::DESCRIPTIONS -> &["The first step.", ""]
```

- `title` defaults to the variant name converted from `PascalCase` to `Title Case` if omitted.
- `description` defaults to `""` if omitted.
- Only unit variants are supported.

---

## `on_server!` / `on_web!`

Function-like macros that apply `#[cfg(feature = "server")]` or `#[cfg(feature = "web")]` to every item inside the block — useful when many adjacent items share the same feature gate.

```rust
macros::on_server! {
    use std::str::FromStr;
    use crate::AutomationStepKind;

    pub fn server_only_helper() { /* ... */ }
}
```