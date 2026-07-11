use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use dioxus::prelude::*;
use utils::format::merge;

use super::state::{
    AnyStepCtx, InternalStepCtx, InternalStepInfo, StepCtx, StepDefinition, StepFieldRegistry,
    StepState, load_persisted, load_persisted_step, remove_persisted, save_persisted,
    save_persisted_step, use_any_step_ctx,
};
use crate::field_name::Field;
use crate::form::{Form, FormContext, FormData, FormProvider};
use crate::icon::{Icon, IconName};
use crate::layout::{FlexGridCols, Grid};
use crate::separator::Separator;
use crate::{Text, TextSize, TextVariant, Title, TitleSize};

const BTN_BASE: &str = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-full text-sm font-semibold h-10 px-5 transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 cursor-pointer active:scale-[0.97] antialiased";

const BTN_PRIMARY: &str = "bg-primary text-primary-foreground hover:opacity-90";

const BTN_OUTLINE: &str = "border border-border bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground";

const BTN_GHOST: &str = "text-muted-foreground hover:text-foreground hover:bg-accent";

/// Step circle class as a single `&'static str` per `(state, clickable)` —
/// avoids the per-step, per-render `format!` the progress bar used to do.
fn step_circle_class(state: StepState, clickable: bool) -> &'static str {
    match (state, clickable) {
        (StepState::Completed, true) => {
            "flex items-center justify-center size-8 rounded-full text-xs font-semibold transition-colors duration-200 shrink-0 disabled:cursor-default cursor-pointer hover:opacity-80 bg-primary text-primary-foreground"
        }
        (StepState::Completed, false) => {
            "flex items-center justify-center size-8 rounded-full text-xs font-semibold transition-colors duration-200 shrink-0 disabled:cursor-default bg-primary text-primary-foreground"
        }
        (StepState::Current, true) => {
            "flex items-center justify-center size-8 rounded-full text-xs font-semibold transition-colors duration-200 shrink-0 disabled:cursor-default cursor-pointer hover:opacity-80 border-2 border-primary text-foreground"
        }
        (StepState::Current, false) => {
            "flex items-center justify-center size-8 rounded-full text-xs font-semibold transition-colors duration-200 shrink-0 disabled:cursor-default border-2 border-primary text-foreground"
        }
        (StepState::Upcoming, true) => {
            "flex items-center justify-center size-8 rounded-full text-xs font-semibold transition-colors duration-200 shrink-0 disabled:cursor-default cursor-pointer hover:opacity-80 bg-muted text-muted-foreground"
        }
        (StepState::Upcoming, false) => {
            "flex items-center justify-center size-8 rounded-full text-xs font-semibold transition-colors duration-200 shrink-0 disabled:cursor-default bg-muted text-muted-foreground"
        }
    }
}

/// Vertical-variant step label class, same no-alloc treatment as the circle.
fn step_vlabel_class(state: StepState, clickable: bool) -> &'static str {
    match (state, clickable) {
        (StepState::Upcoming, true) => {
            "text-sm font-medium text-left disabled:cursor-default cursor-pointer hover:opacity-80 text-muted-foreground"
        }
        (StepState::Upcoming, false) => {
            "text-sm font-medium text-left disabled:cursor-default text-muted-foreground"
        }
        (_, true) => {
            "text-sm font-medium text-left disabled:cursor-default cursor-pointer hover:opacity-80 text-foreground"
        }
        (_, false) => "text-sm font-medium text-left disabled:cursor-default text-foreground",
    }
}

fn validate_step_fields<T: FormData>(form: &Form<T>, fields: &[Field]) -> bool {
    if fields.is_empty() {
        return true;
    }
    form.validate_fields(fields)
}

/// Provides [`StepFieldRegistry`] for [`Stepper`] (no `FormProvider` / [`FormContext`]).
#[component]
fn StepFieldScope(registry: Signal<Vec<Field>>, children: Element) -> Element {
    use_context_provider(|| StepFieldRegistry(registry));
    rsx! {
        {children}
    }
}

