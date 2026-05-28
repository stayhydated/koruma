use crate::pages;
use crate::site::i18n::PageMetadataMessage;
use dioxus::cli_config;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::{DioxusI18n, use_i18n};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PageKind {
    Home,
    Demos,
    CollectionDioxus,
    SalesForm,
}

impl PageKind {
    pub(crate) fn all() -> [Self; 4] {
        [
            Self::Home,
            Self::Demos,
            Self::CollectionDioxus,
            Self::SalesForm,
        ]
    }

    pub(crate) fn route(self) -> &'static str {
        match self {
            Self::Home => "",
            Self::Demos => "demos",
            Self::CollectionDioxus => "demos/koruma-collection",
            Self::SalesForm => "demos/sales-form",
        }
    }

    pub(crate) fn path(self) -> String {
        let route = self.route();
        if route.is_empty() {
            "/".to_string()
        } else {
            format!("/{route}/")
        }
    }

    pub(crate) fn output_dir(self) -> &'static str {
        self.route()
    }

    pub(crate) fn title_i18n(self, i18n: &DioxusI18n) -> String {
        match self {
            Self::Home => i18n.localize_message(&PageMetadataMessage::HomeTitle),
            Self::Demos => i18n.localize_message(&PageMetadataMessage::DemosTitle),
            Self::CollectionDioxus => i18n.localize_message(&PageMetadataMessage::DioxusDemoTitle),
            Self::SalesForm => i18n.localize_message(&PageMetadataMessage::SalesFormDemoTitle),
        }
    }

    pub(crate) fn description_i18n(self, i18n: &DioxusI18n) -> String {
        match self {
            Self::Home => i18n.localize_message(&PageMetadataMessage::HomeDescription),
            Self::Demos => i18n.localize_message(&PageMetadataMessage::DemosDescription),
            Self::CollectionDioxus => {
                i18n.localize_message(&PageMetadataMessage::DioxusDemoDescription)
            },
            Self::SalesForm => {
                i18n.localize_message(&PageMetadataMessage::SalesFormDemoDescription)
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
pub(crate) enum AppRoute {
    #[route("/", HomeRoute)]
    Home {},
    #[route("/demos/", DemosRoute)]
    Demos {},
    #[route("/demos/koruma-collection/", CollectionDioxusRoute)]
    CollectionDioxus {},
    #[route("/demos/sales-form/", SalesFormRoute)]
    SalesForm {},
}

pub(crate) fn app_route(page: PageKind) -> AppRoute {
    match page {
        PageKind::Home => AppRoute::Home {},
        PageKind::Demos => AppRoute::Demos {},
        PageKind::CollectionDioxus => AppRoute::CollectionDioxus {},
        PageKind::SalesForm => AppRoute::SalesForm {},
    }
}

pub(crate) fn app_base_href() -> String {
    let base_path = cli_config::base_path();
    stayhydated_site::routing::base_href(base_path.as_deref())
}

pub(crate) fn page_href(page: PageKind) -> String {
    stayhydated_site::routing::href(&app_base_href(), page.route())
}

pub(crate) fn book_href() -> String {
    stayhydated_site::routing::href(&app_base_href(), "book")
}

const GENERATED_ROUTE_CACHE_MARKER: &str = ".koruma-generated-route-cache";

pub(crate) fn mark_generated_route_cache(public_dir: &Path) -> std::io::Result<()> {
    stayhydated_site::route_cache::mark_generated_route_cache(
        public_dir,
        GENERATED_ROUTE_CACHE_MARKER,
        "Generated route cache owned by koruma web server.\n",
    )
}

pub(crate) fn cleanup_generated_route_cache(public_dir: &Path) -> std::io::Result<()> {
    let generated_top_level_dirs = PageKind::all().into_iter().filter_map(|page| {
        page.output_dir()
            .split('/')
            .next()
            .filter(|segment| !segment.is_empty())
    });

    stayhydated_site::route_cache::cleanup_generated_route_cache(
        public_dir,
        GENERATED_ROUTE_CACHE_MARKER,
        generated_top_level_dirs,
        |_, _| false,
    )
}

#[cfg(test)]
pub(crate) fn site_route_from_path_with_base_path(path: &str, base_path: Option<&str>) -> PageKind {
    let segments = normalized_path_segments(path, base_path);
    page_from_segments(&segments)
}

#[cfg(test)]
fn normalized_path_segments<'a>(path: &'a str, base_path: Option<&str>) -> Vec<&'a str> {
    stayhydated_site::routing::normalized_path_segments(path, base_path)
}

#[cfg(test)]
fn page_from_segments(segments: &[&str]) -> PageKind {
    match segments {
        [] => PageKind::Home,
        ["demos"] => PageKind::Demos,
        ["demos", "koruma-collection"] => PageKind::CollectionDioxus,
        ["demos", "sales-form"] => PageKind::SalesForm,
        _ => PageKind::Home,
    }
}

fn route_element(page: PageKind) -> Element {
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                Title { "koruma" }
                Meta {
                    name: "description",
                    content: "Failed to initialize i18n",
                }
                div { class: "page-shell", "Failed to initialize i18n: {error}" }
                {pages::route_content(page)}
            };
        },
    };

    let title = page.title_i18n(&i18n);
    let description = page.description_i18n(&i18n);

    rsx! {
        Title { "{title}" }
        Meta {
            name: "description",
            content: description,
        }
        {pages::route_content(page)}
    }
}

#[server(endpoint = "static_routes")]
async fn static_routes() -> Result<Vec<String>, ServerFnError> {
    Ok(PageKind::all().into_iter().map(page_href).collect())
}

#[component]
fn HomeRoute() -> Element {
    route_element(PageKind::Home)
}

#[component]
fn DemosRoute() -> Element {
    route_element(PageKind::Demos)
}

#[component]
fn CollectionDioxusRoute() -> Element {
    route_element(PageKind::CollectionDioxus)
}

#[component]
fn SalesFormRoute() -> Element {
    route_element(PageKind::SalesForm)
}
