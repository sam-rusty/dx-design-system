use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use time::Date;

use super::{CalendarDay, WEEKDAY_LABELS, calendar_grid};
use crate::button::{Button, ButtonSize, ButtonVariant};

const MAX_VISIBLE_EVENTS: usize = 3;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub date: Date,
    pub label: String,
    pub muted: bool,
}

/// A presentational 6×7 month calendar grid.
///
/// The parent owns `year` and `month` signals — this component renders only.
/// Navigation chrome, month-switching controls, etc. are the parent's responsibility.
#[component]
pub fn MonthView(
    year: ReadSignal<i32>,
    month: ReadSignal<u32>,
    events: ReadSignal<Vec<CalendarEvent>>,
    on_day_click: EventHandler<Date>,
    on_event_click: EventHandler<String>,
) -> Element {
    let days = use_memo(move || calendar_grid(year(), month()));

    rsx! {
        div { class: "flex flex-col w-full border border-border rounded-lg overflow-hidden bg-background",
            // Weekday header
            div { class: "grid grid-cols-7 border-b border-border bg-muted/30",
                for label in WEEKDAY_LABELS {
                    div { class: "py-2 text-center text-xs font-medium text-muted-foreground select-none",
                        "{label}"
                    }
                }
            }
            // Calendar grid — 6 rows × 7 columns = 42 cells
            div { class: "grid grid-cols-7 flex-1",
                for (i , day) in days().into_iter().enumerate() {
                    DayCell {
                        key: "{i}",
                        day,
                        events: events()
                            .into_iter()
                            .filter(|e| e.date == day.date)
                            .collect::<Vec<_>>(),
                        on_day_click,
                        on_event_click,
                    }
                }
            }
        }
    }
}

#[component]
fn DayCell(
    day: CalendarDay,
    events: Vec<CalendarEvent>,
    on_day_click: EventHandler<Date>,
    on_event_click: EventHandler<String>,
) -> Element {
    let date = day.date;
    let day_num = date.day();

    let day_class = if day.in_current_month {
        "text-sm font-medium text-foreground"
    } else {
        "text-sm font-medium text-muted-foreground/50"
    };

    let overflow = events.len().saturating_sub(MAX_VISIBLE_EVENTS);
    let visible: Vec<CalendarEvent> = events.into_iter().take(MAX_VISIBLE_EVENTS).collect();

    rsx! {
        div {
            class: "relative min-h-[80px] border-b border-r border-border p-1 flex flex-col gap-0.5 \
                    hover:bg-accent/30 transition-colors cursor-pointer select-none",
            onclick: move |_| on_day_click.call(date),

            // Day number
            span { class: "self-start px-1 {day_class}", "{day_num}" }

            // Event chips (up to MAX_VISIBLE_EVENTS)
            for event in visible {
                EventChip {
                    key: "{event.id}",
                    event,
                    on_event_click,
                }
            }

            // Overflow affordance
            if overflow > 0 {
                span {
                    class: "mt-auto text-[10px] text-muted-foreground px-1",
                    "+{overflow} more"
                }
            }
        }
    }
}

#[component]
fn EventChip(event: CalendarEvent, on_event_click: EventHandler<String>) -> Element {
    let id = event.id.clone();

    let (chip_class, variant) = if event.muted {
        (
            "max-w-full truncate pointer-events-none opacity-50",
            ButtonVariant::Ghost,
        )
    } else {
        ("max-w-full truncate", ButtonVariant::Accent)
    };

    rsx! {
        div {
            class: chip_class,
            onclick: move |e| {
                // Stop propagation so the day-cell click doesn't also fire
                e.stop_propagation();
                if !event.muted {
                    on_event_click.call(id.clone());
                }
            },

            Button {
                variant,
                size: ButtonSize::Badge,
                class: "w-full justify-start text-left truncate",
                disabled: event.muted,
                "{event.label}"
            }
        }
    }
}