/// Slotted `Element` children under [`MultiStepForm`] still resolve the outer [`FormContext`] from
/// `FormProvider`; a separate `StepFieldRegistry` provider is not visible to them. Re-provide
/// `FormContext` with `step_field_registry` so `FormField` registers into this step's signal.
#[component]
fn StepFormRegistryScope(registry: Signal<Vec<Field>>, children: Element) -> Element {
    let parent = try_consume_context::<FormContext>()
        .expect("StepFormRegistryScope: use inside FormProvider (e.g. MultiStepForm)");
    use_context_provider(|| FormContext {
        values_signal: parent.values_signal,
        errors_signal: parent.errors_signal,
        touched_signal: parent.touched_signal,
        set_value: parent.set_value,
        touch_field: parent.touch_field,
        disabled: parent.disabled,
        submit: parent.submit,
        step_field_registry: Some(registry),
    });
    rsx! {
        {children}
    }
}

#[component]
pub fn Stepper<S: StepDefinition>(
    #[props(default)] initial: Option<S>,
    children: Element,
) -> Element {
    // use_hook: InternalStepCtx + noop clear_form only once per instance (ROOT CopyValues).
    // use_any_step_ctx runs at component level so use_memo is not nested inside this hook.
    let (internal, clear_form_fn) = use_hook(|| {
        let step = initial.unwrap_or_else(S::initial);
        let ctx = InternalStepCtx::<S>::new(step);
        let clear_cv = CopyValue::new_in_scope(Box::new(|| {}) as Box<dyn Fn()>, ScopeId::ROOT);
        (ctx, clear_cv)
    });
    let any = use_any_step_ctx(internal, None, clear_form_fn);
    use_context_provider(|| internal);
    use_context_provider(|| StepCtx { internal });
    use_context_provider(|| any);
    let _current_step_anchor = internal.current.read();
    rsx! { {children} }
}

#[component]
pub fn Step<S: StepDefinition>(
    id: S,
    #[props(default)] title: Option<&'static str>,
    #[props(default)] fields: Option<Vec<Field>>,
    #[props(default)] when: Option<ReadSignal<bool>>,
    children: Element,
) -> Element {
    let ctx = use_context::<InternalStepCtx<S>>();

    let step_title = title.unwrap_or_else(|| id.title());
    let step_description = id.description();

    // Static fields from the trait impl (or the `fields` prop override).
    let static_fields: Vec<Field> = fields.unwrap_or_else(|| id.fields().unwrap_or_default());
    let has_override = !static_fields.is_empty();

    // Field lists are read from StepNav, MultiStepForm effects, and ROOT memos in `use_any_step_ctx`.
    // They must live in ROOT like `InternalStepCtx` signals — not in this `Step` scope — or sibling
    // components read a signal owned by a non-ancestor scope and navigation/validation breaks.
    // Stable per `Step` instance: new ROOT signals each render would desync `register_ordered` (early
    // return keeps the first `InternalStepInfo`) from `StepFieldRegistry` passed to children.
    let (fields_signal, registry_signal) = use_hook(|| {
        if has_override {
            (
                Signal::new_in_scope(static_fields.clone(), ScopeId::ROOT),
                Signal::new_in_scope(Vec::new(), ScopeId::ROOT),
            )
        } else {
            let s = Signal::new_in_scope(Vec::new(), ScopeId::ROOT);
            (s, s)
        }
    });

    let info = InternalStepInfo {
        id,
        title: Arc::from(step_title),
        fields: fields_signal,
    };

    // Unconditional hooks: register reactively from `when`, and always unregister on unmount.
    // `when: None` still registers synchronously so the step exists in `InternalStepCtx` during
    // the first render (same as the previous `else` branch).
    if when.is_none() {
        ctx.register_ordered(info.clone());
    }
    use_effect(use_reactive((&when,), move |(when_opt,)| {
        let visible = when_opt.as_ref().map(|s| s()).unwrap_or(true);
        if visible {
            ctx.register_ordered(info.clone());
        } else {
            ctx.unregister(id);
        }
    }));
    use_drop(move || {
        ctx.unregister(id);
    });

    let any = use_context::<AnyStepCtx>();
    let render_title_desc = S::RENDER_TITLE_DESCRIPTION_IN_STEP;
    let has_description = !step_description.is_empty();

    let parent_form = try_consume_context::<FormContext>();
    let step_shell = rsx! {
        div {
            "data-name": "Step",
            "data-active": if *ctx.current.read() == id { "true" },
            hidden: if !(when.map(|w| w()).unwrap_or(true)
                && *ctx.current.read() == id
                && !*any.completed.read())
            {
                true
            },
            if render_title_desc {
                div { class: "mb-3",
                    Title { class: "mb-0 mt-4", size: TitleSize::H4, "{step_title}" }
                    if has_description {
                        Text { variant: TextVariant::Secondary, size: TextSize::Small,
                            "{step_description}"
                        }
                    }
                }
            }
            {children}
        }
    };

    rsx! {
        match parent_form {
            Some(_) => rsx! {
                StepFormRegistryScope {
                    registry: registry_signal,
                    {step_shell}
                }
            },
            None => rsx! {
                StepFieldScope {
                    registry: registry_signal,
                    {step_shell}
                }
            },
        }
    }
}

