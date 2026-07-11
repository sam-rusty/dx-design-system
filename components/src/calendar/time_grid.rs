use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use time::Date;

use crate::button::{Button, ButtonSize, ButtonVariant};

/// 0.8 px per minute → 48 px per hour (matches `w-12` gutter = 3rem = 48px)
const PX_PER_MINUTE: f32 = 0.8;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeGridEvent {
    pub id: String,
    /// Minutes from local midnight (0–1439)
    pub start_minute: u16,
    /// Minutes from local midnight (0–1439), exclusive upper bound
    pub end_minute: u16,
    pub label: String,
    pub muted: bool,
}

/// A presentational single-day vertical time grid.
///
/// The parent owns `day` and `events` signals — this component renders only.
/// Navigation chrome, timezone handling, and current-time indicators are the parent's
/// responsibility.
#[component]
pub fn TimeGrid(
    day: ReadSignal<Date>,
    events: ReadSignal<Vec<TimeGridEvent>>,
    #[props(default = 0)] start_hour: u8,
    #[props(default = 24)] end_hour: u8,
    on_slot_click: EventHandler<u16>,
    on_event_click: EventHandler<String>,
) -> Element {
    let start_min = (start_hour as u16) * 60;
    let end_min = (end_hour as u16) * 60;
    let visible_minutes = end_min - start_min;
    let total_height = visible_minutes as f32 * PX_PER_MINUTE;

    let columns = use_memo(move || {
        let evts: Vec<TimeGridEvent> = events()
            .into_iter()
            .filter(|e| e.end_minute > start_min && e.start_minute < end_min)
            .collect();
        assign_columns(evts)
    });

    rsx! {
        div { class: "flex flex-col w-full border border-border rounded-lg overflow-hidden bg-background",

            // Day header
            div { class: "px-4 py-2 border-b border-border bg-muted/30 text-sm font-medium text-foreground select-none",
                "{day()}"
            }

            // Scrollable time body
            div { class: "overflow-y-auto",
                div {
                    class: "relative",
                    style: "height: {total_height}px;",

                    // Hour row background stripes — clickable
                    for hour in (start_hour as u16)..(end_hour as u16) {
                        HourRow {
                            key: "{hour}",
                            hour,
                            start_min,
                            on_slot_click,
                        }
                    }

                    // Absolutely-positioned event blocks
                    for (event , col_idx , col_total) in columns() {
                        EventBlock {
                            key: "{event.id}",
                            event,
                            start_min,
                            col_idx,
                            col_total,
                            on_event_click,
                        }
                    }
                }
            }
        }
    }
}

/// Groups events into overlap clusters and assigns each a (col_index, col_total) pair.
///
/// Events whose intervals overlap share the available width equally (greedy lane pack per cluster).
/// Non-overlapping events in separate clusters each get full width (col_total = 1 for a solo event).
fn assign_columns(events: Vec<TimeGridEvent>) -> Vec<(TimeGridEvent, usize, usize)> {
    if events.is_empty() {
        return vec![];
    }

    let mut sorted = events;
    sorted.sort_by(|a, b| {
        a.start_minute
            .cmp(&b.start_minute)
            .then(b.end_minute.cmp(&a.end_minute))
    });

    // --- Step 1: partition sorted events into overlap clusters ---
    // A cluster extends as long as each new event starts before the running max end_minute.
    let mut clusters: Vec<Vec<TimeGridEvent>> = vec![];
    let mut cluster_max_end: u16 = 0;

    for event in sorted {
        if clusters.is_empty() || event.start_minute >= cluster_max_end {
            // Start a new cluster
            cluster_max_end = event.end_minute;
            clusters.push(vec![event]);
        } else {
            // Extend current cluster
            cluster_max_end = cluster_max_end.max(event.end_minute);
            clusters.last_mut().unwrap().push(event);
        }
    }

    // --- Step 2: greedy lane pack within each cluster; col_total = lanes used by that cluster ---
    let mut result: Vec<(TimeGridEvent, usize, usize)> = vec![];

    for cluster in clusters {
        // lanes[i] = end_minute of the last event placed in lane i
        let mut lanes: Vec<u16> = vec![];
        let mut assignments: Vec<(TimeGridEvent, usize)> = vec![];

        for event in cluster {
            let lane = lanes
                .iter()
                .position(|&end| end <= event.start_minute)
                .unwrap_or_else(|| {
                    lanes.push(0);
                    lanes.len() - 1
                });
            lanes[lane] = event.end_minute;
            assignments.push((event, lane));
        }

        let col_total = lanes.len();
        for (event, lane) in assignments {
            result.push((event, lane, col_total));
        }
    }

    result
}

#[component]
fn HourRow(hour: u16, start_min: u16, on_slot_click: EventHandler<u16>) -> Element {
    let top = (hour * 60 - start_min) as f32 * PX_PER_MINUTE;
    let height = 60.0 * PX_PER_MINUTE;
    let label = format!("{hour:02}:00");
    let slot_minute = hour * 60;

    rsx! {
        div {
            class: "absolute left-0 right-0 border-b border-border/50 flex items-start \
                    hover:bg-accent/20 transition-colors cursor-pointer select-none",
            style: "top: {top}px; height: {height}px;",
            onclick: move |_| on_slot_click.call(slot_minute),

            span {
                class: "text-[10px] text-muted-foreground w-12 pl-2 pt-0.5 shrink-0",
                "{label}"
            }
        }
    }
}

#[component]
fn EventBlock(
    event: TimeGridEvent,
    start_min: u16,
    col_idx: usize,
    col_total: usize,
    on_event_click: EventHandler<String>,
) -> Element {
    let top = event.start_minute.saturating_sub(start_min) as f32 * PX_PER_MINUTE;
    let height =
        (event.end_minute.saturating_sub(event.start_minute) as f32 * PX_PER_MINUTE).max(16.0);

    // Event columns occupy the space to the right of the 3rem (48px) hour-label gutter.
    // Use bare integer col_idx / col_total so CSS calc() sees only number × length-percentage,
    // which is valid; a percentage × length-percentage is not (breaks Firefox / Safari).
    let id = event.id.clone();

    let (block_class, variant) = if event.muted {
        (
            "absolute overflow-hidden pointer-events-none opacity-50 rounded",
            ButtonVariant::Ghost,
        )
    } else {
        ("absolute overflow-hidden rounded", ButtonVariant::Accent)
    };

    rsx! {
        div {
            class: block_class,
            style: "top: {top}px; height: {height}px; \
                    left: calc(3rem + {col_idx} * (100% - 3rem) / {col_total}); \
                    width: calc((100% - 3rem) / {col_total} - 2px);",
            onclick: move |e| {
                e.stop_propagation();
                if !event.muted {
                    on_event_click.call(id.clone());
                }
            },

            Button {
                variant,
                size: ButtonSize::Badge,
                class: "w-full h-full min-h-0 justify-start text-left truncate items-start \
                        px-1 py-0.5 rounded",
                disabled: event.muted,
                "{event.label}"
            }
        }
    }
}
