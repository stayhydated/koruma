use crate::site::routing::PageKind;
use dioxus::prelude::*;
use stayhydated_dioxus::{LinkTarget, RouteCardLink, RouteLink};

#[component]
pub(crate) fn PageLink(page: PageKind, class: String, label: String) -> Element {
    rsx! {
        RouteLink {
            target: LinkTarget::route(crate::site::routing::app_route(page)),
            class,
            label,
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
            label,
            title,
            body,
            body_class: String::new(),
            action,
        }
    }
}
