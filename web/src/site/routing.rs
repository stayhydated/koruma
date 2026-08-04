use crate::pages;
use dioxus::prelude::*;
use stayhydated_dioxus::StayhydatedProjectPageMetadata;

use crate::site::constants::PROJECT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PageKind {
    Home,
    Demos,
    CollectionDioxus,
    SalesForm,
}

impl PageKind {
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

fn route_element(page: PageKind) -> Element {
    rsx! {
        StayhydatedProjectPageMetadata {
            project: PROJECT,
            page_title: page.title(),
            description: page.description(),
        }
        {pages::route_content(page)}
    }
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
