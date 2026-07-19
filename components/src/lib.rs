#![recursion_limit = "256"]
#![allow(non_snake_case)]

// ---- core (always compiled) ----

pub(crate) mod back;
pub(crate) mod badge;
pub(crate) mod button;
pub(crate) mod card;
pub(crate) mod copyable;
pub mod field_name;
pub(crate) mod focus;
// Shared behavior hooks — infrastructure consumed by component migrations
// (see docs/audits/components-audit.md §3); allow dead_code until adopted.
// unused_imports fires for re-exports whose consumers are feature-gated off.
#[allow(dead_code, unused_imports)]
pub(crate) mod hooks;
pub mod icon;
pub(crate) mod icon_bubble;
pub(crate) mod label;
pub(crate) mod layout;
pub(crate) mod link;
pub(crate) mod placement;
pub(crate) mod portal;
pub(crate) mod separator;
pub(crate) mod spinner;
pub(crate) mod text;
pub(crate) mod title;

// ---- form family ----

#[cfg(feature = "form")]
pub(crate) mod checkbox;
#[cfg(feature = "form")]
pub(crate) mod chip_toggle;
#[cfg(feature = "form")]
pub(crate) mod color_swatch_picker;
#[cfg(feature = "form")]
pub(crate) mod file_upload;
#[cfg(feature = "form")]
pub mod form;
#[cfg(feature = "form")]
pub(crate) mod input;
#[cfg(feature = "form")]
pub(crate) mod input_types;
#[cfg(feature = "form")]
pub(crate) mod number_stepper;
#[cfg(feature = "form")]
pub(crate) mod password_strength;
#[cfg(feature = "form")]
pub(crate) mod radio;
#[cfg(feature = "form")]
pub(crate) mod select;
#[cfg(feature = "form")]
pub(crate) mod slider;
#[cfg(feature = "form")]
pub(crate) mod step_dots;
#[cfg(feature = "form")]
pub mod stepper;
#[cfg(feature = "form")]
pub(crate) mod textarea;
#[cfg(feature = "form")]
pub(crate) mod toggle_card;
#[cfg(feature = "form")]
pub(crate) mod use_action_feedback;

// ---- other families ----

#[cfg(feature = "calendar")]
pub mod calendar;
#[cfg(feature = "charts")]
pub mod charts;
#[cfg(feature = "data-table")]
pub mod data_table;
#[cfg(feature = "data-table")]
pub(crate) mod list_view;
#[cfg(feature = "data-table")]
pub(crate) mod resource_view;
#[cfg(feature = "date-picker")]
pub(crate) mod date_picker;
#[cfg(feature = "rich-text")]
pub(crate) mod rich_text_editor;

#[cfg(feature = "nav")]
pub(crate) mod app_shell;
#[cfg(feature = "nav")]
pub(crate) mod nav_sliding_indicator;
#[cfg(feature = "nav")]
pub(crate) mod nav_tabs;
#[cfg(feature = "nav")]
pub(crate) mod route_transition_outlet;
#[cfg(feature = "nav")]
pub(crate) mod segmented_control;
#[cfg(feature = "nav")]
pub(crate) mod tabs;

#[cfg(feature = "feedback")]
pub mod alert;
#[cfg(feature = "feedback")]
pub mod fallback_view;
#[cfg(feature = "feedback")]
pub(crate) mod loading_overlay;
#[cfg(feature = "feedback")]
pub(crate) mod progress;
#[cfg(feature = "feedback")]
pub(crate) mod status_dot;
#[cfg(feature = "feedback")]
pub(crate) mod toast;
#[cfg(feature = "feedback")]
pub(crate) mod tooltip;

#[cfg(feature = "overlay")]
pub(crate) mod dropdown;
#[cfg(feature = "overlay")]
pub mod modal;
#[cfg(feature = "overlay")]
pub(crate) mod popover;

