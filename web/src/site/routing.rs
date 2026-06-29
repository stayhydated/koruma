use crate::pages;
use crate::site::i18n::{PageMetadataMessage, SiteLanguage};
use dioxus::cli_config;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::{DioxusAssetI18nHandle, use_i18n};
use stayhydated_dioxus::{
    LocalizedRouteSegment, Project, ProjectNavItem, StayhydatedProjectPageMetadata,
    StayhydatedSiteLanguage as _,
};
use stayhydated_site::routing::{BaseHref, BasePath, Href, RoutePath};

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

    pub(crate) const fn project_nav_item(self) -> ProjectNavItem {
        match self {
            Self::Home => ProjectNavItem::Home,
            Self::Demos | Self::CollectionDioxus | Self::SalesForm => ProjectNavItem::Demos,
        }
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
    pub(crate) locale: SiteLanguage,
    pub(crate) page: PageKind,
}

impl SiteRoute {
    pub(crate) const fn new(locale: SiteLanguage, page: PageKind) -> Self {
        Self { locale, page }
    }

    pub(crate) fn path(self) -> Href {
        stayhydated_site::routing::href(&BaseHref::root(), &relative_path(self.locale, self.page))
    }
}

pub(crate) fn all_routes() -> Vec<SiteRoute> {
    let mut routes = Vec::new();

    for locale in SiteLanguage::all_languages() {
        for page in PageKind::all() {
            routes.push(SiteRoute::new(locale, page));
        }
    }

    routes
}

pub(crate) type LocaleSegment = LocalizedRouteSegment<SiteLanguage>;

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
    #[route("/:locale/", LocalizedHomeRoute)]
    LocalizedHome { locale: LocaleSegment },
    #[route("/:locale/demos/", LocalizedDemosRoute)]
    LocalizedDemos { locale: LocaleSegment },
    #[route("/:locale/demos/koruma-collection/", LocalizedCollectionDioxusRoute)]
    LocalizedCollectionDioxus { locale: LocaleSegment },
    #[route("/:locale/demos/sales-form/", LocalizedSalesFormRoute)]
    LocalizedSalesForm { locale: LocaleSegment },
}

pub(crate) fn app_route(locale: SiteLanguage, page: PageKind) -> AppRoute {
    match (locale.route_slug(), page) {
        (None, PageKind::Home) => AppRoute::Home {},
        (None, PageKind::Demos) => AppRoute::Demos {},
        (None, PageKind::CollectionDioxus) => AppRoute::CollectionDioxus {},
        (None, PageKind::SalesForm) => AppRoute::SalesForm {},
        (Some(_), PageKind::Home) => AppRoute::LocalizedHome {
            locale: LocaleSegment::new(locale),
        },
        (Some(_), PageKind::Demos) => AppRoute::LocalizedDemos {
            locale: LocaleSegment::new(locale),
        },
        (Some(_), PageKind::CollectionDioxus) => AppRoute::LocalizedCollectionDioxus {
            locale: LocaleSegment::new(locale),
        },
        (Some(_), PageKind::SalesForm) => AppRoute::LocalizedSalesForm {
            locale: LocaleSegment::new(locale),
        },
    }
}

pub(crate) fn app_base_href() -> BaseHref {
    let base_path = cli_config::base_path();
    let base_path = base_path.as_deref().map(BasePath::new);
    stayhydated_site::routing::base_href(base_path.as_ref())
}

pub(crate) fn page_href(locale: SiteLanguage, page: PageKind) -> Href {
    stayhydated_site::routing::href(&app_base_href(), &relative_path(locale, page))
}

pub(crate) fn book_href() -> Href {
    stayhydated_site::routing::href(&app_base_href(), &RoutePath::new("book"))
}

fn relative_path(locale: SiteLanguage, page: PageKind) -> RoutePath {
    let mut segments = Vec::new();

    if let Some(slug) = locale.route_slug() {
        segments.push(slug);
    }

    let page_segment = page.route();
    if !page_segment.is_empty() {
        segments.push(page_segment.to_string());
    }

    RoutePath::new(segments.join("/"))
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

    let route_language = route.locale.language_identifier();
    let i18n_result = if i18n.peek_requested_language() == route_language {
        Ok(i18n)
    } else {
        i18n.select_language(route_language)
            .map(|()| i18n)
            .map_err(|error| {
                format!(
                    "failed to select localized route '{}': {error}",
                    route.locale.html_lang()
                )
            })
    };

    match i18n_result {
        Ok(i18n) => {
            let _ = i18n.requested_language();
            let page_title = route.page.title_i18n(&i18n);
            let description = route.page.description_i18n(&i18n);

            rsx! {
                StayhydatedProjectPageMetadata {
                    project: Project::Koruma,
                    page_title,
                    description,
                }
                {pages::route_content(route)}
            }
        },
        Err(error) => rsx! {
            Title { "koruma" }
            Meta {
                name: "description",
                content: "Failed to initialize i18n",
            }
            div { class: "page-shell", "Failed to initialize i18n: {error}" }
            {pages::route_content(route)}
        },
    }
}

#[component]
fn HomeRoute() -> Element {
    route_element(SiteRoute::new(SiteLanguage::default(), PageKind::Home))
}

#[component]
fn DemosRoute() -> Element {
    route_element(SiteRoute::new(SiteLanguage::default(), PageKind::Demos))
}

#[component]
fn CollectionDioxusRoute() -> Element {
    route_element(SiteRoute::new(
        SiteLanguage::default(),
        PageKind::CollectionDioxus,
    ))
}

#[component]
fn SalesFormRoute() -> Element {
    route_element(SiteRoute::new(SiteLanguage::default(), PageKind::SalesForm))
}

#[component]
fn LocalizedHomeRoute(locale: LocaleSegment) -> Element {
    route_element(SiteRoute::new(locale.language(), PageKind::Home))
}

#[component]
fn LocalizedDemosRoute(locale: LocaleSegment) -> Element {
    route_element(SiteRoute::new(locale.language(), PageKind::Demos))
}

#[component]
fn LocalizedCollectionDioxusRoute(locale: LocaleSegment) -> Element {
    route_element(SiteRoute::new(
        locale.language(),
        PageKind::CollectionDioxus,
    ))
}

#[component]
fn LocalizedSalesFormRoute(locale: LocaleSegment) -> Element {
    route_element(SiteRoute::new(locale.language(), PageKind::SalesForm))
}