#[component]
pub fn SummaryField(
    #[props(default)] field: Option<Field>,
    #[props(default)] label: &'static str,
    #[props(default)] name: String,
    #[props(default)] transform: Option<EventHandler<String>>,
) -> Element {
    let (label, name) = Field::resolve(field.as_ref(), label, name);
    let form_ctx = use_context::<FormContext>();

    let val = form_ctx
        .values_signal
        .read()
        .get(name.as_str())
        .cloned()
        .unwrap_or_default();
    let display = if val.is_empty() {
        "\u{2014}".to_string()
    } else {
        val
    };

    rsx! {
        div { "data-name": "SummaryField", class: "flex flex-col gap-0.5 py-1",
            span { class: "text-xs text-muted-foreground", "{label}" }
            span { class: "text-sm text-foreground", "{display}" }
        }
    }
}

#[component]
pub fn SummarySection<S: StepDefinition>(
    step: S,
    #[props(default)] title: Option<String>,
    #[props(default)] editable: Option<bool>,
    children: Element,
) -> Element {
    let nav = use_context::<StepCtx<S>>();
    let section_title = title.unwrap_or_else(|| step.title().to_string());
    let show_edit = editable.unwrap_or(true);

    rsx! {
        div { "data-name": "SummarySection", class: "border border-border rounded-lg p-4",
            div { class: "flex items-center justify-between mb-3",
                h3 { class: "text-sm font-semibold text-foreground", "{section_title}" }
                if show_edit {
                    button {
                        r#type: "button",
                        class: "text-xs font-medium text-muted-foreground hover:text-foreground cursor-pointer transition-colors",
                        onclick: move |_| nav.go_to(step),
                        "Edit"
                    }
                }
            }
            Separator { class: "mb-3" }
            Grid { cols: FlexGridCols::C1, class: "sm:grid-cols-2 gap-x-6 gap-y-2",
                {children}
            }
        }
    }
}

#[component]
pub fn ClearDraftButton(
    #[props(default)] label: Option<String>,
    #[props(default)] class: String,
    #[props(default)] on_clear: Option<EventHandler<()>>,
) -> Element {
    let ctx = use_context::<AnyStepCtx>();
    let text = label.unwrap_or_else(|| "Clear Draft".to_string());

    let btn_class = if class.is_empty() {
        format!("{BTN_BASE} {BTN_GHOST}")
    } else {
        class
    };

    rsx! {
        Text {
            variant: TextVariant::Secondary,
            size: TextSize::Small,
            class: "{btn_class}",
            onclick: move |_| {
                ctx.clear_draft();
                if let Some(cb) = on_clear {
                    cb.call(());
                }
            },
            "{text}"
        }
    }
}

#[component]
pub fn StepSuccess(children: Element) -> Element {
    let ctx = use_context::<AnyStepCtx>();

    rsx! {
        div {
            "data-name": "StepSuccess",
            hidden: if !*ctx.completed.read() { true },
            {children}
        }
    }
}

