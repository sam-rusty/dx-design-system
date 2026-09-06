#![allow(unpredictable_function_pointer_comparisons)]
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use dioxus::prelude::*;
use ds_utils::DsError;
use ds_utils::format::merge;

use crate::alert::Alert;
use crate::button::{Button, ButtonVariant};
use crate::layout::{Column, Flex, FlexAlign, FlexGap, FlexGridCols, FlexJustify, Grid, Row};
use crate::spinner::Spinner;
use crate::text::{Text, TextSize, TextVariant};
use crate::title::Title;

#[cfg(target_arch = "wasm32")]
type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;
#[cfg(not(target_arch = "wasm32"))]
type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

#[cfg(not(target_arch = "wasm32"))]
type FetchTrait<T> = dyn Fn(u32, u32) -> BoxFuture<Result<ListPage<T>, DsError>> + Send + Sync;
#[cfg(target_arch = "wasm32")]
type FetchTrait<T> = dyn Fn(u32, u32) -> BoxFuture<Result<ListPage<T>, DsError>>;

/// Wrapper for a fetch function that boxes the future and implements PartialEq (always false).
#[allow(clippy::type_complexity)]
pub struct FetchFn<T>(pub Arc<FetchTrait<T>>);

impl<T> Clone for FetchFn<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for FetchFn<T> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: 'static, F, Fut> From<F> for FetchFn<T>
where
    F: Fn(u32, u32) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<ListPage<T>, DsError>> + Send + 'static,
{
    fn from(f: F) -> Self {
        Self(Arc::new(move |page, per_page| Box::pin(f(page, per_page))))
    }
}

#[cfg(target_arch = "wasm32")]
impl<T: 'static, F, Fut> From<F> for FetchFn<T>
where
    F: Fn(u32, u32) -> Fut + 'static,
    Fut: Future<Output = Result<ListPage<T>, DsError>> + 'static,
{
    fn from(f: F) -> Self {
        Self(Arc::new(move |page, per_page| Box::pin(f(page, per_page))))
    }
}

#[cfg(not(target_arch = "wasm32"))]
type RenderTrait<T> = dyn Fn(T) -> Element + Send + Sync;
#[cfg(target_arch = "wasm32")]
type RenderTrait<T> = dyn Fn(T) -> Element;

/// Wrapper for a render function that implements PartialEq (always false).
pub struct RenderFn<T>(pub Arc<RenderTrait<T>>);

impl<T> Clone for RenderFn<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for RenderFn<T> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: 'static, F> From<F> for RenderFn<T>
where
    F: Fn(T) -> Element + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        Self(Arc::new(f))
    }
}

#[cfg(target_arch = "wasm32")]
impl<T: 'static, F> From<F> for RenderFn<T>
where
    F: Fn(T) -> Element + 'static,
{
    fn from(f: F) -> Self {
        Self(Arc::new(f))
    }
}

