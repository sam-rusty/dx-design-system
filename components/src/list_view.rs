#![allow(unpredictable_function_pointer_comparisons)]
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use dioxus::prelude::*;
use utils::DsError;
use utils::format::merge;

use crate::alert::Alert;
use crate::button::{Button, ButtonVariant};
use crate::layout::{Column, Flex, FlexAlign, FlexGap, FlexGridCols, FlexJustify, Grid, Row};
use crate::logo::Logo;
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

#[component]
pub fn ListEmpty(#[props(default)] children: Option<Element>) -> Element {
    match children {
        Some(c) => rsx! { {c} },
        None => rsx! {
            Column { align: FlexAlign::Center, justify: FlexJustify::Center, class: "py-12",
                Text { variant: TextVariant::Secondary, size: TextSize::Small,
                    "No items found"
                }
            }
        },
    }
}

#[component]
pub fn ListError(#[props(default)] message: Option<String>) -> Element {
    let msg = message.unwrap_or_else(|| "Failed to load items".to_string());
    rsx! {
        Column { align: FlexAlign::Center, justify: FlexJustify::Center, class: "py-12",
            Alert { "{msg}" }
        }
    }
}

#[component]
fn ListPagination(mut page: Signal<u32>, has_more: bool) -> Element {
    let current_page = page();
    let show = current_page > 1 || has_more;

    if !show {
        return rsx! {};
    }

    rsx! {
        Row { align: FlexAlign::Center, justify: FlexJustify::Between, class: "pt-4",
            Button {
                variant: ButtonVariant::Outline,
                disabled: current_page == 1,
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

/// Matches [`ListViewBody`] loading layout: title row, optional action spacer, skeleton grid.
#[component]
fn ListViewSuspenseFallback(
    class: String,
    title: Element,
    cols: FlexGridCols,
    per_page: u32,
    skeleton: Option<Element>,
) -> Element {
    let merged = merge(&["gap-6", &class]);
    match skeleton {
        Some(s) => rsx! {
            Column { class: "{merged}",
                Row { align: FlexAlign::Center, justify: FlexJustify::Between, gap: FlexGap::None,
                    Title { {title} }
                    div { style: "height:40px" }
                }
                Grid { cols: FlexGridCols::C1, class: responsive_cols(cols), gap: FlexGap::Sm,
                    for _ in 0..per_page {
                        {s.clone()}
                    }
                }
            }
        },
        None => rsx! {
            Flex {
                align: FlexAlign::Center,
                justify: FlexJustify::Center,
                class: "min-h-[min(50vh,24rem)] w-full",
                div { class: "animate-pulse",
                    Logo { size: 40 }
                }
            }
        },
    }
}

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
    let class_inner = class.clone();
    let skel = skeleton.clone();
    let title_fallback = title.clone();
    let skel_for_suspense = skel.clone();
    rsx! {
        SuspenseBoundary {
            fallback: move |_| rsx! {
                ListViewSuspenseFallback {
                    class: class.clone(),
                    title: title_fallback.clone(),
                    cols,
                    per_page,
                    skeleton: skel_for_suspense.clone(),
                }
            },
            ListViewBody {
                class: class_inner,
                title,
                action,
                empty,
                per_page,
                cols,
                fetch,
                render,
                item_key,
                skeleton: skel,
            }
        }
    }
}

#[component]
fn ListViewBodyErrorLayout(
    #[props(default)] class: String,
    title: Element,
    message: String,
) -> Element {
    let outer_class = merge(&["gap-6", &class]);
    rsx! {
        Column { class: "{outer_class}",
            Row { align: FlexAlign::Center, justify: FlexJustify::Between, gap: FlexGap::None,
                Title { {title} }
                div { style: "height:40px" }
            }
            ListError { message }
        }
    }
}

#[component]
fn ListViewBodyReady<T, K>(
    #[props(default)] class: String,
    title: Element,
    #[props(default)] action: Option<Element>,
    #[props(default)] empty: Option<Element>,
    #[props(default = 25)] per_page: u32,
    #[props(default = FlexGridCols::C1)] cols: FlexGridCols,
    #[props(into)] render: RenderFn<T>,
    item_key: fn(&T) -> K,
    mut page: Signal<u32>,
    resource: Resource<Result<ListPage<T>, DsError>>,
    #[props(default)] skeleton: Option<Element>,
) -> Element
where
    T: Clone + serde::Serialize + serde::de::DeserializeOwned + PartialEq + 'static,
    K: Display + Clone + 'static,
{
    let outer_class = merge(&["gap-6", &class]);
    rsx! {
        Column { class: "{outer_class}",
            Row { align: FlexAlign::Center, justify: FlexJustify::Between, gap: FlexGap::None,
                Title { {title.clone()} }
                div { style: "height:40px",
                    if let Some(a) = action {
                        {a}
                    }
                }
            }
            {
                match resource.value()() {
                    Some(Ok(list_page)) => {
                        let items = list_page.items.clone();
                        let has_more = list_page.has_more;
                        let is_empty = items.is_empty() && page() == 1;
                        if is_empty {
                            rsx! {
                                ListEmpty { {empty} }
                            }
                        } else {
                            let render_fn = render.0.clone();
                            rsx! {
                                Grid { cols: FlexGridCols::C1, class: responsive_cols(cols), gap: FlexGap::Md,
                                    for item in items {
                                        div {
                                            class: "min-w-0",
                                            key: "{item_key(&item)}",
                                            {(render_fn.clone())(item)}
                                        }
                                    }
                                }
                                ListPagination { page: page, has_more: has_more }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! {
                        ListViewBodyErrorLayout {
                            class: class.clone(),
                            title: title.clone(),
                            message: e.to_string(),
                        }
                    },
                    None => rsx! {
                        ListViewSuspenseFallback {
                            class: class.clone(),
                            title: title.clone(),
                            cols,
                            per_page,
                            skeleton: skeleton.clone(),
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn ListViewBody<T, K>(
    #[props(default)] class: String,
    title: Element,
    #[props(default)] action: Option<Element>,
    #[props(default)] empty: Option<Element>,
    #[props(default = 25)] per_page: u32,
    #[props(default = FlexGridCols::C1)] cols: FlexGridCols,
    #[props(into)] fetch: FetchFn<T>,
    #[props(into)] render: RenderFn<T>,
    item_key: fn(&T) -> K,
    #[props(default)] skeleton: Option<Element>,
) -> Element
where
    T: Clone + serde::Serialize + serde::de::DeserializeOwned + PartialEq + 'static,
    K: Display + Clone + 'static,
{
    let page = use_signal(|| 1u32);
    let resource = use_resource(move || fetch.0(page(), per_page));

    rsx! {
        match resource.value()() {
            None => rsx! {
                ListViewSuspenseFallback {
                    class: class.clone(),
                    title: title.clone(),
                    cols,
                    per_page,
                    skeleton: skeleton.clone(),
                }
            },
            Some(Err(e)) => rsx! {
                ListViewBodyErrorLayout {
                    class: class.clone(),
                    title: title.clone(),
                    message: e.to_string(),
                }
            },
            Some(Ok(_)) => rsx! {
                ListViewBodyReady {
                    class: class.clone(),
                    title: title.clone(),
                    action,
                    empty,
                    per_page,
                    cols,
                    render,
                    item_key,
                    page,
                    resource,
                    skeleton: skeleton.clone(),
                }
            },
        }
    }
}
