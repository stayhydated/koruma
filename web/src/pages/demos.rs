use crate::components::{FooterPanel, PageHeader};
use crate::site::i18n::DemosPageMessage;
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use stayhydated_dioxus::{DemoCard, DemoCardGrid, GridColumns, ProjectPageShell};

#[component]
pub(crate) fn DemosPage() -> Element {
    let demos_style = crate::components::use_reveal_style(0, 24.0);
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                div { class: "page-shell",
                    "Failed to initialize i18n: {error}"
                }
            };
        },
    };

    let (
        dioxus_label,
        dioxus_title,
        dioxus_body,
        dioxus_action,
        sales_label,
        sales_title,
        sales_body,
        sales_action,
    ) = (
        i18n.localize_message(&DemosPageMessage::DioxusLabel),
        i18n.localize_message(&DemosPageMessage::DioxusTitle),
        i18n.localize_message(&DemosPageMessage::DioxusBody),
        i18n.localize_message(&DemosPageMessage::DioxusAction),
        i18n.localize_message(&DemosPageMessage::SalesLabel),
        i18n.localize_message(&DemosPageMessage::SalesTitle),
        i18n.localize_message(&DemosPageMessage::SalesBody),
        i18n.localize_message(&DemosPageMessage::SalesAction),
    );

    rsx! {
        ProjectPageShell {
            header: rsx!(PageHeader { current_page: PageKind::Demos }),
            footer: Some(rsx!(FooterPanel {})),
            DemoCardGrid::<crate::site::routing::AppRoute> {
                cards: vec![
                    DemoCard::route(
                        crate::site::routing::app_route(PageKind::CollectionDioxus),
                        dioxus_label,
                        dioxus_title,
                        dioxus_body,
                        dioxus_action,
                    ),
                    DemoCard::route(
                        crate::site::routing::app_route(PageKind::SalesForm),
                        sales_label,
                        sales_title,
                        sales_body,
                        sales_action,
                    ),
                ],
                columns: GridColumns::Two,
                extra_class: "motion-reveal",
                style: demos_style,
            }
        }
    }
}
