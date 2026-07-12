use dioxus::prelude::*;
use ds_utils::format::merge;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FlexGap {
    None,
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
    Xxl,
}

impl FlexGap {
    fn class(self) -> &'static str {
        match self {
            Self::None => "gap-0",
            Self::Xs => "gap-1",
            Self::Sm => "gap-2",
            Self::Md => "gap-4",
            Self::Lg => "gap-6",
            Self::Xl => "gap-8",
            Self::Xxl => "gap-12",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FlexAlign {
    #[default]
    Stretch,
    Start,
    Center,
    End,
    Baseline,
}

impl FlexAlign {
    fn class(self) -> &'static str {
        match self {
            Self::Stretch => "items-stretch",
            Self::Start => "items-start",
            Self::Center => "items-center",
            Self::End => "items-end",
            Self::Baseline => "items-baseline",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FlexJustify {
    #[default]
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
}

impl FlexJustify {
    fn class(self) -> &'static str {
        match self {
            Self::Start => "justify-start",
            Self::Center => "justify-center",
            Self::End => "justify-end",
            Self::Between => "justify-between",
            Self::Around => "justify-around",
            Self::Evenly => "justify-evenly",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

impl FlexWrap {
    fn class(self) -> &'static str {
        match self {
            Self::NoWrap => "flex-nowrap",
            Self::Wrap => "flex-wrap",
            Self::WrapReverse => "flex-wrap-reverse",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl FlexDirection {
    fn class(self) -> &'static str {
        match self {
            Self::Row => "flex-row",
            Self::Column => "flex-col",
            Self::RowReverse => "flex-row-reverse",
            Self::ColumnReverse => "flex-col-reverse",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FlexGridCols {
    #[default]
    None,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C12,
}

impl FlexGridCols {
    fn class(self) -> &'static str {
        match self {
            Self::None => "grid-cols-none",
            Self::C1 => "grid-cols-1",
            Self::C2 => "grid-cols-2",
            Self::C3 => "grid-cols-3",
            Self::C4 => "grid-cols-4",
            Self::C5 => "grid-cols-5",
            Self::C6 => "grid-cols-6",
            Self::C7 => "grid-cols-7",
            Self::C12 => "grid-cols-12",
        }
    }
}

#[component]
pub fn Flex(
    #[props(default)] class: String,
    #[props(default)] direction: FlexDirection,
    #[props(default)] gap: FlexGap,
    #[props(default)] align: FlexAlign,
    #[props(default)] justify: FlexJustify,
    #[props(default)] wrap: FlexWrap,
    children: Element,
) -> Element {
    let merged = merge(&[
        "flex",
        direction.class(),
        gap.class(),
        align.class(),
        justify.class(),
        wrap.class(),
        &class,
    ]);

    rsx! {
        div { class: "{merged}", {children} }
    }
}

#[component]
pub fn Row(
    #[props(default)] class: String,
    #[props(default)] gap: FlexGap,
    #[props(default)] align: FlexAlign,
    #[props(default)] justify: FlexJustify,
    #[props(default)] wrap: FlexWrap,
    children: Element,
) -> Element {
    let merged = merge(&[
        "flex flex-row",
        gap.class(),
        align.class(),
        justify.class(),
        wrap.class(),
        &class,
    ]);

    rsx! {
        div { class: "{merged}", {children} }
    }
}

#[component]
pub fn Column(
    #[props(default)] class: String,
    #[props(default)] gap: FlexGap,
    #[props(default)] align: FlexAlign,
    #[props(default)] justify: FlexJustify,
    #[props(default)] wrap: FlexWrap,
    children: Element,
) -> Element {
    let merged = merge(&[
        "flex flex-col",
        gap.class(),
        align.class(),
        justify.class(),
        wrap.class(),
        &class,
    ]);

    rsx! {
        div { class: "{merged}", {children} }
    }
}

#[component]
pub fn Grid(
    #[props(default)] class: String,
    #[props(default)] cols: FlexGridCols,
    #[props(default)] gap: FlexGap,
    #[props(default)] align: FlexAlign,
    #[props(default)] justify: FlexJustify,
    children: Element,
) -> Element {
    let merged = merge(&[
        "grid",
        cols.class(),
        gap.class(),
        align.class(),
        justify.class(),
        &class,
    ]);

    rsx! {
        div { class: "{merged}", {children} }
    }
}

#[component]
pub fn Container(children: Element) -> Element {
    rsx! {
        main { {children} }
    }
}