fn responsive_cols(cols: FlexGridCols) -> &'static str {
    match cols {
        FlexGridCols::None | FlexGridCols::C1 => "",
        FlexGridCols::C2 => "sm:grid-cols-2",
        FlexGridCols::C3 => "sm:grid-cols-2 lg:grid-cols-3",
        FlexGridCols::C4 => "sm:grid-cols-2 lg:grid-cols-4",
        FlexGridCols::C5 => "sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5",
        FlexGridCols::C6 => "sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6",
        FlexGridCols::C7 => "sm:grid-cols-2 lg:grid-cols-4 xl:grid-cols-7",
        FlexGridCols::C12 => "sm:grid-cols-3 lg:grid-cols-6 xl:grid-cols-12",
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ListPage<T> {
    pub items: Vec<T>,
    pub has_more: bool,
}

/// What a [`ListStatus`] block shows.
#[derive(Clone, PartialEq)]
pub(crate) enum ListStatusKind {
    /// "No items found".
    Empty,
    /// An [`Alert`] with the message (default: "Failed to load items").
    Error(Option<String>),
    /// A centred [`Spinner`].
    Loading,
}

/// Shared empty / error / loading block used by `ListView` and `DataTable`
/// (both live behind `data-table`, which implies `feedback`).
#[component]
pub(crate) fn ListStatus(kind: ListStatusKind, #[props(default)] class: String) -> Element {
    match kind {
        ListStatusKind::Empty => rsx! {
            Column { align: FlexAlign::Center, justify: FlexJustify::Center, class: merge(&["py-12", &class]),
                Text { variant: TextVariant::Secondary, size: TextSize::Small,
                    "No items found"
                }
            }
        },
        ListStatusKind::Error(message) => {
            let msg = message.unwrap_or_else(|| "Failed to load items".to_string());
            rsx! {
                Column { align: FlexAlign::Center, justify: FlexJustify::Center, class: merge(&["py-12", &class]),
                    Alert { "{msg}" }
                }
            }
        }
        ListStatusKind::Loading => rsx! {
            Flex {
                align: FlexAlign::Center,
                justify: FlexJustify::Center,
                class: merge(&["min-h-[min(50vh,24rem)] w-full", &class]),
                Spinner {}
            }
        },
    }
}

#[component]
pub fn ListEmpty(#[props(default)] children: Option<Element>) -> Element {
    match children {
        Some(c) => rsx! { {c} },
        None => rsx! {
            ListStatus { kind: ListStatusKind::Empty }
        },
    }
}

#[component]
pub fn ListError(#[props(default)] message: Option<String>) -> Element {
    rsx! {
        ListStatus { kind: ListStatusKind::Error(message) }
    }
}

#[component]
fn ListPagination(
    mut page: Signal<u32>,
    has_more: bool,
    #[props(default)] pending: bool,
) -> Element {
    let current_page = page();
    let show = current_page > 1 || has_more;

    if !show {
        return rsx! {};
    }

    rsx! {
        Row { align: FlexAlign::Center, justify: FlexJustify::Between, class: "pt-4",
            Button {
                variant: ButtonVariant::Outline,
                disabled: current_page == 1 || pending,
                onclick: move |_| {
                    let p = page();
                    page.set(p.saturating_sub(1).max(1));
                },
                "Previous"
            }
            Text { variant: TextVariant::Secondary, size: TextSize::Small,
                "Page {current_page}"
            }
            if has_more {
                Button {
                    variant: ButtonVariant::Outline,
                    disabled: pending,
                    onclick: move |_| {
                        let p = page();
                        page.set(p + 1);
                    },
                    "Next"
                }
            } else {
                div {}
            }
        }
    }
}

/// Title row + body column shared by every non-spinner state of [`ListView`].
#[component]
fn ListViewFrame(
    class: String,
    title: Element,
    #[props(default)] action: Option<Element>,
    children: Element,
) -> Element {
    let outer_class = merge(&["gap-6", &class]);
    rsx! {
        Column { class: "{outer_class}",
            Row { align: FlexAlign::Center, justify: FlexJustify::Between, gap: FlexGap::None,
                Title { {title} }
                div { style: "height:40px",
                    if let Some(a) = action {
                        {a}
                    }
                }
            }
            {children}
        }
    }
}

/// Loading layout: the skeleton grid inside the frame, or a bare centred spinner
/// when no `skeleton` is given.
#[component]
fn ListViewLoading(
    class: String,
    title: Element,
    cols: FlexGridCols,
    per_page: u32,
    skeleton: Option<Element>,
) -> Element {
    match skeleton {
        Some(s) => rsx! {
            ListViewFrame { class, title,
                Grid { cols: FlexGridCols::C1, class: responsive_cols(cols), gap: FlexGap::Sm,
                    for _ in 0..per_page {
                        {s.clone()}
                    }
                }
            }
        },
        None => rsx! {
            ListStatus { kind: ListStatusKind::Loading }
        },
    }
}

/// Paged list backed by an async `fetch`.
///
/// `fetch` and `per_page` are captured when the component mounts: the resource
/// closure is created once and re-runs only when the internal page changes.
/// Read reactive state (filters, search) *inside* the closure so a refetch sees
/// it, and remount the list with a `key` when the query itself changes.
#[component]
pub fn ListView<T, K>(
    #[props(default)] class: String,
    title: Element,
    #[props(default)] action: Option<Element>,
    #[props(default)] empty: Option<Element>,
    #[props(default)] skeleton: Option<Element>,
    #[props(default = 25)] per_page: u32,
    #[props(default = FlexGridCols::C1)] cols: FlexGridCols,
    #[props(into)] fetch: FetchFn<T>,
    #[props(into)] render: RenderFn<T>,
    item_key: fn(&T) -> K,
) -> Element
where
    T: Clone + serde::Serialize + serde::de::DeserializeOwned + PartialEq + 'static,
    K: Display + Clone + 'static,
{
    let page = use_signal(|| 1u32);
    let resource = use_resource(move || fetch.0(page(), per_page));
    // `use_resource` keeps the previous value while the next page loads, so the
    // old page stays visible; `pending` disables paging and marks the grid busy.
    let pending = resource.pending();

    match resource.value()() {
        None => rsx! {
            ListViewLoading { class, title, cols, per_page, skeleton }
        },
        Some(Err(e)) => rsx! {
            ListViewFrame { class, title,
                ListError { message: e.to_string() }
                // A failed page must not strand the user: keep "Previous" reachable.
                if page() > 1 {
                    ListPagination { page, has_more: false, pending }
                }
            }
        },
        Some(Ok(list_page)) => {
            let items = list_page.items;
            let has_more = list_page.has_more;
            if items.is_empty() && page() == 1 {
                rsx! {
                    ListViewFrame { class, title, action,
                        ListEmpty { {empty} }
                    }
                }
            } else {
                let render_fn = render.0.clone();
                rsx! {
                    ListViewFrame { class, title, action,
                        Grid {
                            cols: FlexGridCols::C1,
                            class: responsive_cols(cols),
                            gap: FlexGap::Md,
                            "aria-busy": pending.then_some("true"),
                            for item in items {
                                div {
                                    class: "min-w-0",
                                    key: "{item_key(&item)}",
                                    {(render_fn.clone())(item)}
                                }
                            }
                        }
                        ListPagination { page, has_more, pending }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responsive_cols_table() {
        assert_eq!(responsive_cols(FlexGridCols::None), "");
        assert_eq!(responsive_cols(FlexGridCols::C1), "");
        assert_eq!(responsive_cols(FlexGridCols::C2), "sm:grid-cols-2");
        assert_eq!(
            responsive_cols(FlexGridCols::C12),
            "sm:grid-cols-3 lg:grid-cols-6 xl:grid-cols-12"
        );
    }
}