#[cfg(feature = "display")]
pub(crate) mod accordion;
#[cfg(feature = "display")]
pub mod avatar;
#[cfg(feature = "display")]
pub(crate) mod empty_state;
#[cfg(feature = "display")]
pub(crate) mod section_header;
#[cfg(feature = "display")]
pub(crate) mod stat_tile;

// ---- core re-exports ----

pub use back::Back;
pub use badge::{Badge, BadgeSize, BadgeVariant};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use card::{Card, CardVariant};
pub use copyable::Copyable;
pub use field_name::{Field, FieldArray, FieldKey, FieldName, FieldPath, FieldType, FormSchema};
pub use focus::active_element_id;
pub use hooks::{use_escape_listener, use_outside_dismiss};
pub use icon::{Icon, IconName};
pub use icon_bubble::{IconBubble, IconBubbleColor, IconBubbleSize};
pub use label::Label;
pub use layout::{
    Column, Container, Flex, FlexAlign, FlexDirection, FlexGap, FlexGridCols, FlexJustify,
    FlexWrap, Grid, Row,
};
pub use link::{Link, LinkType};
pub use placement::{Align, Placement};
pub use portal::Portal;
pub use separator::{Separator, SeparatorOrientation};
#[doc(hidden)]
pub use serde_json;
pub use spinner::Spinner;
pub use text::{Text, TextSize, TextTone, TextVariant, TextWeight};
pub use title::{Title, TitleSize};

// ---- form family re-exports ----

#[cfg(feature = "form")]
pub use checkbox::{Checkbox, CheckboxBase};
#[cfg(feature = "form")]
pub use chip_toggle::ChipToggle;
#[cfg(feature = "form")]
pub use color_swatch_picker::{ColorSwatchOption, ColorSwatchPicker};
#[cfg(feature = "form")]
pub use ds_macros::{FormFields, FormOptions, Steps};
#[cfg(feature = "form")]
pub use file_upload::{FileInfo, FileUpload};
#[cfg(feature = "form")]
pub use form::{
    BareTextInput, Form, FormData, FormProvider, FormSubmit, NumberInput, PasswordInput, TextInput,
    use_form,
};
// Root exports for the typed form store; the derive macros reference these
// paths (`components::FormValue`).
#[cfg(feature = "form")]
pub use form::typed::{FormValue, ParseError};
#[cfg(feature = "form")]
#[allow(deprecated)]
pub use input::InputSize;
#[cfg(feature = "form")]
pub use input::{AutofocusGate, FieldSize, InputBase, InputType};
#[cfg(feature = "form")]
pub use input_types::{
    EmailInputBase, NumberInputBase, PasswordInputBase, PercentageInputBase, PhoneInputBase,
    TextInputBase, TypedInputBaseProps,
};
#[cfg(feature = "form")]
pub use number_stepper::NumberStepper;
#[cfg(feature = "form")]
pub use password_strength::{PasswordStrength, PasswordStrengthProps};
#[cfg(feature = "form")]
pub use radio::{Radio, RadioGroup, RadioGroupDirection};
#[cfg(feature = "form")]
pub use select::{Select, SelectBase, SelectOption, use_select_contexts};
#[cfg(feature = "form")]
pub use slider::Slider;
#[cfg(feature = "form")]
pub use step_dots::{StepDots, StepKey, StepMeta};
#[cfg(feature = "form")]
#[deprecated(note = "renamed to `StepDefinition`")]
pub use stepper::StepDefination;
#[cfg(feature = "form")]
pub use stepper::{
    AnyStepCtx, ClearDraftButton, Direction, MultiStepForm, Step, StepCtx, StepDefinition, StepId,
    StepInfo, StepNav, StepProgress, StepProgressVariant, StepState, StepSuccess, Stepper,
    SummaryField, SummarySection, use_step, use_step_ctx,
};
#[cfg(feature = "form")]
pub use textarea::{TextArea, TextAreaBase, TextAreaResize, textarea_insert_at_cursor};
#[cfg(feature = "form")]
pub use toggle_card::{Switch, ToggleCard};
#[cfg(feature = "form")]
pub use use_action_feedback::use_action_feedback;

