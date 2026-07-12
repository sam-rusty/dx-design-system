use dioxus::prelude::*;

use crate::{
    Alert, AlertVariant, Button, ButtonVariant, Column, Container, Flex, FlexAlign, FlexGap,
    FlexJustify, Icon, IconName, Link, Row, Text, TextSize, TextVariant, Title, TitleSize,
};

/// Compact loading state for nested [`SuspenseBoundary`] (main column only, not full viewport).
/// The `logo` is supplied by the host app (each brand ships its own mark).
#[component]
pub fn SectionLoader(logo: Element) -> Element {
    rsx! {
        Flex {
            align: FlexAlign::Center,
            justify: FlexJustify::Center,
            class: "min-h-[min(50vh,24rem)] w-full bg-background text-foreground",
            div { class: "animate-pulse", {logo} }
        }
    }
}

/// Compact fallback for nested [`ErrorBoundary`] around async data hooks (`use_resource`, etc.).
///
/// Keeps loader/network failures in the local column instead of the full-page [`AppRouteErrorFallback`].
#[component]
pub fn SectionErrorFallback(ctx: ErrorContext) -> Element {
    let detail = ctx.error().map(|e| e.to_string()).unwrap_or_default();
    let message = if detail.is_empty() {
        "Something went wrong.".to_string()
    } else {
        detail
    };
    rsx! {
        div { class: "rounded-lg border border-border bg-card p-4",
            Alert { variant: AlertVariant::Destructive, "{message}" }
            Button {
                variant: ButtonVariant::Secondary,
                onclick: move |_| ctx.clear_errors(),
                class: "mt-3",
                "Try again"
            }
        }
    }
}

#[component]
pub fn AppRouteErrorFallback(ctx: ErrorContext, logo: Element) -> Element {
    let detail = ctx.error().map(|e| e.to_string()).unwrap_or_default();
    rsx! {
        Column {
            align: FlexAlign::Center,
            justify: FlexJustify::Center,
            gap: FlexGap::Lg,
            class: "min-h-screen w-full bg-background text-foreground text-center px-4",
            div { class: "opacity-70", {logo} }
            Title { "Something went wrong" }
            Text { variant: TextVariant::Secondary, size: TextSize::Large, class: "max-w-md",
                "This page hit an unexpected error. You can try again or return to the dashboard."
            }
            if !detail.is_empty() {
                Text { variant: TextVariant::Secondary, size: TextSize::Small, class: "max-w-lg font-mono text-left break-words",
                    "{detail}"
                }
            }
            Row {
                align: FlexAlign::Center,
                justify: FlexJustify::Center,
                gap: FlexGap::Sm,
                class: "flex-wrap",
                Button {
                    variant: ButtonVariant::Default,
                    onclick: move |_| ctx.clear_errors(),
                    "Try again"
                }
                // Hard navigation, not a router `Link`: this fallback is rendered by an
                // `ErrorBoundary` that sits *above* the `Router`, so no router context
                // exists here. A full reload also resets the broken SPA state.
                a {
                    href: "/",
                    class: "inline-flex items-center justify-center rounded-md text-sm font-medium border border-border bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4",
                    "Go to dashboard"
                }
            }
        }
    }
}

#[component]
pub fn NotFound(route: Vec<String>, logo: Element) -> Element {
    rsx! {
        Column {
            align: FlexAlign::Center,
            justify: FlexJustify::Center,
            class: "min-h-screen w-full bg-background text-foreground text-center px-4",
            div { class: "mb-12 opacity-70", {logo} }
            Title { "404" }
            Text { variant: TextVariant::Secondary, size: TextSize::Large, class: "mb-8",
                "The page you're looking for doesn't exist."
            }
            Link {
                to: "/",
                class: "text-sm font-medium text-muted-foreground hover:text-foreground border border-border hover:border-foreground px-8 py-3 transition-all duration-200",
                "Return to Dashboard"
            }
        }
    }
}

#[component]
pub fn PageLoader(logo: Element) -> Element {
    rsx! {
        Flex {
            align: FlexAlign::Center,
            justify: FlexJustify::Center,
            class: "min-h-screen w-full bg-background text-foreground",
            div { class: "animate-pulse", {logo} }
        }
    }
}

