use dioxus::prelude::*;
use stayhydated_dioxus::{
    DemoGallery, DemoGalleryItem, NavigationTarget, StayhydatedProjectPortalShell,
};

use crate::site::{
    constants::{PROJECT, VERSION},
    routing::{AppRoute, PageKind},
};

#[component]
pub(crate) fn DemosPage() -> Element {
    let demos = vec![
        DemoGalleryItem::route(
            crate::site::routing::app_route(PageKind::CollectionDioxus),
            "koruma-collection",
            "collection-demo-card-shader",
        ),
        DemoGalleryItem::route(
            crate::site::routing::app_route(PageKind::SalesForm),
            "Sales form",
            "sales-form-demo-card-shader",
        ),
    ];

    rsx! {
        StayhydatedProjectPortalShell {
            project: PROJECT,
            version: VERSION,
            home: NavigationTarget::Internal(crate::site::routing::app_route(PageKind::Home)),
            DemoGallery::<AppRoute> { items: demos }
        }
    }
}
