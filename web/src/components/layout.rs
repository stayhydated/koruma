use crate::site::i18n::{ContributeMessage, SiteFooterMessage, SiteLanguage};
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use stayhydated_dioxus::{
    ContributePanelShell, InPlaceLocalizedLanguageSelect, LinkTarget, Project,
    ProjectFluentTextLink, ProjectPackage, ProjectPackageTextLink,
    ProjectPackagesFooterPanelForProject, ProjectSourceTextLink, ProjectSupportLink,
    ProjectSupportTextLink, StayhydatedProjectHeader, StayhydatedProjectHeaderConfig,
    contribute_reveal_style,
};

#[component]
pub(crate) fn PageHeader(current_page: PageKind) -> Element {
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                div { "Failed to initialize i18n: {error}" }
            };
        },
    };

    let config = StayhydatedProjectHeaderConfig::localized_with_i18n(
        Project::Koruma,
        crate::site::routing::page_href(PageKind::Home).into_string(),
        LinkTarget::route(crate::site::routing::app_route(PageKind::Home)),
        LinkTarget::route(crate::site::routing::app_route(PageKind::Demos)),
        crate::site::routing::book_href().as_str(),
        current_page.project_nav_item(),
        &i18n,
    );

    rsx! {
        StayhydatedProjectHeader::<crate::site::routing::AppRoute> {
            config,
            LocaleSwitcher {}
        }
    }
}

#[component]
fn LocaleSwitcher() -> Element {
    rsx! {
        InPlaceLocalizedLanguageSelect::<SiteLanguage> {}
    }
}

#[component]
pub(crate) fn ContributePanel() -> Element {
    let reveal_style = contribute_reveal_style();
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                section { class: "contribute-panel",
                    div { class: "contribute-copy", "Failed to initialize i18n: {error}" }
                }
            };
        },
    };

    let (label, headline, body_prefix, project_fluent, body_crowdin, body_github, dot) = (
        i18n.localize_message(&ContributeMessage::Label),
        i18n.localize_message(&ContributeMessage::Headline),
        i18n.localize_message(&ContributeMessage::BodyPrefix),
        i18n.localize_message(&ContributeMessage::BodyProjectFluent),
        i18n.localize_message(&ContributeMessage::BodyCrowdin),
        i18n.localize_message(&ContributeMessage::BodyGithub),
        i18n.localize_message(&ContributeMessage::FooterDot),
    );
    let body_prefix = body_prefix.trim().to_string();
    let project_fluent = project_fluent.trim().to_string();
    let body_github = body_github.trim().to_string();

    rsx! {
        ContributePanelShell { style: reveal_style,
            span { class: "panel-label", "{label}" }
            h2 {
                "{headline} "
                ProjectPackageTextLink { package: ProjectPackage::KORUMA_COLLECTION }
            }
            p {
                ProjectPackageTextLink { package: ProjectPackage::KORUMA_COLLECTION }
                " "
                "{body_prefix}"
                " "
                ProjectFluentTextLink {
                    label: "Project Fluent",
                }
                " "
                "{project_fluent}"
                " "
                ProjectSupportTextLink {
                    link: ProjectSupportLink::KORUMA_COLLECTION_CROWDIN,
                    label: body_crowdin,
                }
                " "
                "{body_github}"
                " "
                ProjectSourceTextLink {
                    project: Project::Koruma,
                    label: "GitHub",
                }
                "{dot}"
            }
        }
    }
}

#[component]
pub(crate) fn FooterPanel() -> Element {
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                footer { class: "site-footer", "Failed to initialize i18n: {error}" }
            };
        },
    };

    let label = i18n.localize_message(&SiteFooterMessage::CratesLabel);
    let prefix = i18n.localize_message(&SiteFooterMessage::CratesTextPrefix);
    let separator = i18n
        .localize_message(&SiteFooterMessage::CratesTextMiddle)
        .trim()
        .to_string();
    let suffix = i18n
        .localize_message(&SiteFooterMessage::CratesTextSuffix)
        .trim()
        .to_string();
    let separator = format!(" {separator} ");
    let suffix = format!(" {suffix}");
    rsx! {
        ProjectPackagesFooterPanelForProject {
            project: Project::Koruma,
            label,
            prefix,
            separator,
            suffix,
        }
    }
}
