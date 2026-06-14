use crate::components::{ContributePanel, FooterPanel, PageHeader};
use crate::site::i18n::HomeMessage;
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use stayhydated_dioxus::{
    FeatureCardItem, HeroListPanel, HeroPanelItem, HeroPanelListKind, LinkTarget, Project,
    ProjectHero, ProjectHeroActions, ProjectHomeShell, SkillFeatureSection, hero_reveal_style,
};

#[component]
pub(crate) fn HomePage() -> Element {
    let hero_style = hero_reveal_style();
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
        ProjectHomeShell {
            header: rsx!(PageHeader { current_page: PageKind::Home }),
            footer: rsx!(FooterPanel {}),
            ProjectHero {
                eyebrow,
                title,
                body,
                style: hero_style,
                side: Some(rsx! {
                    HeroListPanel {
                        label: workflow_panel_label,
                        kind: HeroPanelListKind::Ordered,
                        items: vec![
                            HeroPanelItem::new(step_define_title, step_define_body),
                            HeroPanelItem::new(step_attach_title, step_attach_body),
                            HeroPanelItem::new(step_inspect_title, step_inspect_body),
                        ],
                    }
                }),
                actions: Some(rsx! {
                    ProjectHeroActions::<crate::site::routing::AppRoute> {
                        book: crate::site::routing::book_href().as_str(),
                        demos: LinkTarget::route(crate::site::routing::app_route(PageKind::Demos)),
                        primary_label: primary_action,
                        secondary_label: secondary_action,
                    }
                }),
            }

            SkillFeatureSection {
                label: surface_panel_label,
                title: surface_title,
                repo: Project::Koruma,
                items: vec![
                    FeatureCardItem::new("derive", surface_card_one_title, surface_card_one_body),
                    FeatureCardItem::new("validators", surface_card_two_title, surface_card_two_body),
                    FeatureCardItem::new("i18n", surface_card_three_title, surface_card_three_body),
                ],
            }

            ContributePanel {}
        }
    }
}
