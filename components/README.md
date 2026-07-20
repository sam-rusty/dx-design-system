# Components crate — usage

How to use exported UI from `components`. The full export list is [`src/lib.rs`](src/lib.rs).

## Feature flags

Every component family ships behind a cargo feature; all are on by default
(`default = ["full"]`). To compile only what you use:

```toml
ds-components = { path = "../components", default-features = false, features = ["form", "nav"] }
```

| Feature | Contents | Implies |
|---|---|---|
| `form` | form store, inputs, checkbox/radio/select/textarea, slider, file upload, stepper, step dots | `feedback` |
| `calendar` | MonthView, TimeGrid | — |
| `date-picker` | DatePicker, DateRangePicker, DateTimePicker | `calendar`, `form`, `overlay` |
| `charts` | Donut, AreaLine, StackedBar | — |
| `data-table` | DataTable, ListView, ResourceView | `form`, `feedback` |
| `rich-text` | RichTextEditor | `form`, `overlay` |
| `nav` | NavTabs, Tabs, SegmentedControl, AppShellProvider, RouteTransitionOutlet | `overlay`, `feedback` |
| `feedback` | Toast, Alert, Tooltip, StatusDot, LoadingOverlay, Progress, fallback views | — |
| `overlay` | Modal, Popover, DropdownMenu | — |
| `display` | Accordion, Avatar, StatTile, EmptyState, SectionHeader | — |

Core primitives (Button, Text, Title, Icon, layout, Card, Badge, Link, Spinner,
Copyable, field-name schema types) are always compiled. `web` is orthogonal to
family flags. `validator` is only pulled in by `form`, `time` only by
`calendar`.

## Maintaining this file

- Document **usage only**: prop tables and `view!` / `rsx!` examples aligned with the current API.
- Do **not** add narrative comments, implementation notes, or design discussion except when **adding or changing** a component—in that case update **only** that section’s prop table and usage example.
- Do **not** add explanatory comments inside code samples unless they are part of the API (e.g. required imports).

## Table of contents

