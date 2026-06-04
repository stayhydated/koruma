mod demos;
mod home;
mod sales_form_demo;
mod wasm_demo;

pub(crate) use demos::DemosPage;
pub(crate) use home::HomePage;
pub(crate) use sales_form_demo::SalesFormPage;
pub(crate) use wasm_demo::CollectionDioxusPage;

use crate::site::routing::{PageKind, SiteRoute};
use dioxus::prelude::*;

pub(crate) fn route_content(route: SiteRoute) -> Element {
    match route.page {
        PageKind::Home => rsx!(HomePage {}),
        PageKind::Demos => rsx!(DemosPage {}),
        PageKind::CollectionDioxus => rsx!(CollectionDioxusPage {}),
        PageKind::SalesForm => rsx!(SalesFormPage {}),
    }
}
