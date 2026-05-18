use crate::components::{ContributePanel, DemoCardLink, FooterPanel, PageHeader};
use crate::site::i18n::DemosPageMessage;
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;

#[component]
pub(crate) fn DemosPage() -> Element {
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

    let (dioxus_label, dioxus_title, dioxus_body, dioxus_action) = (
        i18n.localize_message(&DemosPageMessage::DioxusLabel),
        i18n.localize_message(&DemosPageMessage::DioxusTitle),
        i18n.localize_message(&DemosPageMessage::DioxusBody),
        i18n.localize_message(&DemosPageMessage::DioxusAction),
    );

    rsx! {
            div { class: "page-shell",
            PageHeader { current_page: PageKind::Demos }
            main { class: "stack",
                ContributePanel {}
                section { class: "grid",
                    DemoCardLink {
                        page: PageKind::CollectionDioxus,
                        label: dioxus_label,
                        title: dioxus_title,
                        body: dioxus_body,
                        action: dioxus_action,
                    }
                }
            }
            FooterPanel {}
        }
    }
}
