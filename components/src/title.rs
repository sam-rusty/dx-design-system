use dioxus::prelude::*;
use ds_utils::format::merge;

const BASE: &str = "scroll-m-20 tracking-tight text-foreground antialiased";

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TitleSize {
    #[default]
    Default,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl TitleSize {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "text-2xl font-bold",
            Self::H1 => "text-4xl font-bold lg:text-5xl",
            Self::H2 => "pb-2 text-3xl font-bold first:mt-0",
            Self::H3 => "text-2xl font-bold",
            Self::H4 => "text-xl font-semibold",
            Self::H5 => "text-lg font-semibold",
            Self::H6 => "text-base font-semibold",
        }
    }
}

#[component]
pub fn Title(
    #[props(default)] size: TitleSize,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let computed_class = merge(&[BASE, size.class(), &class]);

    match size {
        TitleSize::Default | TitleSize::H1 => rsx! {
            h1 { class: "{computed_class}", "data-name": "Title", {children} }
        },
        TitleSize::H2 => rsx! {
            h2 { class: "{computed_class}", "data-name": "Title", {children} }
        },
        TitleSize::H3 => rsx! {
            h3 { class: "{computed_class}", "data-name": "Title", {children} }
        },
        TitleSize::H4 => rsx! {
            h4 { class: "{computed_class}", "data-name": "Title", {children} }
        },
        TitleSize::H5 => rsx! {
            h5 { class: "{computed_class}", "data-name": "Title", {children} }
        },
        TitleSize::H6 => rsx! {
            h6 { class: "{computed_class}", "data-name": "Title", {children} }
        },
    }
}
