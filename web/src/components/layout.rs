use crate::site::routing::PageKind;
use dioxus::prelude::*;
use stayhydated_dioxus::{
    ContributePanelShell, LinkTarget, Project, ProjectFluentTextLink, ProjectFooterPanelForProject,
    ProjectPackage, ProjectPackageTextLink, ProjectSourceTextLink, ProjectSupportLink,
    ProjectSupportTextLink, StayhydatedProjectHeader, StayhydatedProjectHeaderConfig,
    contribute_reveal_style, stayhydated_header_labels,
};

#[component]
pub(crate) fn PageHeader(current_page: PageKind) -> Element {
    let config = StayhydatedProjectHeaderConfig::new(
        Project::Koruma,
        crate::site::routing::page_href(PageKind::Home).into_string(),
        LinkTarget::route(crate::site::routing::app_route(PageKind::Home)),
        LinkTarget::route(crate::site::routing::app_route(PageKind::Demos)),
        crate::site::routing::book_href().as_str(),
        stayhydated_header_labels(),
        current_page.project_nav_item(),
    );

    rsx! {
        StayhydatedProjectHeader::<crate::site::routing::AppRoute> {
            config,
        }
    }
}

#[component]
pub(crate) fn ContributePanel() -> Element {
    let reveal_style = contribute_reveal_style();

    rsx! {
        ContributePanelShell { style: reveal_style,
            span { class: "panel-label", "Help translate" }
            h2 {
                "Improve "
                ProjectPackageTextLink { package: ProjectPackage::KORUMA_COLLECTION }
            }
            p {
                ProjectPackageTextLink { package: ProjectPackage::KORUMA_COLLECTION }
                " ships "
                ProjectFluentTextLink {
                    label: "Project Fluent",
                }
                " messages. Add missing translations through "
                ProjectSupportTextLink {
                    link: ProjectSupportLink::KORUMA_COLLECTION_CROWDIN,
                    label: "Crowdin",
                }
                " or contribute directly on "
                ProjectSourceTextLink {
                    project: Project::Koruma,
                    label: "GitHub",
                }
                "."
            }
        }
    }
}

#[component]
pub(crate) fn FooterPanel() -> Element {
    rsx! {
        ProjectFooterPanelForProject {
            project: Project::Koruma,
        }
    }
}
