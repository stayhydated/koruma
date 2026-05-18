use crate::site::routing::PageKind;
use dioxus::prelude::*;

#[component]
pub(crate) fn FeatureCard(label: String, title: String, body: String) -> Element {
    rsx! {
        article { class: "feature-card",
            span { class: "card-label", "{label}" }
            h3 { "{title}" }
            p { "{body}" }
        }
    }
}

#[component]
pub(crate) fn DemoCardLink(
    page: PageKind,
    label: String,
    title: String,
    body: String,
    action: String,
) -> Element {
    if try_router().is_some() {
        rsx! {
            Link {
                class: "demo-card",
                to: crate::site::routing::app_route(page),
                span { class: "card-label", "{label}" }
                h2 { "{title}" }
                p { "{body}" }
                span { class: "card-link", "{action}" }
            }
        }
    } else {
        rsx! {
            a {
                class: "demo-card",
                href: crate::site::routing::page_href(page),
                span { class: "card-label", "{label}" }
                h2 { "{title}" }
                p { "{body}" }
                span { class: "card-link", "{action}" }
            }
        }
    }
}
