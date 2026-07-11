#![recursion_limit = "256"]
#![allow(non_snake_case)]

extern crate self as components;

pub(crate) mod accordion;
pub mod alert;
pub(crate) mod app_shell;
pub mod avatar;
pub(crate) mod badge;
pub mod calendar;
pub mod charts;
pub mod data_table;
pub mod field_name;
pub mod form;
// Shared behavior hooks — infrastructure consumed by component migrations
// (see docs/audits/components-audit.md §3); allow dead_code until adopted.
#[allow(dead_code)]
pub(crate) mod hooks;
pub mod icon;
pub mod modal;
pub mod stepper;

pub(crate) mod back;
pub(crate) mod button;
pub(crate) mod card;
pub(crate) mod checkbox;
pub(crate) mod chip_toggle;
pub(crate) mod color_swatch_picker;
pub(crate) mod copyable;
pub(crate) mod date_picker;
pub(crate) mod dropdown;
pub(crate) mod empty_state;
pub mod fallback_view;
pub(crate) mod file_upload;
pub(crate) mod focus;
pub(crate) mod icon_bubble;
pub(crate) mod input;
pub(crate) mod label;
pub(crate) mod layout;
pub(crate) mod link;
pub(crate) mod list_view;
pub(crate) mod loading_overlay;
pub(crate) mod logo;
pub(crate) mod nav_sliding_indicator;
pub(crate) mod nav_tabs;
pub(crate) mod placement;
pub(crate) mod popover;
pub(crate) mod portal;
pub(crate) mod radio;
pub(crate) mod resource_view;
pub(crate) mod rich_text_editor;
pub(crate) mod route_transition_outlet;
pub(crate) mod section_header;
pub(crate) mod segmented_control;
pub(crate) mod select;
pub(crate) mod separator;
pub(crate) mod spinner;
pub(crate) mod tabs;
pub(crate) mod text;
pub(crate) mod textarea;
pub(crate) mod title;
pub(crate) mod toast;
pub(crate) mod toggle_card;
pub(crate) mod tooltip;
pub(crate) mod use_action_feedback;

pub use accordion::{Accordion, AccordionItem};
pub use alert::{Alert, AlertVariant};
pub use app_shell::AppShellProvider;
pub use avatar::Avatar;
pub use back::Back;
pub use badge::{Badge, BadgeSize, BadgeVariant};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use calendar::{CalendarEvent, MonthView, TimeGrid, TimeGridEvent};
pub use card::Card;
pub use charts::{
    AreaLineChart, ChartSegment, DonutChart, LineMarker, LinePoint, LineSeries, SegmentColor,
    StackedBarChart,
};
pub use checkbox::{Checkbox, CheckboxBase};
pub use chip_toggle::ChipToggle;
pub use color_swatch_picker::{ColorSwatchOption, ColorSwatchPicker};
pub use copyable::Copyable;
pub use data_table::{
    Col, ColRenderFn, DataTable, DataTableSkeleton, ItemKeyProp, SortDir, TableColumn, col,
};
pub use date_picker::{
    DatePicker, DatePickerBase, DateRangePicker, DateTimePicker, DateTimePickerBase,
};
pub use dropdown::{
    DropdownCloseButton, DropdownMenu, DropdownMenuAlign, DropdownMenuCoordinatorProvider,
    DropdownMenuGroup, DropdownMenuItem, DropdownMenuRadioItem, DropdownMenuSeparator,
    DropdownMenuSize, DropdownMenuSub,
};
pub use empty_state::EmptyState;
pub use fallback_view::{
    AppRouteErrorFallback, NotFound, PageLoader, SectionErrorFallback, SectionLoader,
    WorkInProgress,
};
pub use field_name::{Field, FieldArray, FieldKey, FieldName, FieldPath, FieldType, FormSchema};
pub use file_upload::{FileInfo, FileUpload};
pub use focus::active_element_id;
pub use form::{Form, FormData, FormSubmit, use_form};
pub use icon::{Icon, IconName};
pub use icon_bubble::{IconBubble, IconBubbleColor, IconBubbleSize};
pub use input::{InputBase, InputSize, InputType};
pub use layout::{
    Column, Container, Flex, FlexAlign, FlexDirection, FlexGap, FlexGridCols, FlexJustify,
    FlexWrap, Grid, Row,
};
pub use link::{Link, LinkType};
pub use list_view::{FetchFn, ListEmpty, ListPage, ListView, RenderFn};
pub use loading_overlay::LoadingOverlay;
pub use logo::Logo;
pub use macros::{FormFields, FormOptions, Steps};
pub use modal::{Modal, ModalSize};
#[cfg(target_arch = "wasm32")]
pub use nav_sliding_indicator::sliding_indicator_style;
pub use nav_sliding_indicator::{
    HORIZONTAL_SLIDING_INDICATOR_CLASS, SlidingIndicatorAxis, VERTICAL_SLIDING_INDICATOR_CLASS,
    sliding_indicator_class,
};
pub use nav_tabs::{NavItem, NavTabs, NavTabsDirection};
pub use placement::{Align, Placement};
pub use popover::{Popover, PopoverConfirm};
pub use portal::Portal;
pub use radio::{RadioGroup, RadioGroupDirection};
pub use resource_view::ResourceView;
pub use rich_text_editor::{RichTextEditor, RichTextEditorBase, rte_insert_text};
pub use route_transition_outlet::RouteTransitionOutlet;
pub use section_header::{SectionHeader, SectionHeaderTitle};
pub use segmented_control::SegmentedControl;
pub use select::{Select, SelectBase, SelectOption, use_select_contexts};
pub use separator::{Separator, SeparatorOrientation};
#[doc(hidden)]
pub use serde_json;
pub use spinner::Spinner;
#[deprecated(note = "renamed to `StepDefinition`")]
pub use stepper::StepDefination;
pub use stepper::{
    AnyStepCtx, ClearDraftButton, Direction, MultiStepForm, Step, StepCtx, StepDefinition, StepId,
    StepInfo, StepNav, StepProgress, StepProgressVariant, StepState, StepSuccess, Stepper,
    SummaryField, SummarySection, use_step, use_step_ctx,
};
pub use tabs::{TabItem, TabType, Tabs};
pub use text::{Text, TextSize, TextVariant};
pub use textarea::{TextArea, TextAreaBase, TextAreaResize, textarea_insert_at_cursor};
pub use title::{Title, TitleSize};
pub use toast::{ToastItem, ToastPlacement, ToastStore, ToastVariant, Toaster, use_toast};
pub use toggle_card::{Switch, ToggleCard};
pub use tooltip::Tooltip;
pub use use_action_feedback::use_action_feedback;
