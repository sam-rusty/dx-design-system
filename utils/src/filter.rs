//! URL-serializable filter sets and (SSR) SQL `WHERE` building for list endpoints.

use serde::{Deserialize, Serialize};
use strum::EnumProperty;

use crate::types::CollectionId;

// ---------------------------------------------------------------------------
// Filter ops & clauses (shared WASM + SSR)
// ---------------------------------------------------------------------------

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
    strum_macros::EnumProperty,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FilterOp {
    #[strum(props(op_label = "contains"))]
    Like,
    #[strum(props(op_label = "does not contain"))]
    NotLike,
    #[strum(props(op_label = "equals"))]
    Eq,
    #[strum(props(op_label = "does not equal"))]
    Neq,
    #[strum(props(op_label = "starts with"))]
    StartsWith,
    #[strum(props(op_label = "ends with"))]
    EndsWith,
    #[strum(props(op_label = "is empty"))]
    IsEmpty,
    #[strum(props(op_label = "is not empty"))]
    IsNotEmpty,
    #[strum(props(op_label = "before / less than"))]
    Lt,
    #[strum(props(op_label = "on or before"))]
    Lte,
    #[strum(props(op_label = "after / greater than"))]
    Gt,
    #[strum(props(op_label = "on or after"))]
    Gte,
    #[strum(props(op_label = "between"))]
    Between,
    #[strum(props(op_label = "is any of"))]
    In,
    #[strum(props(op_label = "is none of"))]
    NotIn,
}

impl FilterOp {
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    /// Returns the canonical URL key for this operator.
    #[must_use]
    pub fn as_key(&self) -> &str {
        self.as_ref()
    }

    /// Parses a URL key back into a [`FilterOp`], returning `None` for unknown keys.
    #[must_use]
    pub fn from_key(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    #[must_use]
    pub fn op_label(self) -> &'static str {
        EnumProperty::get_str(&self, "op_label").expect("FilterOp: op_label set on all variants")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterType {
    #[default]
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterClause {
    pub col: String,
    pub op: FilterOp,
    #[serde(default)]
    pub val: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterSet {
    #[serde(default)]
    pub clauses: Vec<FilterClause>,
    #[serde(default)]
    pub filter_type: FilterType,
    /// Optional list scope (e.g. people in this collection). Not a column clause; not in `clauses`.
    #[serde(default)]
    pub collection_id: Option<CollectionId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilterEnumOption {
    pub value: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumWidget {
    Select,
    Checkbox,
    Radio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnType {
    Text,
    Email,
    Number,
    Date,
    Bool,
    Enum {
        options: &'static [FilterEnumOption],
        widget: EnumWidget,
    },
}

impl ColumnType {
    #[must_use]
    pub fn valid_ops(self) -> &'static [FilterOp] {
        use FilterOp::*;
        match self {
            Self::Text | Self::Email => &[
                Like, NotLike, Eq, Neq, StartsWith, EndsWith, IsEmpty, IsNotEmpty,
            ],
            Self::Number => &[
                Eq, Neq, Lt, Lte, Gt, Gte, Between, In, NotIn, IsEmpty, IsNotEmpty,
            ],
            Self::Date => &[Eq, Neq, Lt, Lte, Gt, Gte, Between],
            Self::Bool => &[Eq, Neq],
            Self::Enum {
                widget: EnumWidget::Checkbox,
                ..
            } => &[In, NotIn],
            Self::Enum {
                widget: EnumWidget::Radio,
                ..
            } => &[Eq, Neq],
            Self::Enum {
                widget: EnumWidget::Select,
                ..
            } => &[Eq, Neq, In, NotIn],
        }
    }

    #[must_use]
    pub fn op_label(op: FilterOp) -> &'static str {
        op.op_label()
    }
}

pub trait FilterColumns: Sized + Copy + Send + Sync + 'static {
    fn key(self) -> &'static str;
    fn label(self) -> &'static str;
    fn col_type(self) -> ColumnType;
    fn hidden(self) -> bool;
    fn all() -> &'static [Self];
}
