use iocraft::prelude::*;

use super::theme::Theme;

// ─── Section: bordered box with title ───────────────────────────────

#[derive(Default, Props)]
pub struct SectionProps<'a> {
    pub title: String,
    pub children: Vec<AnyElement<'a>>,
    pub variant: SectionVariant,
}

#[derive(Default, Clone, Copy)]
pub enum SectionVariant {
    #[default]
    Default,
    Success,
    Warning,
    Error,
}

#[component]
pub fn Section<'a>(props: &mut SectionProps<'a>) -> impl Into<AnyElement<'a>> + use<'a> {
    let border_color = match props.variant {
        SectionVariant::Default => Some(Theme::BORDER),
        SectionVariant::Success => Some(Theme::SUCCESS),
        SectionVariant::Warning => Some(Theme::WARNING),
        SectionVariant::Error => Some(Theme::ERROR),
    };

    let title_color = match props.variant {
        SectionVariant::Default => Some(Theme::ACCENT),
        SectionVariant::Success => Some(Theme::SUCCESS),
        SectionVariant::Warning => Some(Theme::WARNING),
        SectionVariant::Error => Some(Theme::ERROR),
    };

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Round,
            border_color: border_color,
            padding_left: 1,
            padding_right: 1,
            margin_bottom: 1,
        ) {
            View(margin_bottom: 1) {
                Text(
                    content: props.title.clone(),
                    color: title_color,
                    weight: Weight::Bold,
                )
            }
            #(&mut props.children)
        }
    }
}

// ─── Entry: key-value pair ──────────────────────────────────────────

#[derive(Default, Props)]
pub struct EntryProps {
    pub label: String,
    pub value: String,
    pub value_color: Option<Color>,
}

#[component]
pub fn Entry<'a>(props: &EntryProps) -> impl Into<AnyElement<'a>> + use<'a> {
    element! {
        View(flex_direction: FlexDirection::Row) {
            View(width: 20) {
                Text(
                    content: format!("  {}", props.label),
                    color: Some(Theme::MUTED),
                )
            }
            Text(
                content: props.value.clone(),
                color: props.value_color.or(Some(Color::White)),
            )
        }
    }
}

// ─── StatusLine: icon + message ─────────────────────────────────────

#[derive(Default, Props)]
pub struct StatusLineProps {
    pub icon: String,
    pub message: String,
    pub variant: StatusVariant,
}

#[derive(Default, Clone, Copy)]
pub enum StatusVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
    Muted,
}

#[component]
pub fn StatusLine<'a>(props: &StatusLineProps) -> impl Into<AnyElement<'a>> + use<'a> {
    let color = match props.variant {
        StatusVariant::Info => Some(Theme::ACCENT),
        StatusVariant::Success => Some(Theme::SUCCESS),
        StatusVariant::Warning => Some(Theme::WARNING),
        StatusVariant::Error => Some(Theme::ERROR),
        StatusVariant::Muted => Some(Theme::MUTED),
    };

    element! {
        View(flex_direction: FlexDirection::Row) {
            Text(content: format!(" {} ", props.icon))
            Text(content: props.message.clone(), color: color)
        }
    }
}

// ─── Header: top-level command header ───────────────────────────────

#[derive(Default, Props)]
pub struct HeaderProps {
    pub title: String,
    pub subtitle: Option<String>,
}

#[component]
pub fn Header<'a>(props: &HeaderProps) -> impl Into<AnyElement<'a>> + use<'a> {
    element! {
        View(flex_direction: FlexDirection::Column, margin_bottom: 1) {
            Text(
                content: props.title.clone(),
                weight: Weight::Bold,
                color: Some(Theme::ACCENT),
            )
            #(props.subtitle.as_ref().map(|s| {
                element! {
                    Text(content: s.clone(), color: Some(Theme::MUTED))
                }
            }))
        }
    }
}

// ─── ToolchainRow: for toolchain list ───────────────────────────────

#[derive(Default, Props)]
pub struct ToolchainRowProps {
    pub name: String,
    pub spec: String,
    pub gcc_version: String,
    pub size: String,
    pub source: String,
    pub active: bool,
}

#[component]
pub fn ToolchainRow<'a>(props: &ToolchainRowProps) -> impl Into<AnyElement<'a>> + use<'a> {
    let marker = if props.active { "▸" } else { " " };
    let name_color = if props.active {
        Some(Theme::ACCENT)
    } else {
        Some(Color::White)
    };

    element! {
        View(flex_direction: FlexDirection::Row) {
            View(width: 3) {
                Text(
                    content: marker.to_string(),
                    color: if props.active { Some(Theme::ACCENT) } else { None },
                )
            }
            View(width: 18) {
                Text(content: props.name.clone(), color: name_color, weight: Weight::Bold)
            }
            View(width: 14) {
                Text(content: format!("gcc {}", props.gcc_version), color: Some(Theme::MUTED))
            }
            View(width: 10) {
                Text(content: props.size.clone(), color: Some(Theme::MUTED))
            }
            #(if !props.spec.is_empty() {
                Some(element! {
                    Text(content: props.spec.clone(), color: Some(Theme::MUTED))
                })
            } else {
                None
            })
        }
    }
}
