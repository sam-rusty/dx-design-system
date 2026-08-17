//! Typed-form variants of the date pickers: same inner controls (they bind
//! through `use_field_binding`, which speaks both form flavors), wrapped in
//! the typed `FormFieldFrame` so the field context is a `BoundField` from
//! `form.field(...)` instead of a legacy string-keyed `Field`.

use dioxus::prelude::*;

use super::Date as WireDate;
use super::DateTime as WireDateTime;
use super::date_range_picker::DateRangePickerControl;
use super::date_time_picker::DateTimePickerControl;
use super::single::DatePickerControl;
use crate::form::typed::FieldHandle;
use crate::form::typed::view::FormFieldFrame;

/// Typed form-bound date-time picker with stacked label and inline error.
#[component]
pub fn DateTimePicker(
    #[props(into)] field: FieldHandle,
    /// Overrides the lens-derived field label.
    #[props(default)]
    label: Option<String>,
    #[props(default)] class: String,
    #[props(default)] min: Option<WireDateTime>,
    #[props(default)] max: Option<WireDateTime>,
    /// Disabled (OR-ed with the form's disabled state).
    #[props(default)]
    disabled: ReadSignal<bool>,
    #[props(default)] tooltip: Option<Element>,
    /// Store RFC3339 UTC, display device-local wall time — for form fields
    /// typed `OffsetDateTime`.
    #[props(default)]
    utc: bool,
) -> Element {
    let open = use_signal(|| false);
    rsx! {
        FormFieldFrame { field, label, tooltip, class,
            DateTimePickerControl { min, max, disabled, open, utc }
        }
    }
}

/// Typed form-bound date picker with stacked label and inline error.
#[component]
pub fn DatePicker(
    #[props(into)] field: FieldHandle,
    /// Overrides the lens-derived field label.
    #[props(default)]
    label: Option<String>,
    #[props(default)] class: String,
    #[props(default)] min: Option<WireDate>,
    #[props(default)] max: Option<WireDate>,
    /// Disabled (OR-ed with the form's disabled state).
    #[props(default)]
    disabled: ReadSignal<bool>,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    let open = use_signal(|| false);
    rsx! {
        FormFieldFrame { field, label, tooltip, class,
            DatePickerControl { min, max, disabled, open }
        }
    }
}

/// Typed form-bound date-range picker with stacked label and inline error.
#[component]
pub fn DateRangePicker(
    #[props(into)] field: FieldHandle,
    /// Overrides the lens-derived field label.
    #[props(default)]
    label: Option<String>,
    #[props(default)] class: String,
    #[props(default)] min: Option<WireDate>,
    #[props(default)] max: Option<WireDate>,
    /// Disabled (OR-ed with the form's disabled state).
    #[props(default)]
    disabled: ReadSignal<bool>,
    #[props(default)] tooltip: Option<Element>,
) -> Element {
    let open = use_signal(|| false);
    rsx! {
        FormFieldFrame { field, label, tooltip, class,
            DateRangePickerControl { min, max, disabled, open }
        }
    }
}
