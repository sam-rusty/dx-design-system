use dioxus::prelude::*;
use macros::on_web;

use crate::{DropdownMenuRadioItem, DropdownMenuSub, Icon, IconName};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

on_web! {
    mod web_theme {
        use super::*;

        impl Theme {
            pub(super) fn from_storage(s: Option<String>) -> Self {
                match s.as_deref() {
                    Some("light") => Self::Light,
                    Some("dark") => Self::Dark,
                    _ => Self::System,
                }
            }
        }

        fn document_classes() -> Option<web_sys::DomTokenList> {
            web_sys::window()?
                .document()?
                .document_element()
                .map(|el: web_sys::Element| el.class_list())
        }

        fn prefers_dark() -> bool {
            web_sys::window()
                .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
                .map(|mq: web_sys::MediaQueryList| mq.matches())
                .unwrap_or(false)
        }

        pub fn get_theme() -> Theme {
            let val = utils::LocalStorage::new().get("theme");
            Theme::from_storage(val)
        }

        pub fn apply_theme(theme: Theme) {
            let Some(classes) = document_classes() else {
                return;
            };

            let storage = utils::LocalStorage::new();

            match theme {
                Theme::Light => {
                    storage.set("theme", &"light".to_string());
                    let _ = classes.remove_1("dark");
                }
                Theme::Dark => {
                    storage.set("theme", &"dark".to_string());
                    let _ = classes.add_1("dark");
                }
                Theme::System => {
                    storage.remove("theme");
                    if prefers_dark() {
                        let _ = classes.add_1("dark");
                    } else {
                        let _ = classes.remove_1("dark");
                    }
                }
            }
        }
    }
}

pub fn ThemeMenuView() -> Element {
    let mut theme = use_signal(|| Theme::System);

    #[cfg(feature = "web")]
    use_effect(move || {
        theme.set(web_theme::get_theme());
    });

    let mut select = move |t: Theme| {
        #[cfg(feature = "web")]
        web_theme::apply_theme(t);
        theme.set(t);
    };

    rsx! {
        DropdownMenuSub { icon: rsx! { Icon { name: IconName::Palette } }, label: rsx! { "Appearance" },
            DropdownMenuRadioItem {
                label: rsx! { "Light" },
                active: use_memo(move || theme() == Theme::Light),
                on_select: move |_| select(Theme::Light),
            }
            DropdownMenuRadioItem {
                label: rsx! { "Dark" },
                active: use_memo(move || theme() == Theme::Dark),
                on_select: move |_| select(Theme::Dark),
            }
            DropdownMenuRadioItem {
                label: rsx! { "System" },
                active: use_memo(move || theme() == Theme::System),
                on_select: move |_| select(Theme::System),
            }
        }
    }
}