// ---- other family re-exports ----

#[cfg(feature = "calendar")]
pub use calendar::{CalendarEvent, MonthView, TimeGrid, TimeGridEvent};
#[cfg(feature = "charts")]
pub use charts::{
    AreaLineChart, ChartSegment, DonutChart, LineMarker, LinePoint, LineSeries, SegmentColor,
    StackedBarChart,
};
#[cfg(feature = "data-table")]
pub use data_table::{
    Col, ColRenderFn, DataTable, DataTableSkeleton, ItemKeyProp, SortDir, TableColumn, col,
};
#[cfg(feature = "data-table")]
pub use list_view::{FetchFn, ListEmpty, ListPage, ListView, RenderFn};
#[cfg(feature = "data-table")]
pub use resource_view::ResourceView;
#[cfg(feature = "date-picker")]
pub use date_picker::{
    Date, DatePicker, DatePickerBase, DateRangePicker, DateTime, DateTimePicker, DateTimePickerBase,
};
#[cfg(feature = "rich-text")]
pub use rich_text_editor::{RichTextEditor, RichTextEditorBase, rte_insert_text};

#[cfg(feature = "nav")]
pub use app_shell::AppShellProvider;
#[cfg(all(feature = "nav", target_arch = "wasm32"))]
pub use nav_sliding_indicator::sliding_indicator_style;
#[cfg(feature = "nav")]
pub use nav_sliding_indicator::{
    HORIZONTAL_SLIDING_INDICATOR_CLASS, SlidingIndicatorAxis, VERTICAL_SLIDING_INDICATOR_CLASS,
    sliding_indicator_class,
};
#[cfg(feature = "nav")]
pub use nav_tabs::{NavItem, NavTabs, NavTabsDirection};
#[cfg(feature = "nav")]
pub use route_transition_outlet::RouteTransitionOutlet;
#[cfg(feature = "nav")]
pub use segmented_control::SegmentedControl;
#[cfg(feature = "nav")]
pub use tabs::{TabItem, TabType, Tabs};

#[cfg(feature = "feedback")]
pub use alert::{Alert, AlertVariant};
#[cfg(feature = "feedback")]
pub use fallback_view::{
    AppRouteErrorFallback, NotFound, PageLoader, SectionErrorFallback, SectionLoader,
    WorkInProgress,
};
#[cfg(feature = "feedback")]
pub use loading_overlay::LoadingOverlay;
#[cfg(feature = "feedback")]
pub use progress::Progress;
#[cfg(feature = "feedback")]
pub use status_dot::{DotTone, StatusDot};
#[cfg(feature = "feedback")]
pub use toast::{ToastItem, ToastPlacement, ToastStore, ToastVariant, Toaster, use_toast};
#[cfg(feature = "feedback")]
pub use tooltip::Tooltip;

#[cfg(feature = "overlay")]
pub use dropdown::{
    DropdownCloseButton, DropdownMenu, DropdownMenuAlign, DropdownMenuCoordinatorProvider,
    DropdownMenuGroup, DropdownMenuItem, DropdownMenuRadioItem, DropdownMenuSeparator,
    DropdownMenuSize, DropdownMenuSub,
};
#[cfg(feature = "overlay")]
pub use modal::{Modal, ModalSize};
#[cfg(feature = "overlay")]
pub use popover::{Popover, PopoverConfirm};

#[cfg(feature = "display")]
pub use accordion::{Accordion, AccordionItem};
#[cfg(feature = "display")]
pub use avatar::Avatar;
#[cfg(feature = "display")]
pub use empty_state::EmptyState;
#[cfg(feature = "display")]
pub use section_header::{SectionHeader, SectionHeaderTitle};
#[cfg(feature = "display")]
pub use stat_tile::{StatTile, StatTone};
