use dioxus::prelude::*;
use ds_utils::format::merge;

#[component]
pub fn FormSet(#[props(default)] class: String, children: Element) -> Element {
    let merged = merge(&["flex flex-col gap-6", &class]);
    rsx! {
        fieldset { class: "{merged}", "data-name": "FormSet",
            {children}
        }
    }
}

#[component]
pub fn FormGroup(#[props(default)] class: String, children: Element) -> Element {
    let merged = merge(&["group/field-group flex flex-col gap-3 w-full", &class]);
    rsx! {
        div { class: "{merged}", "data-name": "FormGroup",
            {children}
        }
    }
}

#[component]
pub fn FormContent(#[props(default)] class: String, children: Element) -> Element {
    let merged = merge(&[
        "group/field-content flex flex-1 flex-col gap-1.5 leading-snug",
        &class,
    ]);
    rsx! {
        div { class: "{merged}", "data-name": "FormContent",
            {children}
        }
    }
}

#[component]
pub fn FormTitle(#[props(default)] class: String, children: Element) -> Element {
    let merged = merge(&[
        "flex items-center gap-2 text-sm leading-snug font-medium w-fit",
        &class,
    ]);
    rsx! {
        div { class: "{merged}", "data-name": "FormTitle",
            {children}
        }
    }
}

#[component]
pub fn FormDescription(#[props(default)] class: String, children: Element) -> Element {
    let merged = merge(&["text-muted-foreground text-sm leading-normal", &class]);
    rsx! {
        p { class: "{merged}", "data-name": "FormDescription",
            {children}
        }
    }
}