#[component]
pub fn WorkInProgress(title: String) -> Element {
    rsx! {
        Container {
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-16 lg:gap-24 items-center",

                // ── Left: copy ─────────────────────────────────────────────
                div { class: "flex flex-col items-start",

                    // Spinning icon + label
                    div { class: "flex items-center gap-2.5 mb-10",
                        div {
                            class: "animate-spin",
                            style: "animation-duration: 3s; animation-timing-function: linear;",
                            Icon { name: IconName::Settings, class: "size-4 text-primary" }
                        }
                        span { class: "text-xs font-semibold tracking-[0.22em] uppercase text-muted-foreground",
                            "Work in Progress"
                        }
                    }

                    // Title
                    Title { size: TitleSize::H1, class: "mb-5",
                        "{title}"
                    }

                    // Description
                    Text {
                        variant: TextVariant::Secondary,
                        size: TextSize::Default,
                        class: "mb-10 max-w-xs leading-relaxed",
                        "We're actively building this page. It'll be ready soon — stay tuned!"
                    }

                    // Design → Build → Launch progress track
                    div { class: "flex items-center mb-12",
                        // Step 1: Design — done
                        div { class: "flex items-center gap-1.5",
                            div { class: "size-2 rounded-full bg-primary" }
                            span { class: "text-xs font-medium text-foreground", "Design" }
                        }
                        // Connector: done
                        div { class: "w-6 h-px bg-primary mx-2.5" }
                        // Step 2: Build — active (pinging)
                        div { class: "flex items-center gap-1.5",
                            span { class: "relative flex size-2",
                                span { class: "animate-ping absolute inline-flex size-full rounded-full bg-primary opacity-60" }
                                span { class: "relative size-2 rounded-full bg-primary" }
                            }
                            span { class: "text-xs font-semibold text-foreground", "Build" }
                        }
                        // Connector: pending
                        div {
                            class: "w-6 h-px mx-2.5",
                            style: "background: repeating-linear-gradient(to right, var(--border) 0px, var(--border) 3px, transparent 3px, transparent 6px);",
                        }
                        // Step 3: Launch — pending
                        div { class: "flex items-center gap-1.5",
                            div { class: "size-2 rounded-full border border-border bg-background" }
                            span { class: "text-xs text-muted-foreground", "Launch" }
                        }
                    }
                }

                // ── Right: floating cards canvas ────────────────────────────
                div {
                    class: "relative min-h-[460px] rounded-2xl border border-border overflow-hidden",
                    style: "background-color: var(--card);",
                    // Dot grid
                    div {
                        class: "absolute inset-0 opacity-60",
                        style: "background-image: radial-gradient(circle, var(--border) 1.5px, transparent 1.5px); background-size: 28px 28px;",
                    }

                    // Primary glow orb
                    div {
                        class: "absolute animate-pulse pointer-events-none",
                        style: "top: 38%; left: 42%; width: 220px; height: 220px; transform: translate(-50%, -50%); border-radius: 50%; filter: blur(60px); background-color: color-mix(in srgb, var(--primary) 18%, transparent);",
                    }

                    // ── Floating card 1: Revenue stat ──────────────────────
                    div {
                        class: "absolute top-8 left-6 animate-wip-float",
                        style: "rotate: -4deg; animation-delay: 0s;",
                        div {
                            class: "bg-card border border-border rounded-2xl p-4 w-44",
                            style: "box-shadow: 0 8px 32px rgba(0,0,0,0.08);",
                            // Icon chip
                            div { class: "size-8 rounded-lg bg-primary/10 flex items-center justify-center mb-3",
                                Icon {
                                    name: IconName::Star,
                                    class: "size-3.5 text-primary",
                                    stroke_width: 2.0,
                                }
                            }
                            div { class: "text-[11px] font-medium text-muted-foreground mb-1",
                                "Total Revenue"
                            }
                            div { class: "text-2xl font-bold text-foreground tracking-tight mb-2",
                                "$24,840"
                            }
                            div { class: "flex items-center gap-1",
                                span {
                                    class: "text-[11px] font-semibold",
                                    style: "color: var(--success);",
                                    "▲ 12.4%"
                                }
                                span { class: "text-[11px] text-muted-foreground",
                                    "vs last month"
                                }
                            }
                        }
                    }

                    // ── Floating card 2: Bar chart ─────────────────────────
                    div {
                        class: "absolute top-5 right-5 animate-wip-float-slow",
                        style: "rotate: 4deg; animation-delay: 1.8s;",
                        div {
                            class: "bg-card border border-border rounded-2xl p-4 w-52",
                            style: "box-shadow: 0 8px 32px rgba(0,0,0,0.08);",
                            div { class: "flex items-center justify-between mb-3",
                                span { class: "text-[11px] font-medium text-muted-foreground",
                                    "Monthly Overview"
                                }
                                div { class: "h-1.5 w-8 rounded-full bg-muted animate-pulse" }
                            }
                            // Chart bars
                            div { class: "flex items-end gap-1.5 h-16",
                                div { class: "flex-1 h-8 rounded-sm bg-primary opacity-20" }
                                div { class: "flex-1 h-11 rounded-sm bg-primary opacity-30" }
                                div { class: "flex-1 h-6 rounded-sm bg-primary opacity-20" }
                                div { class: "flex-1 h-14 rounded-sm bg-primary opacity-50" }
                                div { class: "flex-1 h-9 rounded-sm bg-primary opacity-35" }
                                div { class: "flex-1 h-16 rounded-sm bg-primary opacity-70" }
                                div { class: "flex-1 h-5 rounded-sm bg-primary opacity-20 animate-pulse" }
                            }
                            div { class: "mt-2 h-px bg-border" }
                        }
                    }

                    // ── Floating card 3: Activity list ─────────────────────
                    div {
                        class: "absolute bottom-10 left-7 right-7 animate-wip-float",
                        style: "rotate: 1.5deg; animation-delay: 3.5s;",
                        div {
                            class: "bg-card border border-border rounded-2xl p-4",
                            style: "box-shadow: 0 8px 32px rgba(0,0,0,0.08);",
                            div { class: "text-[11px] font-medium text-muted-foreground mb-3",
                                "Recent Activity"
                            }
                            // Row 1 — active
                            div { class: "flex items-center gap-3 py-2.5 border-b border-border",
                                div { class: "size-7 rounded-full bg-primary/10 flex-shrink-0" }
                                div { class: "flex-1 min-w-0",
                                    div { class: "h-2 w-28 rounded-full bg-foreground/10 mb-1.5" }
                                    div { class: "h-1.5 w-20 rounded-full bg-foreground/5" }
                                }
                                div { class: "h-5 w-14 rounded-full bg-primary/10 flex-shrink-0" }
                            }
                            // Row 2 — in progress
                            div { class: "flex items-center gap-3 py-2.5 border-b border-border opacity-50",
                                div { class: "size-7 rounded-full bg-muted flex-shrink-0" }
                                div { class: "flex-1 min-w-0",
                                    div { class: "h-2 w-20 rounded-full bg-foreground/10 mb-1.5" }
                                    div { class: "h-1.5 w-16 rounded-full bg-foreground/5" }
                                }
                                div { class: "h-5 w-14 rounded-full bg-muted flex-shrink-0" }
                            }
                            // Row 3 — not started yet
                            div { class: "flex items-center gap-3 py-2.5 opacity-20",
                                div { class: "size-7 rounded-full bg-muted flex-shrink-0" }
                                div { class: "flex-1 min-w-0",
                                    div { class: "h-2 w-24 rounded-full bg-foreground/10 mb-1.5" }
                                    div { class: "h-1.5 w-12 rounded-full bg-foreground/5" }
                                }
                                div { class: "h-5 w-14 rounded-full bg-muted flex-shrink-0" }
                            }
                        }
                    }

                    // ── Live "Building" badge ──────────────────────────────
                    div { class: "absolute bottom-3.5 right-4 flex items-center gap-1.5 bg-card/90 backdrop-blur-sm border border-border rounded-full px-3 py-1.5",
                        span { class: "relative flex size-1.5",
                            span { class: "animate-ping absolute inline-flex size-full rounded-full bg-primary opacity-60" }
                            span { class: "relative size-1.5 rounded-full bg-primary" }
                        }
                        span { class: "text-[10px] font-semibold tracking-[0.18em] uppercase text-muted-foreground",
                            "Building"
                        }
                    }
                }
            }
        }
    }
}
