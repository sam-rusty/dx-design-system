use dioxus::prelude::*;
use ds_utils::format::merge;

/// Apply `delta * step` to `value`, clamped to `[min, max]`. Pure — unit-tested.
pub(crate) fn stepped(value: i64, delta: i64, step: i64, min: i64, max: i64) -> i64 {
    (value + delta * step).clamp(min, max)
}

/// Numeric increment/decrement control bound to a `Signal<i64>`. Distinct from
/// the multi-step-form `Stepper` — this is a plain number spinner (e.g. a booth
/// count).
#[component]
pub fn NumberStepper(
    value: Signal<i64>,
    #[props(default = 1)] step: i64,
    #[props(default = i64::MIN)] min: i64,
    #[props(default = i64::MAX)] max: i64,
    #[props(default)] class: String,
) -> Element {
    let mut value = value;
    let v = value();
    let btn = "w-7 h-[30px] grid place-items-center text-[15px] text-foreground \
               hover:bg-[rgba(120,120,128,.14)] disabled:opacity-40 disabled:pointer-events-none cursor-pointer";
    let container = merge(&[
        "inline-flex items-center rounded-lg border-[.5px] border-line-strong overflow-hidden bg-glass-hi",
        &class,
    ]);
    rsx! {
        div { class: container,
            button {
                r#type: "button",
                class: btn,
                disabled: v <= min,
                onclick: move |_| value.set(stepped(value(), -1, step, min, max)),
                "−"
            }
            span { class: "w-[42px] text-center text-sm tabular-nums leading-[30px] border-x-[.5px] border-line",
                "{v}"
            }
            button {
                r#type: "button",
                class: btn,
                disabled: v >= max,
                onclick: move |_| value.set(stepped(value(), 1, step, min, max)),
                "+"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::stepped;

    #[test]
    fn clamps_to_min() {
        assert_eq!(stepped(0, -1, 5, 0, 100), 0);
    }

    #[test]
    fn clamps_to_max() {
        assert_eq!(stepped(100, 1, 5, 0, 100), 100);
    }

    #[test]
    fn steps_by_amount() {
        assert_eq!(stepped(70, 1, 1, 0, 200), 71);
        assert_eq!(stepped(70, -1, 1, 0, 200), 69);
    }
}