/// Multi-step form with optional localStorage persistence and step-change callbacks.
///
/// Hooks run in a fixed order regardless of optional props; keep `persist_key` and
/// `on_step_change` stable for a given mount if you rely on predictable restore/persist behavior.
#[component]
pub fn MultiStepForm<T, S>(
    form: Form<T>,
    #[props(default)] initial: Option<S>,
    #[props(default)] on_step_change: Option<EventHandler<(usize, usize)>>,
    #[props(default)] disabled: Option<Signal<bool>>,
    #[props(default)] persist_key: Option<String>,
    children: Element,
) -> Element
where
    T: FormData + Send + Sync + Clone + 'static,
    S: StepDefinition,
{
    // Latest optional props for effects: hooks must run unconditionally (stable order), but effects
    // must read current `persist_key` / `on_step_change` when signals (e.g. step index) update.
    let persist_key_cell = use_hook(|| Rc::new(RefCell::new(None::<String>)));
    *persist_key_cell.borrow_mut() = persist_key.clone();
    let on_step_change_cell = use_hook(|| Rc::new(RefCell::new(on_step_change)));
    *on_step_change_cell.borrow_mut() = on_step_change;

    // use_hook: InternalStepCtx + form reset callback only once per instance.
    // use_any_step_ctx runs at component level so use_memo is not nested inside this hook.
    let (internal, clear_form_fn) = use_hook(|| {
        let ctx = InternalStepCtx::<S>::new(initial.unwrap_or_else(S::initial));
        let form_reset = form;
        let clear_fn: Box<dyn Fn()> = Box::new(move || {
            form_reset.reset();
        });
        let clear_cv = CopyValue::new_in_scope(clear_fn, ScopeId::ROOT);
        (ctx, clear_cv)
    });
    let any_ctx = use_any_step_ctx(internal, persist_key.clone(), clear_form_fn);
    use_context_provider(|| internal);
    use_context_provider(|| StepCtx { internal });
    use_context_provider(|| any_ctx);

    let mut form_clone = form;
    use_effect(move || {
        let cur = *internal.current.read();
        let fields_vec: Vec<Field> = internal
            .steps
            .read()
            .iter()
            .find(|si| si.id == cur)
            .map(|si| si.fields.read().clone())
            .unwrap_or_default();
        form_clone.required_fields.set(
            fields_vec
                .iter()
                .map(|f| (f.name.to_string(), f.required))
                .collect(),
        );
    });

    let mut restored_ready = use_signal(|| persist_key.is_none());

    let persist_key_for_restore = Rc::clone(&persist_key_cell);
    let mut form_restore = form;
    use_effect(move || {
        let pk = persist_key_for_restore.borrow().clone();
        let Some(key) = pk else {
            return;
        };
        if restored_ready() {
            return;
        }

        let restored_values = load_persisted(&key);
        let restored_step = load_persisted_step(&key);

        if let Some(ref values) = restored_values {
            form_restore.values_signal.set(values.clone());
        }

        match restored_step {
            Some(idx) => {
                let total = any_ctx.total();
                if total == 0 {
                    return;
                }
                let target = idx.min(total - 1);
                internal.mark_visited_through(target);
                restored_ready.set(true);
            }
            None => restored_ready.set(true),
        }
    });

    let persist_key_for_values = Rc::clone(&persist_key_cell);
    let form_persist = form;
    use_effect(move || {
        let pk = persist_key_for_values.borrow().clone();
        let Some(key) = pk else {
            return;
        };
        if !restored_ready() {
            return;
        }
        let values = form_persist.values_signal.read().clone();
        save_persisted(&key, &values);
    });

    let persist_key_for_step = Rc::clone(&persist_key_cell);
    use_effect(move || {
        let pk = persist_key_for_step.borrow().clone();
        let Some(key) = pk else {
            return;
        };
        if !restored_ready() {
            return;
        }
        let idx = any_ctx.current_index();
        save_persisted_step(&key, idx);
    });

    // StepNav owns submission; when it flips `submitted`, drop the persisted draft so a
    // completed wizard leaves no stale per-caller localStorage entry behind.
    let persist_key_for_clear = Rc::clone(&persist_key_cell);
    use_effect(move || {
        if !any_ctx.is_submitted() {
            return;
        }
        if let Some(key) = persist_key_for_clear.borrow().clone() {
            remove_persisted(&key);
        }
    });

    let on_step_for_effect = Rc::clone(&on_step_change_cell);
    let mut prev_index = use_signal(|| 0usize);
    use_effect(move || {
        let cb = *on_step_for_effect.borrow();
        let Some(cb) = cb else {
            return;
        };
        let cur = any_ctx.current_index();
        let prev = *prev_index.peek();
        if cur != prev {
            prev_index.set(cur);
            cb.call((prev, cur));
        }
    });

    let _current_step_anchor = internal.current.read();

    rsx! {
        FormProvider { form: form, loading: disabled,
            Form { children: rsx! { {children} } }
        }
    }
}

