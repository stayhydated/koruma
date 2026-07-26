use dioxus::prelude::*;
use stayhydated_dioxus::{
    NavigationTarget, ShaderBackground, StayhydatedProjectPortalShell, page_entry_reveal_style,
};

use crate::site::{
    constants::{PROJECT, VERSION},
    routing::{AppRoute, PageKind},
};

#[component]
fn DemoCardContents(title: &'static str, shader_id: &'static str, time_offset: f32) -> Element {
    rsx! {
        ShaderBackground {
            canvas_id: shader_id,
            extra_class: "demo-card-shader",
            time_offset,
        }
        span { class: "demo-card-tint", aria_hidden: "true" }
        h2 { class: "demo-card-title", "{title}" }
    }
}

#[component]
fn DemoCardLink(
    route: AppRoute,
    title: &'static str,
    shader_id: &'static str,
    time_offset: f32,
) -> Element {
    let aria_label = format!("Open {title} demo");

    if try_router().is_some() {
        rsx! {
            Link {
                class: "demo-card",
                to: route,
                aria_label,
                DemoCardContents { title, shader_id, time_offset }
            }
        }
    } else {
        rsx! {
            a {
                class: "demo-card",
                href: route.to_string(),
                aria_label,
                DemoCardContents { title, shader_id, time_offset }
            }
        }
    }
}

#[component]
pub(crate) fn DemosPage() -> Element {
    let demos_style = page_entry_reveal_style().into_string();

    rsx! {
        StayhydatedProjectPortalShell {
            project: PROJECT,
            version: VERSION,
            home: NavigationTarget::Internal(crate::site::routing::app_route(PageKind::Home)),
            div { class: "demo-page demo-gallery",
                section {
                    class: "grid columns-2 demo-example-cards motion-reveal",
                    style: demos_style,
                    DemoCardLink {
                        route: crate::site::routing::app_route(PageKind::CollectionDioxus),
                        title: "koruma-collection",
                        shader_id: "collection-demo-card-shader",
                        time_offset: 0.0,
                    }
                    DemoCardLink {
                        route: crate::site::routing::app_route(PageKind::SalesForm),
                        title: "Sales form",
                        shader_id: "sales-form-demo-card-shader",
                        time_offset: 13.0,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demos_page_renders_shader_backed_cards() {
        let html = dioxus::ssr::render_element(rsx! { DemosPage {} });

        assert_eq!(html.matches("class=\"demo-card\"").count(), 2);
        assert_eq!(html.matches("class=\"demo-card-title\"").count(), 2);
        assert_eq!(html.matches("class=\"demo-card-tint\"").count(), 2);
        assert_eq!(
            html.matches("data-shader-background=\"loading\"").count(),
            2
        );
        assert!(html.contains("id=\"collection-demo-card-shader\""));
        assert!(html.contains("id=\"sales-form-demo-card-shader\""));
    }
}
