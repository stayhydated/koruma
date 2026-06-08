use crate::components::{ContributePanel, FeatureCard, FooterPanel, PageHeader, PageLink};
use crate::site::i18n::HomeMessage;
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use stayhydated_dioxus::{DisplayText, SkillInstallCommand};

#[component]
pub(crate) fn HomePage() -> Element {
    let hero_style = crate::components::use_reveal_style(0, 24.0);
    let surface_style = crate::components::use_reveal_style(90, 18.0);
    let first_card_style = crate::components::use_reveal_style(160, 16.0);
    let second_card_style = crate::components::use_reveal_style(230, 16.0);
    let third_card_style = crate::components::use_reveal_style(300, 16.0);
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                div { class: "page-shell", "Failed to initialize i18n: {error}" }
            };
        },
    };

    let (
        eyebrow,
        title,
        body,
        primary_action,
        secondary_action,
        workflow_panel_label,
        step_define_title,
        step_define_body,
        step_attach_title,
        step_attach_body,
        step_inspect_title,
        step_inspect_body,
        surface_panel_label,
        surface_title,
        surface_card_one_title,
        surface_card_one_body,
        surface_card_two_title,
        surface_card_two_body,
        surface_card_three_title,
        surface_card_three_body,
    ) = (
        i18n.localize_message(&HomeMessage::HeroEyebrow),
        i18n.localize_message(&HomeMessage::HeroTitle),
        i18n.localize_message(&HomeMessage::HeroBody),
        i18n.localize_message(&HomeMessage::HeroPrimaryAction),
        i18n.localize_message(&HomeMessage::HeroSecondaryAction),
        i18n.localize_message(&HomeMessage::WorkflowPanelLabel),
        i18n.localize_message(&HomeMessage::WorkflowStepDefineTitle),
        i18n.localize_message(&HomeMessage::WorkflowStepDefineBody),
        i18n.localize_message(&HomeMessage::WorkflowStepAttachTitle),
        i18n.localize_message(&HomeMessage::WorkflowStepAttachBody),
        i18n.localize_message(&HomeMessage::WorkflowStepInspectTitle),
        i18n.localize_message(&HomeMessage::WorkflowStepInspectBody),
        i18n.localize_message(&HomeMessage::SurfacePanelLabel),
        i18n.localize_message(&HomeMessage::SurfaceTitle),
        i18n.localize_message(&HomeMessage::SurfaceDescribeTitle),
        i18n.localize_message(&HomeMessage::SurfaceDescribeBody),
        i18n.localize_message(&HomeMessage::SurfaceReusableTitle),
        i18n.localize_message(&HomeMessage::SurfaceReusableBody),
        i18n.localize_message(&HomeMessage::SurfaceI18nTitle),
        i18n.localize_message(&HomeMessage::SurfaceI18nBody),
    );

    rsx! {
        div { class: "page-shell",
            PageHeader { current_page: PageKind::Home }
            main { class: "stack",
                section { class: "hero motion-reveal",
                    style: hero_style.as_str(),
                    div { class: "hero-copy",
                        div { class: "eyebrow", "{eyebrow}" }
                        h1 { "{title}" }
                        p { "{body}" }
                        div { class: "hero-actions",
                            a {
                                class: "button-link primary",
                                href: crate::site::routing::book_href().as_str(),
                                "{primary_action}"
                            }
                            PageLink {
                                page: PageKind::Demos,
                                class: "button-link secondary".to_string(),
                                label: secondary_action,
                            }
                        }
                    }
                    aside { class: "workflow-panel",
                        span { class: "panel-label", "{workflow_panel_label}" }
                        ol { class: "workflow-list",
                            li {
                                strong { "{step_define_title}" }
                                span { "{step_define_body}" }
                            }
                            li {
                                strong { "{step_attach_title}" }
                                span { "{step_attach_body}" }
                            }
                            li {
                                strong { "{step_inspect_title}" }
                                span { "{step_inspect_body}" }
                            }
                        }
                    }
                }

                section { class: "section-band motion-reveal",
                    style: surface_style.as_str(),
                    div { class: "surface-panel-header",
                        div { class: "section-heading surface-panel-heading",
                            span { class: "panel-label", "{surface_panel_label}" }
                            h2 { "{surface_title}" }
                        }
                        div { class: "surface-install-command",
                            SkillInstallCommand { repo: "koruma" }
                        }
                    }
                    div { class: "feature-grid",
                        FeatureCard {
                            label: DisplayText::new("derive"),
                            title: DisplayText::new(surface_card_one_title),
                            body: DisplayText::new(surface_card_one_body),
                            style: first_card_style,
                        }
                        FeatureCard {
                            label: DisplayText::new("validators"),
                            title: DisplayText::new(surface_card_two_title),
                            body: DisplayText::new(surface_card_two_body),
                            style: second_card_style,
                        }
                        FeatureCard {
                            label: DisplayText::new("i18n"),
                            title: DisplayText::new(surface_card_three_title),
                            body: DisplayText::new(surface_card_three_body),
                            style: third_card_style,
                        }
                    }
                }

                ContributePanel {}
            }
            FooterPanel {}
        }
    }
}
