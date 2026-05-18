use crate::site::routing::PageKind;
use dioxus::prelude::*;

#[component]
pub(crate) fn PageLink(page: PageKind, class: String, label: String) -> Element {
    if try_router().is_some() {
        rsx! {
            Link {
                class,
                to: crate::site::routing::app_route(page),
                "{label}"
            }
        }
    } else {
        rsx! {
            a {
                class,
                href: crate::site::routing::page_href(page),
                "{label}"
            }
        }
    }
}
