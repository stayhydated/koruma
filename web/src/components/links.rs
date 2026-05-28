use crate::site::routing::PageKind;
use dioxus::prelude::*;
use stayhydated_dioxus::{RouteCardLink, RouteLink};

#[component]
pub(crate) fn PageLink(page: PageKind, class: String, label: String) -> Element {
    rsx! {
        RouteLink {
            route: crate::site::routing::app_route(page),
            href: crate::site::routing::page_href(page),
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
            route: crate::site::routing::app_route(page),
            href: crate::site::routing::page_href(page),
            label,
            title,
            body,
            body_class: String::new(),
            action,
        }
    }
}
