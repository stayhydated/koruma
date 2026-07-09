use crate::components::{FooterPanel, PageHeader};
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use stayhydated_dioxus::{
    DemoCard, DemoCardGrid, GridColumns, ProjectPageShell, page_entry_reveal_style,
};

#[component]
pub(crate) fn DemosPage() -> Element {
    let demos_style = page_entry_reveal_style();

    rsx! {
        ProjectPageShell {
            header: rsx!(PageHeader { current_page: PageKind::Demos }),
            footer: Some(rsx!(FooterPanel {})),
            DemoCardGrid::<crate::site::routing::AppRoute> {
                cards: vec![
                    DemoCard::route(
                        crate::site::routing::app_route(PageKind::CollectionDioxus),
                        "web",
                        "koruma-collection - web",
                        "Interactive Dioxus UI showcasing validator behavior and localized error messages.",
                        "Open Dioxus demo",
                    ),
                    DemoCard::route(
                        crate::site::routing::app_route(PageKind::SalesForm),
                        "form",
                        "Sales intake form",
                        "A live sales intake form that validates a realistic lead payload with typed koruma errors.",
                        "Open sales form",
                    ),
                ],
                columns: GridColumns::Two,
                extra_class: "motion-reveal",
                style: demos_style,
            }
        }
    }
}
