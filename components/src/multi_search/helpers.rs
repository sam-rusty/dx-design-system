//! Filter helpers for the multi-search UI.

use dioxus::prelude::{Signal, WritableExt};
use utils::{ColumnType, EnumWidget, FilterClause, FilterOp, FilterSet};

pub(crate) fn clause_for_key(set: &FilterSet, key: &str) -> Option<FilterClause> {
    set.clauses.iter().find(|c| c.col == key).cloned()
}

pub(crate) fn remove_clause(set: &mut FilterSet, key: &str) {
    set.clauses.retain(|c| c.col != key);
}

pub(crate) fn upsert_clause(set: &mut FilterSet, clause: FilterClause) {
    remove_clause(set, &clause.col);
    set.clauses.push(clause);
}

pub(crate) fn default_op_for_column(ct: ColumnType) -> FilterOp {
    match ct {
        ColumnType::Bool => FilterOp::Eq,
        ColumnType::Enum {
            widget: EnumWidget::Checkbox,
            ..
        } => FilterOp::In,
        ColumnType::Enum {
            widget: EnumWidget::Radio,
            ..
        } => FilterOp::Eq,
        _ => FilterOp::Like,
    }
}

pub(crate) fn collect_values(
    ct: ColumnType,
    op: FilterOp,
    a: String,
    b: String,
) -> Option<Vec<String>> {
    match op {
        FilterOp::IsEmpty | FilterOp::IsNotEmpty => Some(vec![]),
        FilterOp::Between => {
            if a.trim().is_empty() || b.trim().is_empty() {
                None
            } else {
                Some(vec![a, b])
            }
        }
        FilterOp::In | FilterOp::NotIn => {
            if matches!(
                ct,
                ColumnType::Enum {
                    widget: EnumWidget::Checkbox,
                    ..
                }
            ) {
                serde_json::from_str(&a)
                    .ok()
                    .filter(|v: &Vec<String>| !v.is_empty())
            } else if a.trim().is_empty() {
                None
            } else {
                Some(vec![a])
            }
        }
        _ => {
            if a.trim().is_empty() {
                None
            } else {
                Some(vec![a])
            }
        }
    }
}

pub(crate) fn clause_primary_val(ct: ColumnType, cl: &FilterClause) -> String {
    if matches!(
        ct,
        ColumnType::Enum {
            widget: EnumWidget::Checkbox,
            ..
        }
    ) {
        serde_json::to_string(&cl.val).unwrap_or_else(|_| "[]".into())
    } else {
        cl.val.first().cloned().unwrap_or_default()
    }
}

pub(crate) fn sync_draft_row(
    mut draft: Signal<FilterSet>,
    col_key: &'static str,
    ct: ColumnType,
    op: FilterOp,
    v0: String,
    v1: String,
) {
    if let Some(vals) = collect_values(ct, op, v0, v1) {
        upsert_clause(
            &mut draft.write(),
            FilterClause {
                col: col_key.to_string(),
                op,
                val: vals,
            },
        );
    } else {
        remove_clause(&mut draft.write(), col_key);
    }
}

/// Short human-readable summary of a clause, shown inside the active filter chip.
pub(crate) fn clause_summary(op: FilterOp, vals: &[String]) -> String {
    match op {
        FilterOp::IsEmpty => "is empty".to_string(),
        FilterOp::IsNotEmpty => "is not empty".to_string(),
        FilterOp::Between => {
            let a = vals.first().map(String::as_str).unwrap_or("?");
            let b = vals.get(1).map(String::as_str).unwrap_or("?");
            format!("{a} – {b}")
        }
        FilterOp::In | FilterOp::NotIn => {
            let items: Vec<String> = vals
                .first()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_else(|| vals.to_vec());
            if items.is_empty() {
                "—".to_string()
            } else {
                items.join(", ")
            }
        }
        FilterOp::Neq => format!("≠ {}", vals.first().map(String::as_str).unwrap_or("")),
        FilterOp::NotLike => {
            format!("not: {}", vals.first().map(String::as_str).unwrap_or(""))
        }
        FilterOp::Lt => format!("< {}", vals.first().map(String::as_str).unwrap_or("")),
        FilterOp::Lte => format!("≤ {}", vals.first().map(String::as_str).unwrap_or("")),
        FilterOp::Gt => format!("> {}", vals.first().map(String::as_str).unwrap_or("")),
        FilterOp::Gte => format!("≥ {}", vals.first().map(String::as_str).unwrap_or("")),
        FilterOp::StartsWith => {
            format!("{}…", vals.first().map(String::as_str).unwrap_or(""))
        }
        FilterOp::EndsWith => {
            format!("…{}", vals.first().map(String::as_str).unwrap_or(""))
        }
        // Like, Eq — just show the raw value
        _ => vals.first().cloned().unwrap_or_default(),
    }
}
