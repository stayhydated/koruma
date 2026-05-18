use crate::components::{ContributePanel, DemoCardLink, FooterPanel, PageHeader};
use crate::site::i18n::DemosPageMessage;
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;

#[component]
pub(crate) fn DemosPage() -> Element {
    let (dioxus_label, dioxus_title, dioxus_body, dioxus_action) = match use_i18n() {
        Ok(i18n) => (
            i18n.localize_message(&DemosPageMessage::DioxusLabel),
            i18n.localize_message(&DemosPageMessage::DioxusTitle),
            i18n.localize_message(&DemosPageMessage::DioxusBody),
            i18n.localize_message(&DemosPageMessage::DioxusAction),
        ),
        Err(_) => (
            "web".to_string(),
            "koruma-collection - web".to_string(),
            "Interactive Dioxus UI showcasing validator behavior and localized error messages."
                .to_string(),
            "Open Dioxus demo".to_string(),
        ),
    };

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
