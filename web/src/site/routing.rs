use crate::pages;
use crate::site::i18n::PageMetadataMessage;
use dioxus::cli_config;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::{DioxusAssetI18nHandle, use_i18n};
use stayhydated_site::routing::{BaseHref, BasePath, Href, OutputDir, RoutePath};
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

    fn route(self) -> &'static str {
        match self {
            Self::Home => "",
            Self::Demos => "demos",
            Self::CollectionDioxus => "demos/koruma-collection",
            Self::SalesForm => "demos/sales-form",
        }
    }

    pub(crate) fn path(self) -> RoutePath {
        RoutePath::new(self.route())
    }

    fn title_i18n(self, i18n: &DioxusAssetI18nHandle) -> String {
        match self {
            Self::Home => i18n.localize_message(&PageMetadataMessage::HomeTitle),
            Self::Demos => i18n.localize_message(&PageMetadataMessage::DemosTitle),
            Self::CollectionDioxus => i18n.localize_message(&PageMetadataMessage::DioxusDemoTitle),
            Self::SalesForm => i18n.localize_message(&PageMetadataMessage::SalesFormDemoTitle),
        }
    }

    fn description_i18n(self, i18n: &DioxusAssetI18nHandle) -> String {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SiteRoute {
    pub(crate) page: PageKind,
}

impl SiteRoute {
    pub(crate) const fn new(page: PageKind) -> Self {
        Self { page }
    }

    pub(crate) fn output_dir(self) -> OutputDir {
        self.page.path().to_output_dir()
    }

    pub(crate) fn path(self) -> Href {
        stayhydated_site::routing::href(&BaseHref::root(), &self.page.path())
    }
}

pub(crate) fn all_routes() -> Vec<SiteRoute> {
    PageKind::all().into_iter().map(SiteRoute::new).collect()
}

#[derive(Clone, Debug, Eq, PartialEq, Routable)]
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

pub(crate) fn app_base_href() -> BaseHref {
    let base_path = cli_config::base_path();
    let base_path = base_path.as_deref().map(BasePath::new);
    stayhydated_site::routing::base_href(base_path.as_ref())
}

pub(crate) fn page_href(page: PageKind) -> Href {
    stayhydated_site::routing::href(&app_base_href(), &page.path())
}

pub(crate) fn book_href() -> Href {
    stayhydated_site::routing::href(&app_base_href(), &RoutePath::new("book"))
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
    let generated_top_level_dirs = all_routes()
        .into_iter()
        .filter_map(|route| {
            route
                .output_dir()
                .as_str()
                .split('/')
                .next()
                .filter(|segment| !segment.is_empty())
                .map(str::to_string)
        })
        .collect::<Vec<_>>();

    stayhydated_site::route_cache::cleanup_generated_route_cache(
        public_dir,
        GENERATED_ROUTE_CACHE_MARKER,
        generated_top_level_dirs,
        |_, _| false,
    )
}

#[cfg(test)]
pub(crate) fn site_route_from_path(path: &str) -> SiteRoute {
    site_route_from_path_with_base_path(path, None)
}

#[cfg(test)]
pub(crate) fn site_route_from_path_with_base_path(
    path: &str,
    base_path: Option<&str>,
) -> SiteRoute {
    let segments = normalized_path_segments(path, base_path);
    SiteRoute::new(page_from_segments(&segments))
}

#[cfg(test)]
fn normalized_path_segments<'a>(path: &'a str, base_path: Option<&str>) -> Vec<&'a str> {
    let base_path = base_path.map(BasePath::new);
    stayhydated_site::routing::normalized_path_segments(path, base_path.as_ref())
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

fn route_element(route: SiteRoute) -> Element {
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
                {pages::route_content(route)}
            };
        },
    };

    let title = route.page.title_i18n(&i18n);
    let description = route.page.description_i18n(&i18n);

    rsx! {
        Title { "{title}" }
        Meta {
            name: "description",
            content: description,
        }
        {pages::route_content(route)}
    }
}

#[server(endpoint = "static_routes")]
async fn static_routes() -> Result<Vec<String>, ServerFnError> {
    Ok(all_routes()
        .into_iter()
        .map(|route| page_href(route.page))
        .map(Href::into_string)
        .collect())
}

#[component]
fn HomeRoute() -> Element {
    route_element(SiteRoute::new(PageKind::Home))
}

#[component]
fn DemosRoute() -> Element {
    route_element(SiteRoute::new(PageKind::Demos))
}

#[component]
fn CollectionDioxusRoute() -> Element {
    route_element(SiteRoute::new(PageKind::CollectionDioxus))
}

#[component]
fn SalesFormRoute() -> Element {
    route_element(SiteRoute::new(PageKind::SalesForm))
}