- [App shell & theme](#app-shell--theme)
- [Layout](#layout)
- [Alert](#alert)
- [Badge](#badge)
- [Copyable](#copyable)
- [Text & Title](#text--title)
- [Button](#button)
- [Card](#card)
- [Link](#link)
- [Back](#back)
- [Avatar](#avatar)
- [Separator](#separator)
- [Forms](#forms)
- [TextArea](#textarea)
- [RichTextEditor](#richtexteditor)
- [Checkbox](#checkbox)
- [RadioGroup](#radiogroup)
- [Select](#select)
- [DatePicker](#datepicker)
- [Calendar](#calendar)
- [DropdownMenu](#dropdownmenu)
- [NavTabs](#navtabs)
- [RouteTransitionOutlet](#routetransitionoutlet)
- [ListView](#listview)
- [MultiStepForm / Stepper](#multistepform--stepper)
- [FieldName & Field](#fieldname--field)
- [Derive macros](#derive-macros)
- [Spinner](#spinner)
- [LoadingOverlay](#loadingoverlay)
- [ResourceView](#resourceview)
- [use_action_feedback](#use_action_feedback)
- [Toaster / Toast](#toaster--toast)
- [ToggleCard](#togglecard)
- [Switch](#switch)
- [Tabs](#tabs)
- [SegmentedControl](#segmentedcontrol)
- [ChipToggle](#chiptoggle)
- [EmptyState](#emptystate)
- [IconBubble](#iconbubble)
- [SectionHeader](#sectionheader)
- [DataTable](#datatable)
- [Popover / PopoverConfirm](#popover--popoverconfirm)
- [Charts](#charts)
- [Tooltip](#tooltip)
- [Accordion](#accordion)
- [FileUpload](#fileupload)
- [Icon](#icon)
- [Modal](#modal)
- [Slider](#slider)
- [Progress](#progress)
- [NumberStepper](#numberstepper)
- [StatTile](#stattile)
- [StatusDot](#statusdot)
- [StepDots](#stepdots)
- [Portal](#portal)
- [Further exports](#further-exports)

---

## App shell & theme

| Export | Props / signature |
|--------|-------------------|
| `PageLoader` | `logo: Element` |
| `SectionLoader` | `logo: Element` |
| `AppRouteErrorFallback` | `ctx: ErrorContext`, `logo: Element` |
| `SectionErrorFallback` | `ctx: ErrorContext` |
| `NotFound` | `route: Vec<String>`, `logo: Element` |
| `WorkInProgress` | `title: String` |
| `AppShellProvider` | `children: Element` — provides `ToastStore`, mounts `Toaster`, dropdown coordination |

```rust
use components::AppShellProvider;

rsx! {
    AppShellProvider {
        Router::<Route> {}
    }
}
```

---

## Layout

| Type | Variants |
|------|----------|
| `FlexGap` | `None`, `Xs`, `Sm`, **`Md`**, `Lg`, `Xl`, `Xxl` |
| `FlexAlign` | **`Stretch`**, `Start`, `Center`, `End`, `Baseline` |
| `FlexJustify` | **`Start`**, `Center`, `End`, `Between`, `Around`, `Evenly` |
| `FlexWrap` | **`NoWrap`**, `Wrap`, `WrapReverse` |
| `FlexDirection` | **`Row`**, `Column`, `RowReverse`, `ColumnReverse` |
| `FlexGridCols` | **`None`**, `C1`–`C7`, `C12` |

### `Row`

| Prop | Type | Default |
|------|------|---------|
| `align` | `FlexAlign` | `Stretch` |
| `justify` | `FlexJustify` | `Start` |
| `gap` | `FlexGap` | `Md` |
| `wrap` | `FlexWrap` | `NoWrap` |
| `class` | `String` | `""` |
| `children` | `Element` | required |

```rust
use components::{Row, FlexAlign, FlexJustify, FlexGap};

view! {
    <Row align=FlexAlign::Center justify=FlexJustify::Between gap=FlexGap::Lg>
        <span>"Left"</span>
        <span>"Right"</span>
    </Row>
}
```

### `Column`

Same props as `Row` (flex column).

```rust
use components::{Column, FlexAlign, FlexGap};

view! {
    <Column align=FlexAlign::Start gap=FlexGap::Sm>
        <span>"First"</span>
        <span>"Second"</span>
    </Column>
}
```

### `Flex`

| Prop | Type | Default |
|------|------|---------|
| `direction` | `FlexDirection` | `Row` |
| `align` / `justify` / `wrap` / `gap` / `class` | same as `Row` | defaults per `#[props(default)]` |

```rust
use components::{Flex, FlexDirection, FlexGap};

view! {
    <Flex direction=FlexDirection::Column gap=FlexGap::Md>
        <span>"Item"</span>
    </Flex>
}
```

### `Grid`

| Prop | Type | Default |
|------|------|---------|
| `class` | `String` | `""` |
| `cols` | `FlexGridCols` | `None` |
| `gap` | `FlexGap` | `Md` |
| `align` | `FlexAlign` | `Stretch` |
| `justify` | `FlexJustify` | `Start` |
| `children` | `Element` | required |

```rust
use components::{Grid, FlexGridCols, FlexGap};

view! {
    <Grid cols=FlexGridCols::C3 gap=FlexGap::Md>
        <div>"1"</div>
        <div>"2"</div>
        <div>"3"</div>
    </Grid>
}
```

### `Container`

| Prop | Type | Default |
|------|------|---------|
| `children` | `Element` | required |

Renders `<main>`.

---

## Alert

| Prop | Type | Default |
|------|------|---------|
| `variant` | `AlertVariant` | `Destructive` |
| `class` | `String` | `""` |
| `children` | `Element` | required |

`AlertVariant`: `Destructive`, `Warning`, `Info`, `Success`.

```rust
use components::{Alert, AlertVariant};

view! {
    <Alert variant=AlertVariant::Info>"Notice"</Alert>
    <Alert variant=AlertVariant::Success>"Done"</Alert>
}
```

---

## Badge

| Prop | Type | Default |
|------|------|---------|
| `variant` | `BadgeVariant` | `Default` |
| `size` | `BadgeSize` | `Sm` |
| `class` | `String` | `""` |
| `children` | `Element` | required |

`BadgeSize`: `Xs`, `Sm`, `Md`, `Lg`.

```rust
use components::{Badge, BadgeVariant, BadgeSize};

view! {
    <Badge>"Default"</Badge>
    <Badge variant=BadgeVariant::Primary size=BadgeSize::Md>"Completed"</Badge>
}
```

---

## Copyable

| Prop | Type | Default |
|------|------|---------|
| `text` | `String` | required |
| `class` | `String` | `""` |

```rust
use components::Copyable;

view! {
    <Copyable text="john@example.com" />
    <Copyable text=user.phone.clone() class="font-mono text-sm" />
}
```

---

## Text & Title

### `Text`

| Prop | Type | Default |
|------|------|---------|
| `variant` | `TextVariant` | `Default` |
| `size` | `TextSize` | `Default` |
| `tone` | `TextTone` | `Default` |
| `weight` | `TextWeight` | `Normal` |
| `inline` | `bool` | `false` |
| `class` | `String` | `""` |
| `onclick` | `EventHandler<MouseEvent>` | no-op |
| `children` | `Element` | required |

`TextTone`: `Default`, `Muted`, `Primary`, `Warning`, `Destructive`, `Success`. `TextWeight`: `Normal`, `Medium`, `Semibold`, `Bold`.

### `Title`

Emits the heading element matching `size` (`H1`..`H6` → `<h1>`..`<h6>`; `Default` → `<h1>`) for a correct document outline.

| Prop | Type | Default |
|------|------|---------|
| `size` | `TitleSize` | `Default` |
| `class` | `String` | `""` |
| `children` | `Element` | required |

```rust
use components::{Text, TextVariant, TextSize, Title, TitleSize};

view! {
    <Text variant=TextVariant::Secondary size=TextSize::Small>"Muted"</Text>
    <Title size=TitleSize::H2>"Page"</Title>
}
```

---

## Button

| Prop | Type | Default |
|------|------|---------|
| `variant` | `ButtonVariant` | `Default` |
| `size` | `ButtonSize` | `Default` |
| `class` | `String` | `""` |
| `icon` | `Option<IconName>` | `None` (leading icon, before children) |
| `loading` | `bool` | `false` (disables + swaps `icon` for a spinner) |
| `disabled` | `bool` | `false` |
| `button_type` | `&'static str` | `"button"` |
| `bare` | `bool` | `false` |
| `onclick` | `EventHandler<MouseEvent>` | no-op |
| `attributes` | `Vec<Attribute>` (`extends = GlobalAttributes`) | `[]` (e.g. `aria-label`, `id`, `data-*`) |
| `children` | `Element` | required |

`loading` also sets `aria-busy="true"`. For a button-styled link, use `Link { r#type: LinkType::Button, .. }` — `Button` has no `href`.

```rust
use components::{Button, ButtonVariant, ButtonSize, IconName};

rsx! {
    Button { "Primary" }
    Button { variant: ButtonVariant::Outline, "Outline" }
    // Leading icon + built-in loading spinner (replaces the icon while pending):
    Button { icon: IconName::Trash, loading: delete_action.pending(), "Delete" }
    // Icon-only button needs an accessible name via the GlobalAttributes passthrough:
    Button { size: ButtonSize::Icon, "aria-label": "Close", Icon { name: IconName::X } }
}
```

---

## Card

| Prop | Type | Default |
|------|------|---------|
| `class` | `String` | `""` |
| `full_height` | `bool` | `false` |
| `compact` | `bool` | `false` |
| `variant` | `Option<CardVariant>` | `None` |
| `onclick` | `Option<EventHandler<()>>` | `None` |
| `attributes` | `Vec<Attribute>` (`extends = GlobalAttributes`) | `[]` |
| `children` | `Element` | required |

```rust
use components::Card;

rsx! {
    Card {
        h3 { "Title" }
    }
    Card { full_height: true, class: "col-span-6",
        "…"
    }
}
```

---

## Link

| Prop | Type | Default |
|------|------|---------|
| `to` | `impl Into<NavigationTarget>` | required |
| `type` | `Option<LinkType>` | `LinkType::Link` |
| `class` | `String` | `""` |
| `new_tab` | `bool` | `false` |
| `onclick_only` | `bool` | `false` |
| `active_class` | `Option<String>` | `None` |
| `title` | `Option<String>` | `None` |
| `style` | `Option<String>` | `None` |
| `variant` | `ButtonVariant` | `Default` (when `type: Button`) |
| `size` | `ButtonSize` | `Default` (when `type: Button`) |
| `onclick` | `EventHandler<MouseEvent>` | no-op |
| `children` | `Element` | required |

`LinkType::Button` renders the link with button styling (`variant` + `size`); `LinkType::Link` (default) renders a plain anchor.

```rust
use components::{Link, LinkType, ButtonVariant};

rsx! {
    Link { to: "/", class: "text-sm font-medium", "Dashboard" }
    Link { to: "/account", active_class: "text-primary font-bold".to_string(), "Account" }
    Link { to: "/new", r#type: LinkType::Button, variant: ButtonVariant::Default, "Create" }
}
```

---

## Back

| Prop | Type | Default |
|------|------|---------|
| `to` | `impl Into<NavigationTarget>` | required |
| `class` | `String` | `""` |

```rust
use components::Back;

rsx! {
    Back { to: "/dashboard" }
}
```

---

## Avatar

| Prop | Type | Default |
|------|------|---------|
| `class` | `String` | `""` |
| `style` | `String` | `""` |
| `src` | `Option<String>` | `None` |
| `alt` | `Option<String>` | `None` |
| `children` | `Element` | required |

Renders `src` as an image when set, falling back to `children` (initials) on load
error or when no `src` is given. With `alt`, the container is exposed as `role="img"`.

```rust
use components::Avatar;

view! {
    // initials fallback
    <Avatar class="h-20 w-20 text-2xl font-bold">"AS"</Avatar>
    // image with initials fallback
    <Avatar src="/avatars/ab.png".to_string() alt="Ada B.".to_string()>"AB"</Avatar>
}
```

---

## Separator

| Prop | Type | Default |
|------|------|---------|
| `orientation` | `Option<SeparatorOrientation>` | `Horizontal` |
| `decorative` | `bool` | `false` (when `true`, emits `role="none"` and drops `aria-orientation`) |
| `class` | `String` | `""` |

```rust
use components::{Separator, SeparatorOrientation};

view! {
    <Separator />
    <Separator orientation=SeparatorOrientation::Vertical class="h-6" />
    // Purely visual divider, hidden from assistive tech:
    <Separator decorative=true />
}
```

---

## Forms

`T` for `use_form::<T>()` must implement `Validate + Clone + Default + Serialize + Deserialize + FormSchema` (see `FormData`).

### `use_form` / `Form<T>`

Public fields: `values_signal`, `errors_signal`, `touched_signal`, `default_schema`, `required_fields`.

| Method | Role |
|--------|------|
| `error(&str)` | Error text for field |
| `is_touched(&str)` | Touched state |
| `set` / `set_string_value` | Set value |
| `get` / `get_untracked` / `get_or` / `has_value` | Typed read via `FieldKey` |
| `toggle_optional` | Optional nested object toggle |
| `touch_field` | Mark touched + validate |
| `reset` | Clear values, errors, touched |
| `default_values` | Hydrate from serializable value |
| `get_data` | Parse map → `T` |
| `validate_and_get` | Parse + `Validate` → `Option<T>` |
| `validate_fields(&[Field])` | Validate subset |
| `submit(fn(T))` | `validate_and_get` then callback |
| `set_server_error` | Populate field/global errors from a server `DsError` |

### `FormProvider`

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `form` | `Form<T>` | required | |
| `action` | `Option<FormSubmit<T>>` | `None` | Auto-wires loading state + submit |
| `loading` | `Option<Signal<bool>>` | `None` | Manual loading/disabled override |
| `inline_error` | `bool` | `true` | Render server errors in the form's global error slot; set `false` when errors are surfaced via toast |
| `children` | `Element` | required | |

When `action` is provided, FormProvider derives loading state from `action.pending()` and stores a submit callback that calls `form.validate_and_get()` then `action.call(data)`. Form auto-submits via this callback when no `on_submit` prop is given. On a failed `action`, the server `DsError` is fed to `form.set_server_error` so it renders below the submit button (unless `inline_error` is `false`).

### `FormSubmit<T>`

Type-erased action wrapper. Two ways to create:

```rust
// Simple: form type == action input type
FormSubmit::from(my_action)           // or my_action.into()

// Transform: form type != action input type
FormSubmit::with_transform(my_action, move |form_data: MyForm| {
    (form_data.into(), extra_ids())
})
```

### `Form` (HTML)

| Prop | Type | Default |
|------|------|---------|
| `class` | `String` | `""` |
| `on_submit` | `Option<EventHandler<FormEvent>>` | `None` |
| `children` | `Element` | required |

When `on_submit` is omitted and FormProvider has an `action`, Form auto-submits (validate → call action).

### `Input`

Generic form-bound input for HTML types without a dedicated wrapper (url, search, hidden, ...).
Prefer the typed wrappers (`TextInput`, `EmailInput`, ...) where one exists.

| Prop | Type | Default |
|------|------|---------|
| `field` | `impl Into<Field>` (`FormFields`) | required |
| `r#type` | `InputType` | required |
| `copyable` / `clearable` / `autofocus` | `bool` | `false` |
| `size` | `FieldSize` | `Default` |
| `tooltip` | `Option<Element>` | `None` — when set, a `circle-help` icon appears after the label and reveals this content on hover/focus |
| `class` | `String` | `""` — merged onto the field wrapper |

`InputType`: `Text`, `Email`, `Password`, `Number`, `Url`, `Tel`, `Search`, `Hidden`.

```rust
rsx! {
    TextInput {
        field: SignUp::name,
        tooltip: Some(rsx! { "Use your legal name as it appears on file." }),
    }
}
```

Override a field's visible label at the call site (binding name is unchanged):

```rust
rsx! {
    NumberInput { field: Field::from(MyForm::amount).with_label("Property value") }
}
```

`FieldSize`: `Default` (tall control), `Sm` (compact height and type scale), `Xs` (densest — `h-7`, `text-xs`, for inline/list edit fields). Shared by `Input*`, `TextArea`, and `Select`.

### `InputBase`

Standalone styled `<input>` (no `FormField` wrapper), controlled or uncontrolled. See [`input.rs`](src/input.rs) for the full prop list.

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `value` | `ReadSignal<Option<String>>` | `None` | `Some` = controlled (pair with `on_value_change`) |
| `default_value` | `String` | `""` | initial value when uncontrolled |
| `on_value_change` | `Callback<String>` | no-op | fires with the new value on every input event |
| `on_commit` | `Callback<String>` | no-op | fires with the committed value on the change event |
| `on_blur` / `on_key_down` | `Callback<FocusEvent>` / `Callback<KeyboardEvent>` | no-op | |
| `r#type` | `InputType` | `Text` | |
| `size` | `FieldSize` | `Default` | |
| `disabled` | `ReadSignal<bool>` | `false` | |
| `trailing` | `Option<Element>` | `None` | absolutely-positioned trailing adornment; requires a `relative` ancestor |
| `class` / `placeholder` / `id` / `autofocus` / `unstyled` / `aria_invalid` / `aria_describedby` | | | |
| ...attributes | extends `GlobalAttributes` + `input` | | `name`, `min`, `max`, `step`, `inputmode`, `readonly`, ... |

```rust
rsx! {
    InputBase {
        value: Some(query()),
        on_value_change: move |v: String| query.set(v),
        placeholder: "Search…",
    }
}
```

### Typed input bases

Standalone counterparts of the typed form inputs — each owns its type-specific behavior once, so
the same behavior works with or without a form: `TextInputBase`, `EmailInputBase`,
`PhoneInputBase` (formatted display, raw digits value), `NumberInputBase` (thousands-separated
display, raw decimal value, keystroke filter), `PercentageInputBase` (percent display, `min`/`max`
clamp on commit), `PasswordInputBase` (reveal toggle in the trailing slot).

All share `TypedInputBaseProps` (same shape as `InputBase` minus `r#type`/`trailing`);
`value` is always the *raw* value. `PercentageInputBase` adds `min`/`max: f64`.

### `ColorSwatchPicker`

A row of round color swatches. First option is selected by default; clicking selects (no deselect).

| Prop | Type | Default | Notes |
|------|------|---------|-------|
| `options` | `Vec<ColorSwatchOption>` | required | `{ value, class, label }`; `class` is a Tailwind bg class |
| `value` | `ReadSignal<Option<String>>` | required | currently selected `value` |
| `on_select` | `EventHandler<String>` | required | fires with chosen `value` (also once on mount to default to the first) |

```rust
rsx! {
    ColorSwatchPicker {
        options: vec![
            ColorSwatchOption { value: "Rose".into(), class: "bg-rose-300".into(), label: "Rose".into() },
            ColorSwatchOption { value: "Sky".into(),  class: "bg-sky-300".into(),  label: "Sky".into() },
        ],
        value: selected,
        on_select: move |v: String| selected.set(Some(v)),
    }
}
```

### `NumberInput` / `PhoneInput` / `TextInput` / `EmailInput` / `PasswordInput` / `PercentageInput`

All six share one props shape (`TypedInputProps`): `field` (required, `impl Into<Field>`),
`copyable`, `clearable`, `autofocus` (`bool`, default `false`), `size` (`FieldSize`), `tooltip`
(`Option<Element>` — `circle-help` hint after the label), `class` (merged onto the field wrapper).

`PercentageInput` adds `min` / `max` (`f64`, defaults `0.0` / `100.0`) — the value is clamped on
commit. `PasswordInput` includes a reveal (eye) toggle. Formatting/parsing behavior lives in the
corresponding typed base, so it is identical standalone and in forms.

### `MoneyInput`

Form-bound money input: the form value is a minor-unit integer string (binds an `i64` wire
field), the user types major units (`"25.5"` ↔ stored `"2550"`).

| Prop | Type | Default |
|------|------|---------|
| `field` | `impl Into<Field>` | required |
| `decimals` | `u32` | required — minor-unit exponent (2 → cents, 0 → zero-decimal) |
| `size` | `FieldSize` | `FieldSize::default()` |
| `autofocus` | `bool` | `false` |
| `tooltip` | `Option<Element>` | `None` |
| `class` | `String` | `""` |

```rust
rsx! {
    MoneyInput { field: PriceTierForm::amount, decimals: 2 }
}
```

### `PasswordStrength`

Strength meter (3 segments + Weak/Good/Strong label) and a live minimum-length checklist for a
form-bound password field. Reads the field's value from the surrounding form context — place it
beside the `PasswordInput` it describes.

| Prop | Type | Default |
|------|------|---------|
| `field` | `impl Into<Field>` | required |
| `min_len` | `usize` | `8` |
| `class` | `String` | `""` |
| `bar_class` | `String` | `"bg-primary"` |
| `bar_muted_class` | `String` | `"bg-muted"` |
| `check_class` | `String` | `"text-success"` |
| `check_muted_class` | `String` | `"text-muted-foreground"` |

```rust
rsx! {
    PasswordInput { field: SignupCreds::password }
    PasswordStrength { field: SignupCreds::password }
}
```

### `FormField`

| Prop | Type | Default |
|------|------|---------|
| `field` | `Field` | required |
| `children` | `Element` | required |

### `FormError`

| Prop | Type | Default |
|------|------|---------|
| `class` | `String` | `""` |
| `children` | `Option<Element>` | `None` |
| `errors` | `Option<Vec<String>>` | `None` |

### `FormSeparator`

`FormSeparator`: `class` (`String`, `""`), `children` (`Option<Element>`, `None` — optional inline label).

### Layout helpers

`FormSet`, `FormGroup`, `FormContent`, `FormTitle`, `FormDescription` — use inside `FormProvider` like other examples.

`FieldLabel` is the one field label for the form family: a static label stacked above the control.
Props: `class` (`String`, `""`), `html_for` (`String`, `""` — defaults to the surrounding field's
name), `tooltip` (`Option<Element>`, `None`), `children` (`Element`, required).

```rust
use components::form::{Form, FormGroup, FormProvider, FormSet, Input};
use components::{FormFields, FormSubmit, InputType, use_form};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Default, Serialize, Deserialize, Validate, FormFields)]
struct SignUp {
    name: String,
    email: String,
}

let form = use_form::<SignUp>();
let action = use_action(|data: SignUp| sign_up(data));

// With action — auto-validates, auto-submits, auto-loading
rsx! {
    FormProvider { form, action: FormSubmit::from(action),
        Form {
            FormSet {
                FormGroup {
                    Input { field: SignUp::name, r#type: InputType::Text }
                    Input { field: SignUp::email, r#type: InputType::Email, copyable: true }
                }
            }
        }
    }
}

// Manual on_submit still works (for custom pre-validation logic)
rsx! {
    FormProvider { form, loading: Some(my_loading_signal),
        Form { on_submit: move |_| { /* custom logic */ },
            // ...
        }
    }
}
```

---

## TextArea

| Prop | Type | Default |
|------|------|---------|
| `field` | `impl Into<Field>` | required |
| `autofocus` | `bool` | `false` |
| `rows` / `cols` / `minlength` / `maxlength` | `Option<u32>` | `None` |
| `size` | `FieldSize` | `Default` |
| `resize` | `TextAreaResize` | `Vertical` |
| `tooltip` | `Option<Element>` | `None` — `circle-help` hint after the label |
| `class` | `String` | `""` — merged onto the field wrapper |

```rust
use components::{TextArea, TextAreaResize};

rsx! {
    TextArea { field: MyForm::notes }
    TextArea { field: MyForm::bio, rows: 6, resize: TextAreaResize::None }
}
```

### `TextAreaBase`

Standalone styled `<textarea>` (no `FormField` wrapper), controlled or uncontrolled.

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `value` | `ReadSignal<Option<String>>` | `None` | `Some` = controlled (pair with `on_value_change`) |
| `default_value` | `String` | `""` | initial value when uncontrolled |
| `on_value_change` | `Callback<String>` | no-op | fires with the new value on every input event |
| `on_commit` | `Callback<String>` | no-op | fires with the committed value on the change event |
| `on_blur` / `on_key_down` | `Callback<FocusEvent>` / `Callback<KeyboardEvent>` | no-op | |
| `disabled` | `ReadSignal<bool>` | `false` | |
| `size` | `FieldSize` | `Default` | |
| `resize` | `TextAreaResize` | `Vertical` | |
| `class` | `String` | `""` | extra classes; full class list when `unstyled` |
| `placeholder` | `Option<String>` | `None` | |
| `id` | `Option<String>` | `None` | form bindings set this to the field name |
| `autofocus` | `bool` | `false` | |
| `unstyled` | `bool` | `false` | skip built-in styling; `class` used verbatim |
| `aria_invalid` | `Option<String>` | `None` | form bindings set `"true"` on validation failure |
| `aria_describedby` | `Option<String>` | `None` | the field's error element id |
| ...attributes | extends `GlobalAttributes` + `textarea` | | `rows`, `cols`, `minlength`, `maxlength`, `name`, ... |

`textarea_insert_at_cursor(element_id, text)` inserts text at the textarea's (or text `Input`'s)
caret and keeps a bound `value` signal in sync (dispatches an `input` event). Pass the element's
`id`, and call `prevent_default()` on the trigger's `mousedown` so the caret survives the click.

`active_element_id() -> Option<String>` returns the id of the currently focused element (`None` on
SSR or when nothing/an id-less element is focused). Use it to route a caret insert to whichever of
several fields holds focus, e.g. subject `Input` vs. body `RichTextEditor`.

---

## RichTextEditor

Contenteditable rich-text field with a formatting toolbar (bold, italic, font size, color, alignment, link).
Form-integrated; follows the same three-layer pattern as `TextArea`.

| Prop | Type | Default |
|------|------|---------|
| `field` | `impl Into<Field>` | required |
| `autofocus` | `bool` | `false` |
| `show_bold` | `bool` | `true` |
| `show_italic` | `bool` | `true` |
| `show_font_size` | `bool` | `true` |
| `show_color` | `bool` | `true` |
| `show_align` | `bool` | `true` |
| `show_link` | `bool` | `true` |

```rust
use components::RichTextEditor;

rsx! {
    RichTextEditor { field: MyForm::bio }
    RichTextEditor { field: MyForm::notes, show_font_size: false, show_color: false }
}
```

Use `RichTextEditorBase` when you need an uncontrolled editor outside a form:

| Prop | Type | Default |
|------|------|---------|
| `class` | `String` | `""` |
| `placeholder` | `Option<String>` | `None` |
| `id` | `Option<String>` | `None` |
| `disabled` | `bool` | `false` |
| `autofocus` | `bool` | `false` |
| `show_bold` | `bool` | `true` |
| `show_italic` | `bool` | `true` |
| `show_font_size` | `bool` | `true` |
| `show_color` | `bool` | `true` |
| `show_align` | `bool` | `true` |
| `show_link` | `bool` | `true` |
| `value` | `Option<Signal<String>>` | `None` |
| `on_change` | `Option<EventHandler<String>>` | `None` |
| `onblur` | `Option<EventHandler<FocusEvent>>` | `None` |
| `aria_invalid` | `Option<String>` | `None` |
| `aria_describedby` | `Option<String>` | `None` |
| `inline` | `bool` | `false` |
| `content_class` | `String` | `""` |
| `content_style` | `String` | `""` |

```rust
use components::RichTextEditorBase;

let mut content = use_signal(|| String::new());

rsx! {
    RichTextEditorBase {
        value: Some(content),
        on_change: move |html: String| content.set(html),
    }
}
```

`rte_insert_text(element_id, text)` inserts plain text at the editor's current caret
(falls back to the end if the editor isn't focused). Pass the editor's `id`, and call
`prevent_default()` on the trigger's `mousedown` so the caret survives the click:

```rust
use components::{RichTextEditorBase, rte_insert_text};

let editor_id = use_hook(|| format!("body-{}", dioxus::core::current_scope_id().0));

rsx! {
    RichTextEditorBase { id: editor_id.clone(), value: Some(content) }
    button {
        onmousedown: |e| e.prevent_default(),
        onclick: {
            let editor_id = editor_id.clone();
            move |_| rte_insert_text(&editor_id, "{{person.first_name}}")
        },
        "First name"
    }
}
```

---

## Checkbox

| Prop | Type | Default |
|------|------|---------|
| `field` | `impl Into<Field>` | required |
| `class` | `String` | `""` |
| `tooltip` | `Option<Element>` | `None` — `circle-help` hint after the inline label |

`CheckboxBase` (standalone): `checked: ReadSignal<Option<bool>>` (`Some` = controlled),
`default_checked: bool`, `on_checked_change: Callback<bool>`, `disabled: ReadSignal<bool>`,
`class`, aria props, plus spread attributes.

```rust
use components::{Checkbox, CheckboxBase};

rsx! {
    CheckboxBase {
        checked: Some(is_selected()),
        on_checked_change: move |on: bool| is_selected.set(on),
    }
}

rsx! {
    Checkbox {
        field: MyForm::agree,
        tooltip: Some(rsx! { "We only contact you about account activity." }),
    }
}
```

---

## RadioGroup

| Prop | Type | Default |
|------|------|---------|
| `field` | `impl Into<Field>` | required |
| `direction` | `RadioGroupDirection` | `Vertical` |
| `class` | `String` | `""` |
| `options` | `&'static [(&'static str, &'static str)]` | `&[]` |
| `tooltip` | `Option<Element>` | `None` — when set, renders a visible group label (the field label) with a `circle-help` hint above the options; when `None`, no visible group label (today's behavior, aria-label only) |
| `children` | `Option<Element>` | `None` |

```rust
use components::{RadioGroup, RadioGroupDirection};

view! {
    <RadioGroup
        field=MyForm::role
        options=&[("admin", "Admin"), ("user", "User")]
    />
    <RadioGroup
        field=MyForm::plan
        direction=RadioGroupDirection::Horizontal
        options=Plan::OPTIONS
    />
}
```

---

## Select

| Prop | Type | Default |
|------|------|---------|
| `field` | `impl Into<Field>` | required |
| `class` | `String` | `""` |
| `searchable` / `multiple` | `bool` | `false` |
| `limit` | `usize` | `0` |
| `copyable` / `clearable` | `bool` | `false` |
| `size` | `FieldSize` | `Default` |
| `options` | `&'static [(&'static str, &'static str)]` | `&[]` |
| `tooltip` | `Option<Element>` | `None` — `circle-help` hint after the floating label |
| `children` | `Option<Element>` | `None` |

`SelectBase` (standalone, with `use_select_contexts(value, on_change: Callback<String>, dynamic,
limit, multiple)`): `disabled` is `ReadSignal<bool>`, `size` is `FieldSize`.

```rust
use components::Select;

view! {
    <Select
        field=MyForm::country
        searchable=true
        options=&[("ca", "Canada"), ("us", "United States")]
    />
}
```

---

## DatePicker

Stored values: `DatePicker` → `YYYY-MM-DD`; `DateRangePicker` → JSON `["YYYY-MM-DD","YYYY-MM-DD"]`; `DateTimePicker` → `YYYY-MM-DD HH:MM:SS` (matches `components::DateTime` `Display`).

| Component | Props |
|-----------|--------|
| `DatePicker` | `field` (`impl Into<Field>`), `class`, `min: Option<Date>`, `max: Option<Date>`, `disabled: ReadSignal<bool>`, `tooltip: Option<Element>` |
| `DateRangePicker` | `field`, `class`, `min: Option<Date>`, `max: Option<Date>`, `disabled: ReadSignal<bool>`, `tooltip` |
| `DateTimePicker` | `field`, `class`, `min: Option<DateTime>`, `max: Option<DateTime>`, `disabled: ReadSignal<bool>`, `tooltip`, `utc: bool` (store RFC3339 UTC, display device-local wall time — for form fields typed `OffsetDateTime`) |
| `DatePickerBase` | standalone (no form binding): `value: ReadSignal<Option<String>>` (`YYYY-MM-DD`; `Some` = controlled), `on_value_change: Callback<String>`, `class`, `disabled: ReadSignal<bool>`, `min`/`max: Option<Signal<Date>>`, `is_open: Option<Signal<bool>>` |

Use `DatePickerBase` when you need a controlled date picker outside a `FormProvider` (drive it with your own `value`/`on_value_change`); the `field`-bound `DatePicker` is preferred inside forms. `DateTimePickerBase` follows the same shape (there is no exported `DateRangePickerBase`). `min` and `max` accept the typed values `components::Date` / `components::DateTime` (`Date` for `DatePicker`/`DateRangePicker`; `DateTime` for `DateTimePicker`). Dates outside the range are rendered disabled in the calendar and rejected in text-input mode. `Date::default()` / `DateTime::default()` return today / today-at-midnight and are convenient for "no past" or "no future" constraints.

```rust
use components::{DatePicker, DateRangePicker, DateTimePicker, Date, DateTime};

rsx! {
    // no constraints
    DatePicker { field: MyForm::start_date }
    DateRangePicker { field: MyForm::vacation }
    DateTimePicker { field: MyForm::appointment_at }

    // date of birth — can't be in the future
    DatePicker { field: PersonForm::date_of_birth, max: Date::default() }

    // task due date — can't be in the past
    DateTimePicker { field: TaskForm::due_date, min: DateTime::default() }
}
```

---

## Calendar

### `MonthView`

Presentational 6×7 month calendar grid. The parent owns all state — this component renders only. Navigation chrome (prev/next month controls) is the parent's responsibility.

#### `CalendarEvent`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Unique identifier for the event; passed to `on_event_click` |
| `date` | `time::Date` | The calendar day this event belongs to |
| `label` | `String` | Display text shown inside the event chip |
| `muted` | `bool` | When `true`, renders the chip dimmed and non-interactive |

#### Props

| Prop | Type | Description |
|------|------|-------------|
| `year` | `ReadSignal<i32>` | Currently displayed year |
| `month` | `ReadSignal<u32>` | Currently displayed month (1–12) |
| `events` | `ReadSignal<Vec<CalendarEvent>>` | All events to display; filtered per-cell by date |
| `on_day_click` | `EventHandler<time::Date>` | Fired when the user clicks a day cell |
| `on_event_click` | `EventHandler<String>` | Fired with the event `id` when an interactive chip is clicked |

Up to 3 events are shown per day; a "+k more" affordance is shown when the day has more.

```rust
use components::{CalendarEvent, MonthView};
use time::Date;

let year = use_signal(|| 2026i32);
let month = use_signal(|| 6u32);
let events = use_signal(|| vec![
    CalendarEvent {
        id: "evt-1".into(),
        date: Date::from_calendar_date(2026, time::Month::June, 15).unwrap(),
        label: "Team sync".into(),
        muted: false,
    },
    CalendarEvent {
        id: "evt-2".into(),
        date: Date::from_calendar_date(2026, time::Month::June, 15).unwrap(),
        label: "Cancelled call".into(),
        muted: true,
    },
]);

rsx! {
    MonthView {
        year: year.into(),
        month: month.into(),
        events: events.into(),
        on_day_click: move |date| { /* navigate to day detail */ },
        on_event_click: move |id| { /* open event detail */ },
    }
}
```

### TimeGrid

Presentational single-day vertical time grid. The parent owns all state — this component renders only. Navigation chrome, timezone handling, and current-time indicators are the parent's responsibility.

#### `TimeGridEvent`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Unique identifier; passed to `on_event_click` |
| `start_minute` | `u16` | Start time in minutes from local midnight (0–1439) |
| `end_minute` | `u16` | End time in minutes from local midnight (0–1439), exclusive |
| `label` | `String` | Display text shown inside the event block |
| `muted` | `bool` | When `true`, renders dimmed and non-interactive (read-only overlay) |

#### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `day` | `ReadSignal<time::Date>` | required | The date shown in the header |
| `events` | `ReadSignal<Vec<TimeGridEvent>>` | required | Events to render; clipped to the visible hour range |
| `start_hour` | `u8` | `0` | First hour shown (0–23) |
| `end_hour` | `u8` | `24` | Last hour shown (1–24, exclusive) |
| `on_slot_click` | `EventHandler<u16>` | required | Fired with the clicked slot's start minute (hour × 60) |
| `on_event_click` | `EventHandler<String>` | required | Fired with the event `id` when an interactive block is clicked |

Overlapping events are laid out side-by-side (greedy lane packing). Each hour row is 48 px tall (0.8 px/min). Muted events render at 50% opacity with `pointer-events-none`.

```rust
use components::{TimeGrid, TimeGridEvent};
use time::Date;

let day = use_signal(|| Date::from_calendar_date(2026, time::Month::June, 27).unwrap());
let events = use_signal(|| vec![
    TimeGridEvent {
        id: "appt-1".into(),
        start_minute: 9 * 60,       // 09:00
        end_minute: 9 * 60 + 30,    // 09:30
        label: "Discovery call".into(),
        muted: false,
    },
    TimeGridEvent {
        id: "appt-2".into(),
        start_minute: 9 * 60 + 15,  // 09:15 — overlaps appt-1
        end_minute: 10 * 60,        // 10:00
        label: "Blocked (personal)".into(),
        muted: true,
    },
]);

rsx! {
    TimeGrid {
        day: day.into(),
        events: events.into(),
        start_hour: 8,
        end_hour: 18,
        on_slot_click: move |minute| { /* open new-event dialog at `minute` */ },
        on_event_click: move |id| { /* open event detail for `id` */ },
    }
}
```

---

## DropdownMenu

Fullstack apps should wrap the router with `AppShellProvider` once (see [Toaster / Toast](#toaster--toast)); it includes `DropdownMenuCoordinatorProvider` so only one menu stays open. For isolated tests or non-app shells, use `DropdownMenuCoordinatorProvider` alone.

### `DropdownMenu`

| Prop | Type | Default |
|------|------|---------|
| `size` | `DropdownMenuSize` | `Default` |
| `placement` | `Placement` | `Auto` |
| `align` | `DropdownMenuAlign` | `End` |
| `open` | `Option<Signal<bool>>` | internal |
| `trigger` | `Element` | required |
| `children` | `Element` | required |

`DropdownMenuSize`: `Default` (w-56) · `Small` (w-44) · `Auto` (content-width, clamped — items never wrap) · `Search` (wide, scrolls).

`DropdownMenuAlign`: `End` aligns the panel to the trigger's right edge (default); `Start` left-aligns it under the trigger — use for left-anchored triggers like inline tokens.

### `DropdownMenuItem`

| Prop | Type | Default |
|------|------|---------|
| `icon` | `Element` | required |
| `label` | `Element` | required |
| `to` | `Option<NavigationTarget>` | `None` |
| `on_click` | `Option<EventHandler<()>>` | `None` |

### `DropdownMenuSub`

| Prop | Type |
|------|------|
| `icon` | `Element` |
| `label` | `Element` |
| `children` | `Element` |

### `DropdownMenuRadioItem`

| Prop | Type | Default |
|------|------|---------|
| `label` | `Element` | required |
| `active` | `ReadSignal<bool>` | required |
| `icon` | `Option<Element>` | `None` |
| `on_select` | `Option<EventHandler<()>>` | `None` |

Selecting an item closes the menu (same as `DropdownMenuItem`).

### `DropdownMenuGroup`

| Prop | Type | Default |
|------|------|---------|
| `label` | `Option<String>` | `None` |
| `children` | `Element` | required |

Sections related items under an optional uppercase label. Every group after the first draws a top divider automatically — stack groups directly, no `DropdownMenuSeparator` needed between them.

### `DropdownMenuSeparator` / `DropdownCloseButton`

`DropdownCloseButton`: `on_click`, `class`, `children`.

```rust
use components::{Button, DropdownMenu, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuSub, Icon, IconName};

rsx! {
    DropdownMenu {
        trigger: rsx! { Button { "Actions" } },
        DropdownMenuItem {
            icon: rsx! { Icon { name: IconName::Edit } },
            label: rsx! { "Edit" },
            on_click: move |_| {},
        }
        DropdownMenuSeparator {}
        DropdownMenuSub {
            icon: rsx! { Icon { name: IconName::MoreHorizontal } },
            label: rsx! { "More" },
            DropdownMenuItem {
                icon: rsx! { Icon { name: IconName::Package } },
                label: rsx! { "Archive" },
                on_click: move |_| {},
            }
        }
    }
}
```

Left-aligned, content-width, grouped menu (e.g. inline token pickers):

```rust
use components::{DropdownCloseButton, DropdownMenu, DropdownMenuAlign, DropdownMenuGroup, DropdownMenuSize};

rsx! {
    DropdownMenu {
        size: DropdownMenuSize::Auto,
        align: DropdownMenuAlign::Start,
        trigger: rsx! { button { class: "underline decoration-dashed", "Then do this" } },
        DropdownMenuGroup { label: "Messaging".to_string(),
            DropdownCloseButton { class: "...", on_click: move |_| {}, "Send email" }
            DropdownCloseButton { class: "...", on_click: move |_| {}, "Send SMS" }
        }
        DropdownMenuGroup { label: "Organize".to_string(),
            DropdownCloseButton { class: "...", on_click: move |_| {}, "Add tag" }
        }
    }
}
```

---

## NavTabs

| Prop | Type | Default |
|------|------|---------|
| `direction` | `NavTabsDirection` | `Horizontal` |
| `items` | `&'static [NavItem<R>]` | required |
| `current_path` | `Option<ReadSignal<String>>` | `None` |

`NavItem::Link(route, label)`, `NavItem::Group(label, &[(route, sub_label), …])`.

`R`: `Routable + Clone + PartialEq + 'static`.

Sliding-indicator helpers (`SlidingIndicatorAxis`, `sliding_indicator_class`, `sliding_indicator_style` on wasm32): see [`src/lib.rs`](src/lib.rs).

```rust
use components::{NavItem, NavTabs, NavTabsDirection};
use dioxus::prelude::*;

let pathname = use_memo(move || {
    let s = router().current::<Route>().to_string();
    s.split_once('?').map(|(p, _)| p.to_string()).unwrap_or(s)
});

rsx! {
    NavTabs::<Route> {
        items: &[NavItem::Link(Route::Dashboard {}, "Dashboard")],
        direction: NavTabsDirection::Horizontal,
        current_path: Some(ReadSignal::from(pathname)),
    }
}
```

---

## RouteTransitionOutlet

| Type param | |
|------------|--|
| `R` | Root `Routable` enum |

```rust
use components::RouteTransitionOutlet;

rsx! {
    RouteTransitionOutlet::<Route> {}
}
```

---

## ListView

### `ListPage<T>`

| Field | Type |
|-------|------|
| `items` | `Vec<T>` |
| `has_more` | `bool` |

### `ListView`

| Prop | Type | Default |
|------|------|---------|
| `class` | `String` | `""` |
| `title` | `Element` | required |
| `action` | `Option<Element>` | `None` |
| `empty` | `Option<Element>` | `None` |
| `skeleton` | `Option<Element>` | `None` |
| `per_page` | `u32` | `25` |
| `cols` | `FlexGridCols` | `C1` |
| `fetch` | `FetchFn<T>` | required |
| `render` | `RenderFn<T>` | required |
| `item_key` | `fn(&T) -> K` | required |

Pass `title` as RSX (e.g. `rsx! { "Items" }` or `rsx! { "{label}" }` for a dynamic `&str`). The list wraps it in the `Title` component (see [Text & Title](#text--title)) for typography. Prefer phrasing-style content inside `title`; block layout inside an `h1` is invalid HTML. When the first page loads with no items, the empty state is shown and the title row (and action slot) are omitted.

```rust
use components::{Button, ButtonVariant, Card, FlexJustify, ListView, Row};

rsx! {
    ListView {
        title: rsx! { "Items" },
        action: rsx! { Button { variant: ButtonVariant::Outline, "Export" } },
        skeleton: rsx! { div { class: "h-16 bg-muted animate-pulse rounded-lg" } },
        fetch: move |page, per_page| async move { load(page, per_page).await },
        item_key: |x| x.id,
        render: |x| rsx! {
            Card {
                Row { justify: FlexJustify::Between,
                    span { "{x.name}" }
                }
            }
        },
    }
}
```

---

## MultiStepForm / Stepper

### `#[derive(Steps)]`

```rust
use components::Steps;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Steps)]
enum MyStep {
    #[step(title = "Personal", description = "…")]
    Personal,
    #[step(title = "Review", description = "…")]
    Review,
}
```

`#[step(...)]` accepts `title` and `description` only. Step field lists come from the `Step`
`fields` prop (or `StepDefinition::fields()`), not the derive.

### `MultiStepForm`

| Prop | Type | Default |
|------|------|---------|
| `form` | `Form<T>` | required |
| `initial` | `Option<S>` | `None` |
| `disabled` | `Option<Signal<bool>>` | `None` |
| `persist_key` | `Option<String>` | `None` |
| `on_step_change` | `Option<EventHandler<(usize, usize)>>` | `None` |
| `children` | `Element` | required |

### `Step`

| Prop | Type | Default |
|------|------|---------|
| `id` | `S: StepDefinition` | required |
| `title` | `Option<&'static str>` | `None` — falls back to `id.title()` |
| `fields` | `Option<Vec<Field>>` | `None` |
| `when` | `Option<ReadSignal<bool>>` | `None` |
| `children` | `Element` | required |

### `StepNav`

| Prop | Type | Default |
|------|------|---------|
| `form` | `Form<T>` | required |
| `on_submit` | `EventHandler<T>` | required |
| `back_label` / `next_label` / `skip_label` / `submit_label` | `Option<String>` | see source |
| `allow_back` | `Option<bool>` | `true` |
| `before_next` | `Option<Callback<(), bool>>` | `None` |
| `class` | `String` | `""` |

### `StepProgress`

| Prop | Type | Default |
|------|------|---------|
| `variant` | `StepProgressVariant` (`Counter` \| `Horizontal` \| `Vertical` \| `Dots`) | `Horizontal` |
| `class` | `String` | `""` |

```rust
// Minimal dots (mobile wizards): active step is a wide pill
StepProgress { variant: StepProgressVariant::Dots }
```

### `SummarySection` / `SummaryField` / `ClearDraftButton` / `StepSuccess`

`SummaryField`: `label`, `name`, `field` (`Option<Field>`), `transform`. `ClearDraftButton`: `label`, `class`, `on_clear`.

### `Stepper` (headless)

```rust
use components::{Step, Stepper};

rsx! {
    Stepper::<MyStep> {
        Step { id: MyStep::First, "…" }
        Step { id: MyStep::Second, "…" }
    }
}
```

### `use_step` / `use_step_ctx`

```rust
use components::{use_step, use_step_ctx};

let ctx = use_step();
ctx.next();
ctx.back();

let ctx = use_step_ctx::<MyStep>();
ctx.go_to(MyStep::Review);
```

```rust
use components::{
    MultiStepForm, Step, StepNav, StepProgress, StepProgressVariant, StepSuccess,
    SummaryField, SummarySection, Text, Title,
};
use components::form::Input;
use components::{FormFields, InputType};
use dioxus::prelude::*;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize, validator::Validate, FormFields)]
struct WizardData {
    name: String,
    email: String,
}

let form = use_form::<WizardData>();

rsx! {
    MultiStepForm {
        form,
        // persist_key is owned: build a per-entity draft key with `format!`/`.to_string()`
        persist_key: Some("wizard_draft".to_string()),
        StepProgress { variant: StepProgressVariant::Horizontal }
        Step { id: MyStep::Personal,
            Input { field: WizardData::name, r#type: InputType::Text }
            Input { field: WizardData::email, r#type: InputType::Email }
        }
        Step { id: MyStep::Review,
            SummarySection { step: MyStep::Personal,
                SummaryField { label: "Name", name: "name" }
                SummaryField { label: "Email", name: "email" }
            }
        }
        StepSuccess {
            Title { "Done" }
            Text { "Submitted." }
        }
        StepNav { form, on_submit: move |_: WizardData| {} }
    }
}
```

---

## FieldName & Field

`FieldName` (scalars) and `FieldArray` (`Vec` / `Option<Vec>`) consts come from `#[derive(FormFields)]`. `FieldPath` is built at runtime via `.dot()` / `.at()` / `From`. `Field` is the erased handle passed to form components.

```rust
use components::{Field, FieldArray, FieldName, FieldPath, FormFields};

#[derive(FormFields)]
struct User {
    name: String,
}

User::name;
```

---

## Derive macros

| Macro | Output |
|-------|--------|
| `FormFields` | `FieldName` / `FieldArray` on struct |
| `FormOptions` | `OPTIONS` for `Select` / `RadioGroup` |
| `Steps` | `ALL` / `COUNT` / `TITLES` / `DESCRIPTIONS` consts on step enums (`StepDefinition` / `StepMeta` impls stay manual) |

```rust
use components::{FormFields, FormOptions};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, FormOptions)]
#[serde(rename_all = "lowercase")]
enum Status { Active, Archived }
```

---

## Spinner

Renders inside a `role="status"` wrapper with a visually-hidden `label` for assistive tech.

| Prop | Type | Default |
|------|------|---------|
| `class` | `String` | `""` |
| `label` | `&'static str` | `"Loading…"` (screen-reader text) |

```rust
use components::Spinner;

view! {
    <Spinner class="size-4 text-muted-foreground" />
}
```

---

## LoadingOverlay

| Prop | Type | Default |
|------|------|---------|
| `when` | `ReadSignal<bool>` | required |
| `message` | `&'static str` | `"Processing…"` |

```rust
use components::LoadingOverlay;

view! {
    <LoadingOverlay when=busy />
    <LoadingOverlay when=saving message="Saving…" />
}
```

---

## ResourceView

Renders the four standard states of a `use_resource` result in one place: loading, error, empty, loaded. Use it instead of hand-rolling the `match resource.value()()` arms **when the empty state replaces the whole content** (views that nest the empty state inside other chrome keep the explicit match).

| Prop | Type | Default |
|------|------|---------|
| `resource` | `Resource<Result<T, DsError>>` | required |
| `skeleton` | `Element` | required — shown while loading |
| `error` | `Callback<DsError, Element>` | required |
| `empty` | `Element` | required — shown when `is_empty` is true |
| `is_empty` | `Callback<T, bool>` | required |
| `view` | `Callback<T, Element>` | required — loaded, non-empty value |

`T: Clone + PartialEq + 'static`.

```rust
use components::{ResourceView, EmptyState};

rsx! {
    ResourceView {
        resource: notes,
        skeleton: rsx! { NotesSkeleton {} },
        error: move |e| rsx! { "Could not load: {e}" },
        empty: rsx! { EmptyState { message: "No notes yet" } },
        is_empty: |notes: Vec<Note>| notes.is_empty(),
        view: move |notes: Vec<Note>| rsx! { /* content */ },
    }
}
```

---

## use_action_feedback

Wires the standard success/error side effects for a `use_action` result — replaces the copy-pasted `use_effect(|| match action.value() { Some(Ok)… Some(Err)… None })` block. On success: runs `on_success`, then shows `success_toast` if set. On error: toasts `"{error_prefix}: {e}"` (or just the error when `error_prefix` is `None`) and resets the action.

| Arg | Type | Notes |
|-----|------|-------|
| `action` | `Action<I, T>` | the `use_action` handle (passed by value; it's `Copy`) |
| `success_toast` | `impl Into<Option<&'static str>>` | pass a `&str`, or `None` to skip the toast |
| `error_prefix` | `impl Into<Option<&'static str>>` | `None` → the toast is just the error |
| `on_success` | `FnMut()` | runs before the success toast |

```rust
use components::use_action_feedback;

use_action_feedback(create_action, "Note posted", "Failed to post note", move || {
    form.reset();
    resource.restart();
});
```

---

## Toaster / Toast

**App integration:** wrap the router with `AppShellProvider` once (provides `ToastStore`, mounts `Toaster`, and includes dropdown coordination). Feature code calls `use_toast()` only—do not add per-route `ToastStore` unless you are testing in isolation.

| Item | Usage |
|------|--------|
| `AppShellProvider` | Wrap router at app root (preferred) |
| `ToastStore` | `use_context_provider(ToastStore::new)` only if not using `AppShellProvider` |
| `store.push(msg, ToastVariant::…)` | Enqueue |
| `Toaster` | Render once with `AppShellProvider`, or manually: `placement`, `dismiss_after_ms` |
| `ToastVariant` | `Success`, `Error`, `Warning`, `Info` |
| `ToastPlacement` | `BottomCenter`, `BottomLeft`, … |

```rust
// services/app — typical
use components::AppShellProvider;
// rsx! { AppShellProvider { Router::<Route> {} } }

// Any descendant
use components::{ToastVariant, use_toast};

let toast = use_toast();
toast.push("Saved".into(), ToastVariant::Success);
```

Manual shell (e.g. tests): `use_context_provider(ToastStore::new)` and `rsx! { Toaster {} }` in the same subtree as consumers.

---

## ToggleCard

A card that can be toggled on or off. The entire header row is clickable. When enabled, renders children below the header.

| Prop | Type | Default |
|------|------|---------|
| `icon` | `Element` | required |
| `title` | `&'static str` | required |
| `description` | `Option<&'static str>` | `None` |
| `enabled` | `ReadSignal<bool>` | required |
| `on_toggle` | `EventHandler<()>` | required |
| `children` | `Option<Element>` | `None` |

```rust
use components::{Icon, IconName, ToggleCard};
use dioxus::prelude::*;

let enabled = use_memo(move || form.get_or(MySource::enabled, false));
rsx! {
    FormProvider { form,
        ToggleCard {
            icon: rsx! { Icon { name: IconName::Banknote, class: "size-5" } },
            title: "401(k)",
            enabled,
            on_toggle: move |_| form.set(MySource::enabled.as_str(), !enabled()),
            // expanded content shown only when enabled
            NumberInput { field: MySource::balance }
        }
    }
}
```

---

## Switch

Standalone controlled toggle. Caller owns state via `checked` + `on_change`.

| Prop | Type | Default |
|------|------|---------|
| `checked` | `bool` | required |
| `on_change` | `Option<EventHandler<bool>>` | `None` |
| `on_toggle` | `Option<EventHandler<bool>>` | `None` — alias of `on_change` |
| `disabled` | `bool` | `false` |
| `loading` | `bool` | `false` |
| `checked_children` | `Option<Element>` | `None` |
| `unchecked_children` | `Option<Element>` | `None` |

```rust
use components::Switch;
use dioxus::prelude::*;

let enabled = use_signal(|| false);
rsx! {
    Switch {
        checked: enabled(),
        on_change: move |v| enabled.set(v),
    }
}
```

---

## Tabs

| Prop | Type | Default |
|------|------|---------|
| `items` | `Vec<TabItem>` | required |
| `tab_type` | `TabType` | `TabType::Card` |
| `active_key` | `Option<Signal<&'static str>>` | `None` |
| `on_change` | `Option<EventHandler<&'static str>>` | `None` |
| `default_active_key` | `Option<&'static str>` | `None` (first item) |

`TabItem` fields: `key: &'static str`, `label: Element`, `children: Element`, `disabled: bool`.

`TabType` variants: `Card` (pill strip), `Line` (underline).

```rust
use components::{Tabs, TabItem, TabType};

// Uncontrolled (internal state)
rsx! {
    Tabs {
        items: vec![
            TabItem { key: "info", label: rsx! { "Info" }, children: rsx! { "Info content" }, disabled: false },
            TabItem { key: "notes", label: rsx! { "Notes" }, children: rsx! { "Notes content" }, disabled: false },
        ],
    }
}

// Line style with default tab
rsx! {
    Tabs {
        tab_type: TabType::Line,
        default_active_key: "notes",
        items: vec![
            TabItem { key: "info", label: rsx! { "Info" }, children: rsx! { "Info content" }, disabled: false },
            TabItem { key: "notes", label: rsx! { "Notes" }, children: rsx! { "Notes content" }, disabled: false },
        ],
    }
}

// Controlled
let active = use_signal(|| "info");
rsx! {
    Tabs {
        active_key: active,
        on_change: move |key| active.set(key),
        items: vec![
            TabItem { key: "info", label: rsx! { "Info" }, children: rsx! { "Info content" }, disabled: false },
        ],
    }
}
```

---

## SegmentedControl

Pure selection control — no content panel. Use `FormOptions` derive for options from an enum.

| Prop | Type | Default |
|------|------|---------|
| `value` | `String` | required |
| `on_change` | `EventHandler<String>` | required |
| `options` | `&'static [(&'static str, &'static str)]` | `&[]` |
| `disabled` | `bool` | `false` |

Renders as a WAI-ARIA radio group with roving focus — Arrow/Home/End move the selection; only the selected option is in the tab order.

```rust
use components::{FormOptions, SegmentedControl};
use dioxus::prelude::*;
use strum_macros::{Display, EnumString};

#[derive(FormOptions, Display, EnumString, Clone, Copy, PartialEq, Default)]
pub enum ViewMode { #[default] Stacked, Comparison }

let mode: Signal<ViewMode> = use_signal(ViewMode::default);
rsx! {
    SegmentedControl {
        value: mode().to_string(),
        on_change: move |v: String| {
            if let Ok(m) = v.parse::<ViewMode>() { mode.set(m); }
        },
        options: ViewMode::OPTIONS,
    }
}
```

---

## ChipToggle

| Prop | Type | Default |
|------|------|---------|
| `selected` | `ReadSignal<bool>` | required |
| `on_click` | `EventHandler<()>` | required |
| `class` | `String` | `""` |
| `aria_label` | `Option<String>` | `None` |
| `children` | `Element` | required |

```rust
use components::ChipToggle;
use dioxus::prelude::*;

let mut on = use_signal(|| false);
rsx! {
    ChipToggle {
        selected: on.into(),
        on_click: move |_| on.set(!on()),
        "Filters"
    }
}
```

---

## EmptyState

| Prop | Type | Default |
|------|------|---------|
| `message` | `&'static str` | required |
| `description` | `Option<&'static str>` | `None` |
| `icon` | `Option<IconName>` | `None` |
| `class` | `String` | `""` |
| `children` | `Option<Element>` | `None` |

```rust
use components::{Button, EmptyState, IconName};

view! {
    <EmptyState message="No items" />
    <EmptyState message="No goals" description="Add one." icon=Some(IconName::Star)>
        <Button>"Add"</Button>
    </EmptyState>
}
```

---

## IconBubble

| Prop | Type | Default |
|------|------|---------|
| `icon` | `IconName` | required |
| `size` | `IconBubbleSize` | `Md` |
| `color` | `IconBubbleColor` | `Primary` |
| `class` | `String` | `""` |

```rust
use components::{IconBubble, IconBubbleColor, IconBubbleSize, IconName};

view! {
    <IconBubble icon=IconName::User />
    <IconBubble icon=IconName::Shield size=IconBubbleSize::Sm color=IconBubbleColor::Muted />
}
```

---

## SectionHeader

| Prop | Type | Default |
|------|------|---------|
| `icon` | `IconName` | required |
| `title` | `&'static str` | required |
| `count` | `Option<usize>` | `None` |
| `class` | `String` | `""` |
| `children` | `Option<Element>` | `None` |

```rust
use components::{Button, ButtonSize, ButtonVariant, IconName, SectionHeader};

rsx! {
    SectionHeader {
        icon: IconName::LayoutList,
        title: "Tasks",
        count: Some(3),
        Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, "Add" }
    }
}
```

---

## DataTable

Columns are declared as `Col` children inside `DataTable`. They register themselves into the
parent on mount via context — no `columns` prop.

| `DataTable` prop | Type | Default |
|------|------|---------|
| `items` | `Vec<T>` | required |
| `item_key` | `ItemKeyProp<T, K>` | required |
| `children` | `Element` (one or more `Col`) | required |
| `sort_key` | `Option<ReadSignal<String>>` | `None` |
| `sort_dir` | `Option<ReadSignal<SortDir>>` | `None` |
| `sort_href` | `SortHrefProp` | default |
| `page` | `Option<ReadSignal<u32>>` | `None` |
| `has_more` | `bool` | `false` |
| `page_href` | `PageHrefProp` | default |
| `selectable` | `bool` | `false` |
| `selection` | `Option<Signal<Vec<K>>>` | `None` |
| `row_href` | `RowHrefProp<T>` | default |
| `row_actions` | `RowActionsProp<T>` | default |
| `row_left` | `RowLeftProp<K>` | default |
| `row_expand` | `RowExpandProp<T>` (`Fn(&T) -> Element`) | default — when set, rows expand on click into a panel rendering this closure, instead of navigating. Mutually exclusive with `row_href` (expand wins). Closure is invoked lazily, only while the row is open. |
| `header_left` | `Option<Element>` | `None` |
| `class` | `String` | `""` |
| `empty` | `Option<Element>` | `None` |
| `loading` | `bool` | `false` — when `true`, renders `skeleton` (or built-in `DataTableSkeleton`) instead of rows |
| `skeleton` | `Option<Element>` | `None` |
| `skeleton_rows` | `usize` | `8` |

`K` (the `item_key` return type) must be `Eq + Hash + Copy + Send + Sync + Display`; the
`Display` impl is used for each row's `key`.

### `Col`

| Prop | Type | Default |
|------|------|---------|
| `id` | `&'static str` | required (column identifier; sort key when `sortable`) |
| `label` | `&'static str` | required |
| `sortable` | `bool` | `false` |
| `class` | `&'static str` | `""` |
| `render` | `ColRenderFn<T>` (`Fn(&T) -> Element`) | required |

```rust
DataTable {
    items,
    item_key: |p: &Person| p.id,
    sort_key: Some(sort_key.into()),
    sort_dir: Some(sort_dir.into()),
    sort_href: sort_href_fn,

    Col {
        id: "name", label: "Name", sortable: true, class: "flex-1 min-w-0",
        render: |p: &Person| rsx! { span { "{p.name}" } },
    }
    Col {
        id: "type", label: "Type", class: "w-36 hidden sm:block",
        render: |p: &Person| rsx! { span { "{p.kind}" } },
    }
}
```

`SortDir` is re-exported alongside `Col`, `DataTable`, and `DataTableSkeleton`.

### Expandable rows

```rust
DataTable {
    items,
    item_key: |r: &Run| r.id,
    row_expand: move |r: &Run| rsx! { RunDetailPanel { id: r.id } },
    Col { id: "name", label: "Name", render: |r: &Run| rsx! { "{r.name}" } }
}
```

---

## Popover / PopoverConfirm

Open state is controlled-or-uncontrolled (`use_controlled`): pass `open` + `on_open_change` to drive
it, or omit them and let the popover own its state (seeded by `default_open`). Escape, outside-pointer
dismissal, and `role="dialog"` / `aria-expanded` / `aria-controls` are wired automatically.

The panel renders through a `Portal` with fixed, collision-aware positioning: it is anchored to the
trigger, centered on the cross axis, and flips to the opposite side when `placement` would overflow
the viewport. This lifts it out of `overflow-hidden`/scroll ancestors (e.g. table cards), so the
panel is never clipped. `class` (on `Popover` only) adds Tailwind to the panel surface (background,
sizing, padding) — not its position, which is computed. Scrolling a container or resizing the window
while open dismisses the popover (the panel does not re-anchor mid-scroll).

| Prop | Component | Type | Default |
|------|-----------|------|---------|
| `trigger` | `Popover` | `Element` | required — the always-visible anchor |
| `open` | both | `ReadSignal<Option<bool>>` | `None` — controlled open state; pair with `on_open_change` |
| `default_open` | both | `bool` | `false` — initial state when uncontrolled |
| `on_open_change` | both | `Callback<bool>` | no-op — fired on every open/close |
| `toggle_on_click` | `Popover` | `bool` | `true` — whether the trigger flips open on click; set `false` when the trigger is itself interactive (a menu) and open is driven externally |
| `placement` | both | `Placement` | `Auto` |
| `class` | `Popover` | `Option<String>` | `None` — extra Tailwind on the **panel** |
| `message` | `PopoverConfirm` | `String` | required — confirmation prompt |
| `confirm_label` | `PopoverConfirm` | `Option<String>` | `"Confirm"` |
| `on_confirm` | `PopoverConfirm` | `EventHandler<()>` | required — fired when confirmed |

```rust
let mut open = use_signal(|| false);
rsx! {
    Popover {
        open: Some(open()),
        on_open_change: move |v| open.set(v),
        trigger: rsx! { Button { "Options" } },
        div { class: "p-2", "Panel content" }
    }
}
```

`PopoverConfirm` is opened externally (a menu item sets the bound signal) and confirmed/cancelled from
its panel, so its trigger does not toggle on click:

```rust
let mut deleting = use_signal(|| false);
rsx! {
    PopoverConfirm {
        open: Some(deleting()),
        on_open_change: move |v| deleting.set(v),
        message: "Delete this item?",
        confirm_label: "Delete",
        on_confirm: move |_: ()| { /* … */ },
        DropdownMenu { /* a menu item does `deleting.set(true)` */ }
    }
}
```

Edge layouts (e.g. sidebars, table rows): no manual alignment needed — the portaled panel anchors to
the trigger and auto-flips when the preferred `placement` would overflow the viewport.

---

## Charts

### `DonutChart`

| Prop | Type | Notes |
|------|------|-------|
| `segments` | `Vec<ChartSegment>` | Ordered arc segments; percentages should sum to ≤ 100 |
| `center_label` | `&'static str` | Small label above the center value |
| `center_pct` | `f64` | Value displayed in center (0–100); animate in caller if desired |
| `aria_label` | `Option<String>` | Screen-reader description on `role="img"` |

```rust
DonutChart {
    segments: vec![
        ChartSegment { label: "Yours".into(), pct: emp, color: SegmentColor::Primary },
        ChartSegment { label: "Match".into(), pct: er,  color: SegmentColor::Success },
        ChartSegment { label: "Growth".into(), pct: gr, color: SegmentColor::Warning },
    ],
    center_label: "Growth",
    center_pct: gr,
}
```

### `StackedBarChart`

| Prop | Type | Notes |
|------|------|-------|
| `segments` | `Vec<ChartSegment>` | Rendered left→right; segments with pct ≤ 0.1 are hidden |
| `aria_label` | `Option<String>` | Screen-reader description on `role="img"` |

```rust
StackedBarChart {
    segments: vec![
        ChartSegment { label: "Your contributions".into(), pct: emp, color: SegmentColor::Primary },
        ChartSegment { label: "Employer match".into(),     pct: er,  color: SegmentColor::Success },
        ChartSegment { label: "Investment growth".into(),  pct: gr,  color: SegmentColor::Warning },
    ],
}
```

### `AreaLineChart`

| Prop | Type | Notes |
|------|------|-------|
| `series` | `Vec<LineSeries>` | Rendered back-to-front; first series is painted first |
| `markers` | `Vec<LineMarker>` | Optional vertical dashed lines (e.g. retirement age, depletion age) |
| `y_format` | `fn(f64) -> String` | Formats y-axis tick labels (e.g. currency, percentage) |
| `x_labels` | `Option<Vec<String>>` | Optional per-point x-axis labels indexed by sorted unique x position; overrides numeric labels |
| `aria_label` | `Option<String>` | Screen-reader description on `role="img"` |

`LineSeries` fields: `label: Cow<'static, str>`, `color: SegmentColor`, `points: Vec<LinePoint>`, `fill: bool` (area fill with 20% opacity).

`LinePoint` fields: `x: f64`, `y: f64`.

`LineMarker` fields: `x: f64` (data x-value), `label: Cow<'static, str>`, `color: SegmentColor`.

Labels accept both `&'static str` literals (`"Balance".into()`) and owned `String`s (`name.into()`) via `Cow::from`.

```rust
AreaLineChart {
    series: vec![
        LineSeries {
            label: "Balance".into(),
            color: SegmentColor::Primary,
            points: balance_points,
            fill: true,
        },
        LineSeries {
            label: "Withdrawals".into(),
            color: SegmentColor::Warning,
            points: withdrawal_points,
            fill: false,
        },
    ],
    markers: vec![
        LineMarker { x: retirement_age as f64, label: "Retire".into(), color: SegmentColor::Success },
        LineMarker { x: depletion_age as f64, label: "Runs out".into(), color: SegmentColor::Destructive },
    ],
    y_format: |v| format!("${:.0}k", v / 1000.0),
}
```

### `ChartSegment` / `SegmentColor`

```rust
ChartSegment { label: Cow<'static, str>, pct: f64, color: SegmentColor }
// SegmentColor variants: Primary | Secondary | Accent | Destructive | Success | Warning
```

---

## Tooltip

Wraps any trigger element and shows a floating label on hover or focus. The panel
is rendered through `Portal`, so it escapes `overflow-hidden`/scroll containers and
modals instead of being clipped by them.

| Prop | Type | Default |
|------|------|---------|
| `title` | `Element` | required |
| `placement` | `Placement` | `Placement::Top` |
| `class` | `String` | `""` — merged onto the root wrapper (e.g. `pointer-events-*` for floating-label hints) |
| `children` | `Element` | required |

```rust
use components::{Tooltip, Placement, Button};

rsx! {
    Tooltip {
        title: rsx! { "Save changes" },
        placement: Placement::Top,
        Button { "Save" }
    }
}
```

`placement` is the *preferred* side; the panel flips to the opposite side when the
preferred side would overflow the viewport (flip only — no shift or arrow). `Auto`
prefers `Bottom`.

---

## Accordion

Collapsible section list. `accordion: true` allows at most one open panel.

| Prop | Type | Default |
|------|------|---------|
| `items` | `Vec<AccordionItem>` | required |
| `accordion` | `bool` | `true` |
| `class` | `String` | `""` |
| `numbered` | `bool` | `false` |
| `default_active_keys` | `Vec<&'static str>` | `[]` |
| `active_keys` | `Option<Signal<Vec<&'static str>>>` | `None` |
| `on_change` | `Option<EventHandler<Vec<&'static str>>>` | `None` |

`numbered: true` renders a 1-based step badge before each label (filled when the section is open) and applies open-aware active styling (left accent bar + tint).

`AccordionItem` fields: `key: &'static str`, `label: Element`, `children: Element`, `disabled: bool`, `summary: Option<Element>` (right-aligned text shown only while the section is collapsed, e.g. `"2 phones · 1 email"`; `None` renders nothing).

```rust
use components::{Accordion, AccordionItem};

rsx! {
    Accordion {
        numbered: true,
        active_keys: open_keys,            // Signal<Vec<&'static str>>
        on_change: move |k| open_keys.set(k),
        items: vec![
            AccordionItem {
                key: "identity",
                label: rsx! { "Identity" },
                summary: Some(rsx! { "Jane Doe · Head of Sales" }),
                children: rsx! { "Profile fields go here." },
                disabled: false,
            },
            AccordionItem {
                key: "usage",
                label: rsx! { "Usage" },
                summary: None,
                children: rsx! { "Usage metrics go here." },
                disabled: false,
            },
        ],
    }
}
```

---

## FileUpload

| Prop | Type | Default |
|------|------|---------|
| `on_change` | `EventHandler<Vec<FileInfo>>` | required |
| `accept` | `Option<&'static str>` | `None` |
| `multiple` | `bool` | `false` |
| `max_size_mb` | `usize` | `0` (no limit) |
| `disabled` | `bool` | `false` |
| `on_error` | `Option<EventHandler<String>>` | `None` |
| `children` | `Option<Element>` | `None` |

`FileInfo` fields: `name: String`, `size: usize`, `mime_type: String`, `data: Vec<u8>`.

Pass `children` for a custom trigger (e.g. a `Button`); it opens the picker on the bubbled click, so the trigger must let the click propagate (no `stop_propagation`).

```rust
use components::{FileInfo, FileUpload};

// Default drop-zone UI
rsx! {
    FileUpload {
        accept: ".csv,text/csv",
        on_change: move |files: Vec<FileInfo>| {
            // files are fully read into memory
        },
    }
}

// With size limit and error handling
rsx! {
    FileUpload {
        multiple: true,
        max_size_mb: 10,
        on_error: move |msg: String| tracing::warn!("{msg}"),
        on_change: move |files: Vec<FileInfo>| { /* ... */ },
    }
}

// Custom trigger (wraps children in a label)
rsx! {
    FileUpload {
        on_change: move |files: Vec<FileInfo>| { /* ... */ },
        Button { "Choose file" }
    }
}
```

---

## Icon

| Prop | Type | Default |
|------|------|---------|
| `name` | `IconName` | required |
| `class` | `String` | `""` |
| `style` | `String` | `""` |
| `stroke_width` | `Option<f32>` | `2.0` (outline icons only) |

Social brand names (solid, monochrome via `currentColor`): `LinkedIn`, `Instagram`, `BrandX`, `Facebook`, `YouTube`, `TikTok`, `WhatsApp`, `Telegram`, `Snapchat`, `Threads`, `GitHub`, `Behance`, `Dribbble`, `Pinterest`. For a website link, use `ExternalLink`.

```rust
use components::{Icon, IconName};

rsx! {
    Icon { name: IconName::LinkedIn, class: "size-5" }
    Icon { name: IconName::BrandX, class: "size-5" }
    Icon { name: IconName::ExternalLink, class: "size-5" }
}
```

---

## Modal

| Prop | Type | Default |
|------|------|---------|
| `title` | `String` | optional |
| `on_close` | `EventHandler<()>` | required |
| `headerless` | `bool` | `false` |
| `unpadded` | `bool` | `false` |
| `size` | `ModalSize` | `Md` |
| `class` | `String` | `""` |
| `attributes` | `Vec<Attribute>` (`extends = GlobalAttributes`) | `[]` |
| `children` | `Element` | required |

`ModalSize`: `Sm`, `Md`, `Lg`, `Xl`, `Xxl`, `Full`. Children render inside a padded, scrollable body wrapper; `unpadded: true` drops it for edge-to-edge content.

```rust
use components::{Modal, ModalSize};

rsx! {
    if show_modal() {
        Modal { title: "Confirm", size: ModalSize::Sm, on_close: move |_| show_modal.set(false),
            "Are you sure?"
        }
    }
}
```

---

## Slider

| Prop | Type | Default |
|------|------|---------|
| `label` | `String` | required |
| `value` | `f32` | required |
| `min` | `f32` | required |
| `max` | `f32` | required |
| `step` | `f32` | `1.0` |
| `unit` | `Option<String>` | `None` |
| `on_change` | `EventHandler<f32>` | required |

```rust
use components::Slider;

rsx! {
    Slider { label: "Opacity", value: opacity(), min: 0.0, max: 100.0, unit: "%", on_change: move |v| opacity.set(v) }
}
```

---

## Progress

| Prop | Type | Default |
|------|------|---------|
| `value` | `f32` | required — clamped to `0.0..=1.0` |
| `class` | `String` | `""` |

```rust
use components::Progress;

rsx! {
    Progress { value: 0.4 }
}
```

---

## NumberStepper

| Prop | Type | Default |
|------|------|---------|
| `value` | `Signal<i64>` | required |
| `step` | `i64` | `1` |
| `min` | `i64` | `i64::MIN` |
| `max` | `i64` | `i64::MAX` |
| `class` | `String` | `""` |

```rust
use components::NumberStepper;

let qty = use_signal(|| 1i64);
rsx! {
    NumberStepper { value: qty, min: 1, max: 10 }
}
```

---

## StatTile

| Prop | Type | Default |
|------|------|---------|
| `label` | `impl Into<String>` | required |
| `value` | `impl Into<String>` | required |
| `sub` | `Option<String>` | `None` |
| `tone` | `StatTone` | `Default` |
| `class` | `String` | `""` |

`StatTone`: `Default`, `Success`, `Destructive`.

```rust
use components::{StatTile, StatTone};

rsx! {
    StatTile { label: "Revenue", value: "$12.4k", sub: "last 30 days", tone: StatTone::Success }
}
```

---

## StatusDot

| Prop | Type | Default |
|------|------|---------|
| `tone` | `DotTone` | `Success` |
| `pulse` | `bool` | `false` |
| `size_px` | `u32` | `8` |
| `class` | `String` | `""` |

`DotTone`: `Success`, `Primary`, `Warning`, `Destructive`, `Muted`, `None`.

```rust
use components::{DotTone, StatusDot};

rsx! {
    StatusDot { tone: DotTone::Warning, pulse: true }
}
```

---

## StepDots

Minimal step-position indicator driven by a `StepMeta` enum (implement `StepMeta` on the enum; `#[derive(Steps)]` supplies the `ALL`/`COUNT`/`TITLES`/`DESCRIPTIONS` consts it needs).

| Prop | Type | Default |
|------|------|---------|
| `current` | `S: StepMeta` | required |
| `failed` | `bool` | `false` |
| `class` | `String` | `""` |

```rust
use components::StepDots;

rsx! {
    StepDots { current: MyStep::Review }
}
```

---

## Portal

Renders `children` into another DOM node (default: `main`).

| Prop | Type | Default |
|------|------|---------|
| `container` | `String` | `"main"` — CSS selector of the target node |
| `class` | `Option<String>` | `None` |
| `id` | `Option<String>` | `None` |
| `children` | `Element` | required |

```rust
use components::Portal;

rsx! {
    Portal { container: "body", div { "floating content" } }
}
```

---

## Further exports

`Placement` / `Align`, `Icon`, `IconName`, `InputBase`, `CheckboxBase`, `FetchFn`, `RenderFn`, `use_escape_listener`, `use_outside_dismiss`, `active_element_id`, `sliding_indicator_class` (+ `SlidingIndicatorAxis`, `HORIZONTAL_SLIDING_INDICATOR_CLASS` / `VERTICAL_SLIDING_INDICATOR_CLASS`), stepper internals, and `#[doc(hidden)]` `serde_json` re-export — see [`src/lib.rs`](src/lib.rs).
