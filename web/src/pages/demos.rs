use crate::components::{FooterPanel, PageCardLink, PageHeader};
use crate::site::i18n::DemosPageMessage;
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_asset_i18n;

#[component]
pub(crate) fn DemosPage() -> Element {
    let demos_style = crate::components::use_reveal_style(0, 24.0);
    let i18n = match use_asset_i18n() {
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
            div { class: "page-shell",
            PageHeader { current_page: PageKind::Demos }
            main { class: "stack",
                section { class: "grid columns-2 motion-reveal",
                    style: demos_style.as_str(),
                    PageCardLink {
                        page: PageKind::CollectionDioxus,
                        label: dioxus_label,
                        title: dioxus_title,
                        body: dioxus_body,
                        action: dioxus_action,
                    }
                    PageCardLink {
                        page: PageKind::SalesForm,
                        label: sales_label,
                        title: sales_title,
                        body: sales_body,
                        action: sales_action,
                    }
                }
            }
            FooterPanel {}
        }
    }
}
