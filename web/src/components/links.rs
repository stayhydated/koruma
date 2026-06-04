use crate::site::routing::PageKind;
use dioxus::prelude::*;
use stayhydated_dioxus::{CssClass, DisplayText, LinkTarget, RouteCardLink, RouteLink};

#[component]
pub(crate) fn PageLink(page: PageKind, class: String, label: String) -> Element {
    rsx! {
        RouteLink {
            target: LinkTarget::route(crate::site::routing::app_route(page)),
            class: CssClass::new(class),
            label: DisplayText::new(label),
        }
    }
}

#[component]
pub(crate) fn PageCardLink(
    page: PageKind,
    label: String,
    title: String,
    body: String,
    action: String,
) -> Element {
    rsx! {
        RouteCardLink {
            target: LinkTarget::route(crate::site::routing::app_route(page)),
            label: DisplayText::new(label),
            title: DisplayText::new(title),
            body: DisplayText::new(body),
            body_class: CssClass::default(),
            action: DisplayText::new(action),
        }
    }
}
