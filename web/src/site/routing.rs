use crate::pages;
use dioxus::cli_config;
use dioxus::prelude::*;
use stayhydated_dioxus::StayhydatedProjectPageMetadata;
use stayhydated_site::routing::{BaseHref, BasePath, Href, RoutePath};

use crate::site::constants::PROJECT;

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

    fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Demos => "Demos",
            Self::CollectionDioxus => "Dioxus Collection Example",
            Self::SalesForm => "Sales Form Demo",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Home => {
                "Type-safe per-field validation for Rust with derive macros, validator attributes, typed error accessors, and Fluent-ready rendering."
            },
            Self::Demos => "Interactive koruma-collection validator demos.",
            Self::CollectionDioxus => {
                "A Dioxus koruma-collection validator browser with localized validation output."
            },
            Self::SalesForm => {
                "A sales intake form showing typed koruma validation errors in a realistic workflow."
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

    pub(crate) fn path(self) -> Href {
        stayhydated_site::routing::href(&BaseHref::root(), &relative_path(self.page))
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

pub(crate) fn book_href() -> Href {
    stayhydated_site::routing::href(&app_base_href(), &RoutePath::new("book"))
}

fn relative_path(page: PageKind) -> RoutePath {
    RoutePath::new(page.route())
}

fn route_element(route: SiteRoute) -> Element {
    rsx! {
        StayhydatedProjectPageMetadata {
            project: PROJECT,
            page_title: route.page.title(),
            description: route.page.description(),
        }
        {pages::route_content(route)}
    }
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