#[component]
pub fn StepNav<T>(
    form: Form<T>,
    on_submit: EventHandler<T>,
    #[props(default)] back_label: Option<String>,
    #[props(default)] next_label: Option<String>,
    #[props(default)] skip_label: Option<String>,
    #[props(default)] submit_label: Option<String>,
    #[props(default)] allow_back: Option<bool>,
    /// Return `false` to cancel moving to the next step (Leptos `before_next` parity).
    #[props(default)]
    before_next: Option<Callback<(), bool>>,
    #[props(default)] class: String,
) -> Element
where
    T: FormData + Send + Sync + Clone + 'static,
{
    let step_ctx = use_context::<AnyStepCtx>();
    let allow_back = allow_back.unwrap_or(true);

    let back_text = back_label.unwrap_or_else(|| "Back".to_string());
    let next_text = next_label.unwrap_or_else(|| "Next".to_string());
    let submit_text = submit_label.unwrap_or_else(|| "Submit".to_string());

    let container_class = merge(&[
        "flex flex-row flex-wrap items-center gap-3 w-full mt-6",
        &class,
    ]);

    let form_next = form;

    let primary_label = use_memo({
        let submit_text = submit_text.clone();
        let next_text = next_text.clone();
        move || {
            if step_ctx.is_last() {
                submit_text.clone()
            } else {
                next_text.clone()
            }
        }
    });

    rsx! {
        div {
            "data-name": "StepNav",
            class: "{container_class}",
            hidden: if *step_ctx.completed.read() { true },
            if allow_back && !step_ctx.is_first() {
                button {
                    r#type: "button",
                    class: "{BTN_BASE} {BTN_OUTLINE}",
                    onclick: move |_| step_ctx.back(),
                    "{back_text}"
                }
            }

            div { class: "ml-auto flex flex-row flex-wrap items-center justify-end gap-3",
                if let Some(skip_label) = skip_label {
                    button {
                        r#type: "button",
                        class: "{BTN_BASE} {BTN_GHOST}",
                        onclick: move |_| {
                            if step_ctx.is_last() {
                                step_ctx.mark_submitted();
                                // Submit with current values (skip validation)
                                if let Some(data) = form_next.get_data() {
                                    on_submit.call(data);
                                }
                            } else {
                                step_ctx.next();
                            }
                        },
                        "{skip_label}"
                    }
                }

                button {
                    r#type: "button",
                    class: "{BTN_BASE} {BTN_PRIMARY}",
                    onclick: move |_| {
                        if let Some(guard) = before_next
                            && !guard.call(())
                        {
                            return;
                        }

                        let fields = step_ctx.current_fields();
                        if !validate_step_fields(&form, &fields) {
                            return;
                        }

                        if step_ctx.is_last() {
                            step_ctx.mark_submitted();
                            form.submit(move |data| on_submit.call(data));
                        } else {
                            step_ctx.next();
                        }
                    },
                    "{primary_label()}"
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum StepProgressVariant {
    Counter,
    #[default]
    Horizontal,
    Vertical,
}

#[component]
pub fn StepProgress(
    #[props(default)] variant: StepProgressVariant,
    #[props(default)] class: String,
) -> Element {
    let ctx = use_context::<AnyStepCtx>();
    let steps_list = use_memo(move || ctx.steps_meta());
    let counter_label =
        use_memo(move || format!("Step {} of {}", ctx.current_index() + 1, ctx.total()));
    let current_step_str = use_memo(move || ctx.current_index().to_string());

    // Per-step element refs so the active step can be scrolled into view when the progress bar
    // overflows. Hooks run before `match variant` so order stays stable across variant changes.
    let mut item_refs = use_signal(Vec::<Option<Rc<MountedData>>>::new);
    use_effect(move || {
        let idx = ctx.current_index();
        if let Some(Some(el)) = item_refs.read().get(idx) {
            let el = el.clone();
            spawn(async move {
                let _ = el
                    .scroll_to_with_options(ScrollToOptions {
                        behavior: ScrollBehavior::Smooth,
                        vertical: ScrollLogicalPosition::Nearest,
                        horizontal: ScrollLogicalPosition::Center,
                    })
                    .await;
            });
        }
    });

    match variant {
        StepProgressVariant::Counter => {
            let counter_class = merge(&["text-sm text-muted-foreground font-medium", &class]);
            rsx! {
                div {
                    "data-name": "StepProgress",
                    class: "{counter_class}",
                    hidden: if *ctx.completed.read() { true },
                    "data-current-step": "{current_step_str()}",
                    "{counter_label()}"
                }
            }
        }
        StepProgressVariant::Horizontal => {
            let container_class = merge(&[
                "flex flex-row flex-nowrap items-center gap-0 min-w-0 w-full overflow-x-auto [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden",
                &class,
            ]);
            let steps_vec = steps_list();
            let len = steps_vec.len();
            rsx! {
                nav {
                    "data-name": "StepProgress",
                    class: "{container_class}",
                    hidden: if *ctx.completed.read() { true },
                    role: "list",
                    "aria-label": "Progress",
                    "data-current-step": "{current_step_str()}",
                    for (i, (_, title)) in steps_vec.into_iter().enumerate() {
                        {
                            let state = ctx.step_state(i);
                            let is_last = i == len - 1;
                            let clickable = ctx.is_visited_index(i) || state == StepState::Current;
                            let is_current = state == StepState::Current;
                            let circle_class = step_circle_class(state, clickable);
                            let label_class = match state {
                                StepState::Upcoming => {
                                    "text-sm font-medium whitespace-nowrap text-muted-foreground max-sm:max-w-[5.5rem] max-sm:truncate"
                                }
                                _ => {
                                    "text-sm font-medium whitespace-nowrap text-foreground max-sm:max-w-[5.5rem] max-sm:truncate"
                                }
                            };
                            rsx! {
                                div {
                                    role: "listitem",
                                    class: "flex flex-row flex-nowrap items-center shrink-0",
                                    "aria-current": if is_current { "step" },
                                    onmounted: move |e| {
                                        item_refs.with_mut(|v| {
                                            if v.len() != len {
                                                v.resize(len, None);
                                            }
                                            v[i] = Some(e.data());
                                        });
                                    },
                                    div { class: "flex flex-row flex-nowrap items-center gap-2 min-w-0",
                                        button {
                                            r#type: "button",
                                            disabled: !clickable,
                                            class: circle_class,
                                            onclick: move |_| {
                                                if clickable { ctx.go_to_index(i); }
                                            },
                                            if state == StepState::Completed {
                                                Icon { name: IconName::Check, class: "size-3.5", stroke_width: 2.5 }
                                            } else {
                                                span { "{i + 1}" }
                                            }
                                        }
                                        span { class: label_class, "{title}" }
                                    }
                                    if !is_last {
                                        div { class: "mx-2 sm:mx-3 flex h-8 shrink-0 items-center",
                                            div { class: "h-px w-6 sm:w-12 bg-border" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        StepProgressVariant::Vertical => {
            let container_class = merge(&["flex flex-col gap-0 min-w-[180px]", &class]);
            let steps_vec = steps_list();
            let len = steps_vec.len();
            rsx! {
                nav {
                    "data-name": "StepProgress",
                    class: "{container_class}",
                    hidden: if *ctx.completed.read() { true },
                    role: "list",
                    "aria-label": "Progress",
                    "data-current-step": "{current_step_str()}",
                    for (i, (_, title)) in steps_vec.into_iter().enumerate() {
                        {
                            let state = ctx.step_state(i);
                            let is_last = i == len - 1;
                            let clickable = ctx.is_visited_index(i) || state == StepState::Current;
                            let is_current = state == StepState::Current;
                            let circle_class = step_circle_class(state, clickable);
                            let connector_class = if state == StepState::Completed {
                                "w-[1px] flex-1 min-h-6 my-1 bg-primary"
                            } else {
                                "w-[1px] flex-1 min-h-6 my-1 bg-border"
                            };
                            let label_class = step_vlabel_class(state, clickable);
                            let pb_class = if is_last { "pb-0 pt-1.5" } else { "pb-4 pt-1.5" };
                            rsx! {
                                div {
                                    role: "listitem",
                                    class: "flex items-stretch gap-3",
                                    "aria-current": if is_current { "step" },
                                    div { class: "flex flex-col items-center",
                                        button {
                                            r#type: "button",
                                            disabled: !clickable,
                                            class: circle_class,
                                            onclick: move |_| {
                                                if clickable { ctx.go_to_index(i); }
                                            },
                                            if state == StepState::Completed {
                                                Icon { name: IconName::Check, class: "size-3.5", stroke_width: 2.5 }
                                            } else {
                                                span { "{i + 1}" }
                                            }
                                        }
                                        if !is_last {
                                            div { class: connector_class }
                                        }
                                    }
                                    div { class: pb_class,
                                        button {
                                            r#type: "button",
                                            disabled: !clickable,
                                            class: label_class,
                                            onclick: move |_| {
                                                if clickable { ctx.go_to_index(i); }
                                            },
                                            "{title}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
