mod demo_i18n;
mod demos;
mod home;
pub(crate) mod i18n;
mod sales_form_demo;
mod wasm_demo;

pub(crate) use demo_i18n::DemoLanguageSwitcher;
pub(crate) use demos::DemosPage;
pub(crate) use home::HomePage;
pub(crate) use sales_form_demo::SalesFormPage;
pub(crate) use wasm_demo::CollectionDioxusPage;

use crate::site::routing::PageKind;
use dioxus::prelude::*;

pub(crate) fn route_content(page: PageKind) -> Element {
    match page {
        PageKind::Home => rsx!(HomePage {}),
        PageKind::Demos => rsx!(DemosPage {}),
        PageKind::CollectionDioxus => rsx!(CollectionDioxusPage {}),
        PageKind::SalesForm => rsx!(SalesFormPage {}),
    }
}
