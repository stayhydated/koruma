use crate::pages;
use crate::site::i18n::PageMetadataMessage;
use dioxus::cli_config;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::{DioxusI18n, use_i18n};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PageKind {
    Home,
    Demos,
    CollectionDioxus,
}

impl PageKind {
    pub(crate) fn all() -> [Self; 3] {
        [Self::Home, Self::Demos, Self::CollectionDioxus]
    }

    pub(crate) fn route(self) -> &'static str {
        match self {
            Self::Home => "",
            Self::Demos => "demos",
            Self::CollectionDioxus => "collection-dioxus-web",
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

    pub(crate) fn title_i18n(self, i18n: &DioxusI18n) -> String {
        match self {
            Self::Home => i18n.localize_message(&PageMetadataMessage::HomeTitle),
            Self::Demos => i18n.localize_message(&PageMetadataMessage::DemosTitle),
            Self::CollectionDioxus => i18n.localize_message(&PageMetadataMessage::DioxusDemoTitle),
        }
    }

    pub(crate) fn description_i18n(self, i18n: &DioxusI18n) -> String {
        match self {
            Self::Home => i18n.localize_message(&PageMetadataMessage::HomeDescription),
            Self::Demos => i18n.localize_message(&PageMetadataMessage::DemosDescription),
            Self::CollectionDioxus => {
                i18n.localize_message(&PageMetadataMessage::DioxusDemoDescription)
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
    #[route("/collection-dioxus-web/", CollectionDioxusRoute)]
    CollectionDioxus {},
}

pub(crate) fn app_route(page: PageKind) -> AppRoute {
    match page {
        PageKind::Home => AppRoute::Home {},
        PageKind::Demos => AppRoute::Demos {},
        PageKind::CollectionDioxus => AppRoute::CollectionDioxus {},
    }
}

pub(crate) fn app_base_href() -> String {
    match cli_config::base_path() {
        Some(base_path) => {
            let base_path = base_path.trim_matches('/');
            if base_path.is_empty() {
                "/".to_string()
            } else {
                format!("/{base_path}/")
            }
        },
        None => "/".to_string(),
    }
}

pub(crate) fn page_href(page: PageKind) -> String {
    let route = page.route();
    if route.is_empty() {
        app_base_href()
    } else {
        format!("{}{route}/", app_base_href())
    }
}

pub(crate) fn book_href() -> String {
    format!("{}book/", app_base_href())
}

#[cfg(test)]
pub(crate) fn site_route_from_path_with_base_path(path: &str, base_path: Option<&str>) -> PageKind {
    let segments = normalized_path_segments(path, base_path);
    page_from_segments(&segments)
}

#[cfg(test)]
fn normalized_path_segments<'a>(path: &'a str, base_path: Option<&str>) -> Vec<&'a str> {
    let segments = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let base_path_segments = base_path
        .into_iter()
        .flat_map(|base_path| base_path.trim_matches('/').split('/'))
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if base_path_segments.is_empty()
        || !segments
            .as_slice()
            .starts_with(base_path_segments.as_slice())
    {
        segments
    } else {
        segments[base_path_segments.len()..].to_vec()
    }
}

#[cfg(test)]
fn page_from_segments(segments: &[&str]) -> PageKind {
    match segments {
        [] => PageKind::Home,
        ["demos"] => PageKind::Demos,
        ["collection-dioxus-web"] => PageKind::CollectionDioxus,
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
