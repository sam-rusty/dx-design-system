use std::borrow::Cow;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::{LazyLock, Mutex};

use serde::de::DeserializeOwned;

/// Upper bound on the number of distinct strings tracked by the interner's
/// dedup set. Static form schemas produce a small, fixed set of paths, so this
/// cap is only ever approached by dynamic array-form paths (`items.0`,
/// `items.1`, …) generated at runtime.
///
/// Once the cap is reached the dedup set stops growing: existing entries are
/// still returned on a hit, but a genuinely-new string past the cap is leaked
/// once and *not* tracked (so a later identical request leaks again). This
/// bounds the interner's tracking structure to a constant while keeping the
/// `&'static str` return type the rest of this module (and `Field`, which is
/// `Copy`) depends on. The tradeoff — possible repeat leaks for distinct
/// dynamic paths beyond the cap — is acceptable because reaching it requires
/// thousands of distinct array indices in a single process.
const INTERN_CAP: usize = 4096;

static INTERNED: LazyLock<Mutex<HashSet<&'static str>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// Interns a string, returning a `&'static str` that is deduplicated up to
/// [`INTERN_CAP`] distinct entries. Each tracked string is leaked only once;
/// subsequent calls with the same value return the previously interned
/// reference. See [`INTERN_CAP`] for the behavior past the cap.
fn intern(s: &str) -> &'static str {
    let mut set = INTERNED.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(&existing) = set.get(s) {
        return existing;
    }
    let leaked: &'static str = Box::leak(s.to_owned().into_boxed_str());
    if set.len() < INTERN_CAP {
        set.insert(leaked);
    }
    leaked
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldType {
    String,
    Bool,
    Number,
    Array,
    Object,
    Null,
}

impl FieldType {
    pub fn from_value(v: &serde_json::Value) -> Self {
        match v {
            serde_json::Value::String(_) => FieldType::String,
            serde_json::Value::Bool(_) => FieldType::Bool,
            serde_json::Value::Number(_) => FieldType::Number,
            serde_json::Value::Array(_) => FieldType::Array,
            serde_json::Value::Object(_) => FieldType::Object,
            serde_json::Value::Null => FieldType::Null,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Field {
    pub name: &'static str,
    pub label: &'static str,
    pub required: bool,
    pub field_type: FieldType,
}

impl Field {
    pub fn new(
        name: &'static str,
        label: &'static str,
        required: bool,
        field_type: FieldType,
    ) -> Self {
        Self {
            name,
            label,
            required,
            field_type,
        }
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    pub fn resolve(
        field: Option<&Field>,
        label: &'static str,
        name: String,
    ) -> (&'static str, String) {
        let label = if !label.is_empty() {
            label
        } else {
            field.map_or("", |f| f.label)
        };
        let name = if !name.is_empty() {
            name
        } else {
            field.map_or_else(String::new, |f| f.name.to_string())
        };
        (label, name)
    }
}

impl<T, F> From<FieldName<T, F>> for Field {
    fn from(f: FieldName<T, F>) -> Self {
        Self {
            name: f.name,
            label: f.label,
            required: f.required,
            field_type: f.field_type,
        }
    }
}

impl<T, F> From<FieldPath<T, F>> for Field {
    fn from(f: FieldPath<T, F>) -> Self {
        Self {
            name: f.field_name(),
            label: f.field_label(),
            required: f.required,
            field_type: f.field_type,
        }
    }
}

impl<T, F> From<FieldArray<T, F>> for Field {
    fn from(f: FieldArray<T, F>) -> Self {
        Self {
            name: f.name,
            label: f.label,
            required: f.required,
            field_type: FieldType::Array,
        }
    }
}

impl<T, F> From<FieldName<T, F>> for Option<Field> {
    fn from(f: FieldName<T, F>) -> Self {
        Some(Field::from(f))
    }
}

impl<T, F> From<FieldPath<T, F>> for Option<Field> {
    fn from(f: FieldPath<T, F>) -> Self {
        Some(Field::from(f))
    }
}

impl<T, F> From<FieldArray<T, F>> for Option<Field> {
    fn from(f: FieldArray<T, F>) -> Self {
        Some(Field::from(f))
    }
}

pub struct FieldName<T, F> {
    pub name: &'static str,
    pub label: &'static str,
    pub required: bool,
    pub field_type: FieldType,
    _phantom: PhantomData<(T, F)>,
}

impl<T, F> FieldName<T, F> {
    pub const fn new(
        name: &'static str,
        label: &'static str,
        required: bool,
        field_type: FieldType,
    ) -> Self {
        Self {
            name,
            label,
            required,
            field_type,
            _phantom: PhantomData,
        }
    }

    pub fn field_name(&self) -> &'static str {
        self.name
    }

    pub fn field_label(&self) -> &'static str {
        self.label
    }

    pub fn as_str(&self) -> &'static str {
        self.name
    }

    pub fn dot<C, CF>(&self, child: FieldName<C, CF>) -> FieldPath<T, CF> {
        FieldPath::new(
            Cow::Owned(format!("{}.{}", self.name, child.name)),
            Cow::Borrowed(child.label),
            child.required,
            child.field_type,
        )
    }

    pub fn dot_array<C, CF>(&self, child: FieldArray<C, CF>) -> FieldPath<T, Vec<CF>> {
        FieldPath::new(
            Cow::Owned(format!("{}.{}", self.name, child.name)),
            Cow::Borrowed(child.label),
            child.required,
            FieldType::Array,
        )
    }
}

impl<T, F> Clone for FieldName<T, F> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, F> Copy for FieldName<T, F> {}

impl<T, F> std::fmt::Debug for FieldName<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldName").field(&self.name).finish()
    }
}

impl<T, F> PartialEq for FieldName<T, F> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl<T, F> Eq for FieldName<T, F> {}

impl<T, F> std::hash::Hash for FieldName<T, F> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<T, F> From<FieldName<T, F>> for FieldPath<T, F> {
    fn from(f: FieldName<T, F>) -> Self {
        FieldPath::new(
            Cow::Borrowed(f.name),
            Cow::Borrowed(f.label),
            f.required,
            f.field_type,
        )
    }
}

impl<T, F> From<FieldName<T, F>> for String {
    fn from(f: FieldName<T, F>) -> Self {
        f.name.to_string()
    }
}

pub struct FieldArray<T, F> {
    pub name: &'static str,
    pub label: &'static str,
    pub required: bool,
    pub element_field_type: FieldType,
    _phantom: PhantomData<(T, F)>,
}

impl<T, F> FieldArray<T, F> {
    pub const fn new(
        name: &'static str,
        label: &'static str,
        required: bool,
        element_field_type: FieldType,
    ) -> Self {
        Self {
            name,
            label,
            required,
            element_field_type,
            _phantom: PhantomData,
        }
    }

    pub fn field_name(&self) -> &'static str {
        self.name
    }

    pub fn field_label(&self) -> &'static str {
        self.label
    }

    pub fn as_str(&self) -> &'static str {
        self.name
    }

    pub fn at(&self, index: usize) -> FieldPath<T, F> {
        FieldPath::new(
            Cow::Owned(format!("{}.{}", self.name, index)),
            Cow::Borrowed(self.label),
            self.required,
            self.element_field_type,
        )
    }
}

impl<T, F> Clone for FieldArray<T, F> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, F> Copy for FieldArray<T, F> {}

impl<T, F> std::fmt::Debug for FieldArray<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldArray").field(&self.name).finish()
    }
}

impl<T, F> PartialEq for FieldArray<T, F> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl<T, F> Eq for FieldArray<T, F> {}

impl<T, F> std::hash::Hash for FieldArray<T, F> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<T, F> From<FieldArray<T, F>> for String {
    fn from(f: FieldArray<T, F>) -> Self {
        f.name.to_string()
    }
}

pub struct FieldPath<T, F> {
    name: Cow<'static, str>,
    label: Cow<'static, str>,
    required: bool,
    pub field_type: FieldType,
    _phantom: PhantomData<(T, F)>,
}

impl<T, F> FieldPath<T, F> {
    pub fn new(
        name: Cow<'static, str>,
        label: Cow<'static, str>,
        required: bool,
        field_type: FieldType,
    ) -> Self {
        Self {
            name,
            label,
            required,
            field_type,
            _phantom: PhantomData,
        }
    }

    pub fn field_name(&self) -> &'static str {
        match &self.name {
            Cow::Borrowed(s) => s,
            Cow::Owned(s) => intern(s),
        }
    }

    pub fn field_label(&self) -> &'static str {
        match &self.label {
            Cow::Borrowed(s) => s,
            Cow::Owned(s) => intern(s),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }

    pub fn dot<C, CF>(&self, child: FieldName<C, CF>) -> FieldPath<T, CF> {
        FieldPath::new(
            Cow::Owned(format!("{}.{}", self.name, child.name)),
            Cow::Borrowed(child.label),
            child.required,
            child.field_type,
        )
    }

    pub fn dot_array<C, CF>(&self, child: FieldArray<C, CF>) -> FieldPath<T, Vec<CF>> {
        FieldPath::new(
            Cow::Owned(format!("{}.{}", self.name, child.name)),
            Cow::Borrowed(child.label),
            child.required,
            FieldType::Array,
        )
    }

    pub fn at(&self, index: usize) -> FieldPath<T, F> {
        FieldPath::new(
            Cow::Owned(format!("{}.{}", self.name, index)),
            self.label.clone(),
            self.required,
            self.field_type,
        )
    }
}

impl<T, F> Clone for FieldPath<T, F> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            label: self.label.clone(),
            required: self.required,
            field_type: self.field_type,
            _phantom: PhantomData,
        }
    }
}

impl<T, F> std::fmt::Debug for FieldPath<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldPath").field(&self.name).finish()
    }
}

impl<T, F> PartialEq for FieldPath<T, F> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl<T, F> Eq for FieldPath<T, F> {}

impl<T, F> std::hash::Hash for FieldPath<T, F> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<T, F> From<FieldPath<T, F>> for String {
    fn from(f: FieldPath<T, F>) -> Self {
        f.name.into_owned()
    }
}

/// Provides a JSON schema value representing the default structure of a type.
/// Used by the form system to determine field types for coercion without
/// runtime serde round-trips.
pub trait FormSchema {
    const FIELD_TYPE: FieldType;
    fn json_schema() -> serde_json::Value;
}

impl FormSchema for String {
    const FIELD_TYPE: FieldType = FieldType::String;
    fn json_schema() -> serde_json::Value {
        serde_json::Value::String(String::new())
    }
}

impl FormSchema for bool {
    const FIELD_TYPE: FieldType = FieldType::Bool;
    fn json_schema() -> serde_json::Value {
        serde_json::Value::Bool(false)
    }
}

macro_rules! impl_form_schema_number {
    ($($ty:ty),*) => {
        $(
            impl FormSchema for $ty {
                const FIELD_TYPE: FieldType = FieldType::Number;
                fn json_schema() -> serde_json::Value {
                    serde_json::json!(0)
                }
            }
        )*
    };
}

impl_form_schema_number!(
    i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, isize, usize
);

impl<T: FormSchema> FormSchema for Option<T> {
    const FIELD_TYPE: FieldType = T::FIELD_TYPE;
    fn json_schema() -> serde_json::Value {
        T::json_schema()
    }
}

impl<T: FormSchema> FormSchema for Vec<T> {
    const FIELD_TYPE: FieldType = FieldType::Array;
    fn json_schema() -> serde_json::Value {
        serde_json::json!([T::json_schema()])
    }
}

pub trait FieldKey<T> {
    type Value: DeserializeOwned;

    fn key(&self) -> &str;
    fn field_type(&self) -> FieldType;
}

impl<T, F: DeserializeOwned> FieldKey<T> for FieldName<T, F> {
    type Value = F;

    fn key(&self) -> &str {
        self.name
    }

    fn field_type(&self) -> FieldType {
        self.field_type
    }
}

impl<T, F: DeserializeOwned> FieldKey<T> for FieldPath<T, F> {
    type Value = F;

    fn key(&self) -> &str {
        self.as_str()
    }

    fn field_type(&self) -> FieldType {
        self.field_type
    }
}

impl<T, F: DeserializeOwned> FieldKey<T> for FieldArray<T, F> {
    type Value = Vec<F>;

    fn key(&self) -> &str {
        self.name
    }

    fn field_type(&self) -> FieldType {
        FieldType::Array
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use strum_macros::Display;
    use validator::Validate;

    use super::*;
    // The `FormFields` derive emits `components::` paths (the consuming app's
    // crate alias for this library); alias ourselves so the derive resolves.
    use crate as components;
    use crate::{FormFields, FormOptions};

    #[derive(Clone, Default, Serialize, Deserialize, Validate, FormFields)]
    struct SimpleForm {
        name: String,
        age: i32,
        active: bool,
    }

    #[derive(Clone, Default, Serialize, Deserialize, Validate, FormFields)]
    struct Address {
        street: String,
        zip: String,
    }

    #[derive(Clone, Default, Serialize, Deserialize, Validate, FormFields)]
    struct NestedForm {
        title: String,
        address: Address,
    }

    #[derive(Clone, Default, Serialize, Deserialize, Validate, FormFields)]
    struct LineItem {
        product: String,
        qty: i32,
    }

    #[derive(Clone, Default, Serialize, Deserialize, Validate, FormFields)]
    struct OrderForm {
        name: String,
        items: Vec<LineItem>,
    }

    #[derive(Clone, Default, Serialize, Deserialize, Validate, FormFields)]
    struct DeepNested {
        label: String,
        inner: Address,
    }

    #[derive(Clone, Default, Serialize, Deserialize, Validate, FormFields)]
    struct TopLevel {
        deep: DeepNested,
    }

    #[derive(Clone, Default, Serialize, Deserialize, Validate, FormFields)]
    struct Matrix {
        rows: Vec<Vec<i32>>,
    }

    #[test]
    fn plain_field_name_value() {
        assert_eq!(SimpleForm::name.as_str(), "name");
        assert_eq!(SimpleForm::age.as_str(), "age");
        assert_eq!(SimpleForm::active.as_str(), "active");
    }

    #[test]
    fn field_name_methods() {
        assert_eq!(SimpleForm::name.field_name(), "name");
        assert_eq!(SimpleForm::name.field_label(), "Name");
    }

    #[test]
    fn with_label_overrides_label_and_keeps_name() {
        let f = Field::from(SimpleForm::name).with_label("Custom Label");
        assert_eq!(f.label, "Custom Label");
        assert_eq!(f.name, "name");
    }

    #[test]
    fn field_name_into_string() {
        let s: String = SimpleForm::name.into();
        assert_eq!(s, "name");
    }

    #[test]
    fn field_name_into_field_path() {
        let p: FieldPath<SimpleForm, String> = SimpleForm::name.into();
        assert_eq!(p.as_str(), "name");
        assert_eq!(p.field_name(), "name");
        assert_eq!(p.field_label(), "Name");
    }

    #[test]
    fn nested_dot_path() {
        let path = NestedForm::address.dot(Address::street);
        assert_eq!(path.as_str(), "address.street");
        assert_eq!(path.field_name(), "address.street");
        assert_eq!(path.field_label(), "Street");

        let path = NestedForm::address.dot(Address::zip);
        assert_eq!(path.as_str(), "address.zip");
        assert_eq!(path.field_label(), "Zip");
    }

    #[test]
    fn nested_dot_path_into_string() {
        let s: String = NestedForm::address.dot(Address::street).into();
        assert_eq!(s, "address.street");
    }

    #[test]
    fn array_field_type() {
        assert_eq!(OrderForm::items.as_str(), "items");
    }

    #[test]
    fn array_at_index() {
        let path = OrderForm::items.at(0);
        assert_eq!(path.as_str(), "items.0");

        let path = OrderForm::items.at(42);
        assert_eq!(path.as_str(), "items.42");
    }

    #[test]
    fn array_at_dot_chain() {
        let path = OrderForm::items.at(0).dot(LineItem::product);
        assert_eq!(path.as_str(), "items.0.product");
        assert_eq!(path.field_label(), "Product");

        let path = OrderForm::items.at(3).dot(LineItem::qty);
        assert_eq!(path.as_str(), "items.3.qty");
        assert_eq!(path.field_label(), "Qty");
    }

    #[test]
    fn array_chain_into_string() {
        let s: String = OrderForm::items.at(1).dot(LineItem::product).into();
        assert_eq!(s, "items.1.product");
    }

    #[test]
    fn deep_nested_dot_chain() {
        let path = TopLevel::deep.dot(DeepNested::inner).dot(Address::street);
        assert_eq!(path.as_str(), "deep.inner.street");
        assert_eq!(path.field_name(), "deep.inner.street");
        assert_eq!(path.field_label(), "Street");
    }

    #[test]
    fn field_path_at() {
        let path = OrderForm::items.at(0).at(1);
        assert_eq!(path.as_str(), "items.0.1");
    }

    #[test]
    fn mixed_field_types() {
        let name_s: String = OrderForm::name.into();
        assert_eq!(name_s, "name");

        assert_eq!(OrderForm::items.as_str(), "items");
        assert_eq!(OrderForm::name.as_str(), "name");
    }

    #[test]
    fn nested_vec_is_array() {
        assert_eq!(Matrix::rows.as_str(), "rows");
        let path = Matrix::rows.at(2);
        assert_eq!(path.as_str(), "rows.2");
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions)]
    #[serde(rename_all = "lowercase")]
    enum Role {
        Admin,
        User,
        Guest,
    }

    #[test]
    fn options_rename_all_lowercase() {
        assert_eq!(
            Role::OPTIONS,
            &[("admin", "Admin"), ("user", "User"), ("guest", "Guest")]
        );
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions)]
    #[serde(rename_all = "kebab-case")]
    enum Status {
        Active,
        OnHold,
        InProgress,
    }

    #[test]
    fn options_rename_all_kebab() {
        assert_eq!(
            Status::OPTIONS,
            &[
                ("active", "Active"),
                ("on-hold", "On Hold"),
                ("in-progress", "In Progress")
            ]
        );
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions)]
    #[serde(rename_all = "snake_case")]
    enum Priority {
        Low,
        Medium,
        VeryHigh,
    }

    #[test]
    fn options_rename_all_snake() {
        assert_eq!(
            Priority::OPTIONS,
            &[
                ("low", "Low"),
                ("medium", "Medium"),
                ("very_high", "Very High")
            ]
        );
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum LogLevel {
        Info,
        Warning,
        CriticalError,
    }

    #[test]
    fn options_rename_all_screaming_snake() {
        assert_eq!(
            LogLevel::OPTIONS,
            &[
                ("INFO", "Info"),
                ("WARNING", "Warning"),
                ("CRITICAL_ERROR", "Critical Error")
            ]
        );
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions)]
    #[serde(rename_all = "camelCase")]
    enum Theme {
        Light,
        Dark,
        HighContrast,
    }

    #[test]
    fn options_rename_all_camel() {
        assert_eq!(
            Theme::OPTIONS,
            &[
                ("light", "Light"),
                ("dark", "Dark"),
                ("highContrast", "High Contrast")
            ]
        );
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions)]
    #[serde(rename_all = "kebab-case")]
    enum MixedRename {
        Active,
        #[serde(rename = "on_hold")]
        OnHold,
        Archived,
    }

    #[test]
    fn options_per_variant_rename_overrides_rename_all() {
        assert_eq!(
            MixedRename::OPTIONS,
            &[
                ("active", "Active"),
                ("on_hold", "On Hold"),
                ("archived", "Archived")
            ]
        );
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions, Display)]
    enum StrumToStringLabel {
        #[strum(to_string = "Custom A")]
        VariantA,
        #[strum(to_string = "Custom B")]
        VariantB,
    }

    #[test]
    fn options_strum_to_string_overrides_label() {
        assert_eq!(
            StrumToStringLabel::OPTIONS,
            &[("VariantA", "Custom A"), ("VariantB", "Custom B")]
        );
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions)]
    enum NoRename {
        Foo,
        BarBaz,
    }

    #[test]
    fn options_no_rename_keeps_pascal() {
        assert_eq!(NoRename::OPTIONS, &[("Foo", "Foo"), ("BarBaz", "Bar Baz")]);
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions)]
    #[serde(rename_all = "UPPERCASE")]
    enum Shout {
        Hello,
        WorldWide,
    }

    #[test]
    fn options_rename_all_uppercase() {
        assert_eq!(
            Shout::OPTIONS,
            &[("HELLO", "Hello"), ("WORLDWIDE", "World Wide")]
        );
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions)]
    #[serde(rename_all = "PascalCase")]
    enum AlreadyPascal {
        One,
        TwoThree,
    }

    #[test]
    fn options_rename_all_pascal_case() {
        assert_eq!(
            AlreadyPascal::OPTIONS,
            &[("One", "One"), ("TwoThree", "Two Three")]
        );
    }

    #[derive(Clone, Serialize, Deserialize, FormOptions)]
    #[serde(rename_all = "SCREAMING-KEBAB-CASE")]
    enum ScreamKebab {
        Red,
        DarkBlue,
    }

    #[test]
    fn options_rename_all_screaming_kebab() {
        assert_eq!(
            ScreamKebab::OPTIONS,
            &[("RED", "Red"), ("DARK-BLUE", "Dark Blue")]
        );
    }
}
