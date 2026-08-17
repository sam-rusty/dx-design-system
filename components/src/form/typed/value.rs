//! String ⇄ typed-value conversion at the DOM boundary.
//!
//! Inputs speak strings; the typed store speaks `F`. `FormValue` is the one
//! conversion point — no serde, no JSON quoting. Enums get an impl from the
//! `FormOptions` derive using their serde names.

/// Error from [`FormValue::from_input`]; the message becomes the field error
/// while the raw text sits in the overlay.
#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// A value editable through a string-typed control.
///
/// Deliberately not `PartialEq`-bound so the `FormOptions` derive can emit
/// impls for existing enums; binding creation adds `PartialEq` where memo
/// dedup needs it.
pub trait FormValue: Clone + 'static {
    /// Render for display inside an input.
    fn to_input(&self) -> String;

    /// Parse committed input text. Never called with an empty string — the
    /// binding maps empty text to [`FormValue::empty`] / pristine instead.
    fn from_input(input: &str) -> Result<Self, ParseError>;

    /// The value representing "cleared", when the type can express one.
    /// `None` means the type has no empty representation (e.g. numbers), so
    /// clearing only unmarks the field as written.
    fn empty() -> Option<Self> {
        None
    }

    /// Whether this value counts as empty for required-field checks.
    fn is_empty_value(&self) -> bool {
        false
    }
}

impl FormValue for String {
    fn to_input(&self) -> String {
        self.clone()
    }

    fn from_input(input: &str) -> Result<Self, ParseError> {
        Ok(input.to_string())
    }

    fn empty() -> Option<Self> {
        Some(String::new())
    }

    fn is_empty_value(&self) -> bool {
        self.trim().is_empty()
    }
}

impl FormValue for bool {
    fn to_input(&self) -> String {
        self.to_string()
    }

    fn from_input(input: &str) -> Result<Self, ParseError> {
        input
            .parse()
            .map_err(|_| ParseError::new("Enter true or false"))
    }
}

macro_rules! impl_form_value_number {
    ($($ty:ty),*) => {
        $(
            impl FormValue for $ty {
                fn to_input(&self) -> String {
                    self.to_string()
                }

                fn from_input(input: &str) -> Result<Self, ParseError> {
                    input
                        .trim()
                        .parse()
                        .map_err(|_| ParseError::new("Enter a valid number"))
                }
            }
        )*
    };
}

impl_form_value_number!(
    i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, isize, usize
);

impl<F: FormValue> FormValue for Option<F> {
    fn to_input(&self) -> String {
        self.as_ref().map(FormValue::to_input).unwrap_or_default()
    }

    fn from_input(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            Ok(None)
        } else {
            F::from_input(input).map(Some)
        }
    }

    fn empty() -> Option<Self> {
        Some(None)
    }

    fn is_empty_value(&self) -> bool {
        self.as_ref().is_none_or(FormValue::is_empty_value)
    }
}

/// `Vec` never renders in a text input; required checks treat an empty `Vec`
/// as empty. `from_input` is unreachable through the binding (arrays are
/// edited via rows/multi-select, not text).
impl<F: Clone + 'static> FormValue for Vec<F> {
    fn to_input(&self) -> String {
        String::new()
    }

    fn from_input(_input: &str) -> Result<Self, ParseError> {
        Err(ParseError::new("Arrays are not text-editable"))
    }

    fn empty() -> Option<Self> {
        Some(Vec::new())
    }

    fn is_empty_value(&self) -> bool {
        self.is_empty()
    }
}

/// RFC3339 string form value — pairs with the typed
/// `DateTimePicker { utc: true }` (UTC store, device-local display).
impl FormValue for time::OffsetDateTime {
    fn to_input(&self) -> String {
        self.format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default()
    }

    fn from_input(input: &str) -> Result<Self, ParseError> {
        Self::parse(input.trim(), &time::format_description::well_known::Rfc3339)
            .map_err(|_| ParseError::new("Enter a valid date and time"))
    }
}
