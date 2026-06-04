use crate::components::{LanguageSelect, ProjectSelect, stayhydated_project_options};
use crate::site::constants::{
    KORUMA_COLLECTION_CRATES_URL, KORUMA_COLLECTION_CROWDIN_URL, KORUMA_CRATES_URL,
    PROJECT_FLUENT_URL,
};
use crate::site::i18n::{ContributeMessage, SiteChromeMessage, SiteFooterMessage, SiteLanguage};
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use stayhydated_dioxus::{DisplayText, Href, ProjectId};

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

    let nav_home = i18n.localize_message(&SiteChromeMessage::NavHome);
    let nav_demos = i18n.localize_message(&SiteChromeMessage::NavDemos);
    let nav_docs = i18n.localize_message(&SiteChromeMessage::NavDocs);
    let nav_source = i18n.localize_message(&SiteChromeMessage::NavSource);

    let is_home_active = current_page == PageKind::Home;
    let is_demos_active = matches!(
        current_page,
        PageKind::Demos | PageKind::CollectionDioxus | PageKind::SalesForm
    );

    rsx! {
        header { class: "page-header",
            ProjectSelect {
                selected: ProjectId::Koruma
                    .option_with_href(Href::new(crate::site::routing::page_href(PageKind::Home).into_string())),
                projects: stayhydated_project_options(),
                label: DisplayText::new("Project selector"),
            }
            div { class: "header-cluster",
                nav { class: "header-nav-links", "aria-label": "Primary navigation",
                    crate::components::PageLink {
                        page: PageKind::Home,
                        class: if is_home_active {
                            "header-nav-item is-active".to_string()
                        } else {
                            "header-nav-item".to_string()
                        },
                        label: nav_home,
                    }
                    crate::components::PageLink {
                        page: PageKind::Demos,
                        class: if is_demos_active {
                            "header-nav-item is-active".to_string()
                        } else {
                            "header-nav-item".to_string()
                        },
                        label: nav_demos,
                    }
                    a {
                        class: "header-nav-item",
                        href: crate::site::routing::book_href().as_str(),
                        "{nav_docs}"
                    }
                    a {
                        class: "header-nav-item",
                        href: ProjectId::Koruma.source_href(),
                        target: "_blank",
                        rel: "noreferrer",
                        "{nav_source}"
                    }
                }
                LocaleSwitcher {}
            }
        }
    }
}

#[component]
fn LocaleSwitcher() -> Element {
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                div { class: "locale-switcher-dropdown", "failed to initialize locale switcher: {error}" }
            };
        },
    };

    let locale_label = i18n.localize_message(&SiteChromeMessage::LocaleLabel);
    let language_options = SiteLanguage::all()
        .map(|language| {
            let label = i18n.localize_message(&language);
            (language, label)
        })
        .collect::<Vec<_>>();

    let requested_language = i18n.requested_language();
    let current_language = SiteLanguage::all()
        .find(|language| language.lang() == requested_language)
        .unwrap_or_default();
    let on_locale_changed = move |next_language: SiteLanguage| {
        let _ = i18n.select_language(next_language.lang());
    };

    rsx! {
        LanguageSelect::<SiteLanguage> {
            label: DisplayText::new(locale_label),
            selected: current_language,
            options: language_options,
            on_change: on_locale_changed,
        }
    }
}

#[component]
pub(crate) fn ContributePanel() -> Element {
    let reveal_style = crate::components::use_reveal_style(370, 16.0);
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
        section { class: "contribute-panel motion-reveal",
            style: reveal_style.as_str(),
            div { class: "contribute-copy",
                span { class: "panel-label", "{label}" }
                h2 {
                    "{headline} "
                    a {
                        href: KORUMA_COLLECTION_CRATES_URL,
                        target: "_blank",
                        rel: "noreferrer",
                        "koruma-collection"
                    }
                }
                p {
                    a {
                        href: KORUMA_COLLECTION_CRATES_URL,
                        target: "_blank",
                        rel: "noreferrer",
                        "koruma-collection"
                    }
                    " "
                    "{body_prefix}"
                    " "
                    a {
                        href: PROJECT_FLUENT_URL,
                        target: "_blank",
                        rel: "noreferrer",
                        "Project Fluent"
                    }
                    " "
                    "{project_fluent}"
                    " "
                    a {
                        href: KORUMA_COLLECTION_CROWDIN_URL,
                        target: "_blank",
                        rel: "noreferrer",
                        "{body_crowdin}"
                    }
                    " "
                    "{body_github}"
                    " "
                    a {
                        href: ProjectId::Koruma.source_href(),
                        target: "_blank",
                        rel: "noreferrer",
                        "GitHub"
                    }
                    "{dot}"
                }
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
    let crates_text_prefix = i18n.localize_message(&SiteFooterMessage::CratesTextPrefix);
    let crates_text_middle = i18n.localize_message(&SiteFooterMessage::CratesTextMiddle);
    let crates_text_suffix = i18n.localize_message(&SiteFooterMessage::CratesTextSuffix);
    let crates_text_middle = crates_text_middle.trim().to_string();
    let crates_text_suffix = crates_text_suffix.trim().to_string();

    rsx! {
        footer { class: "site-footer",
            p {
                span { class: "footer-label", "{label}" }
                span { class: "footer-text",
                    "{crates_text_prefix}"
                    a {
                        href: KORUMA_CRATES_URL,
                        target: "_blank",
                        rel: "noreferrer",
                        "koruma"
                    }
                    " "
                    "{crates_text_middle}"
                    " "
                    a {
                        href: KORUMA_COLLECTION_CRATES_URL,
                        target: "_blank",
                        rel: "noreferrer",
                        "koruma-collection"
                    }
                    " "
                    "{crates_text_suffix}"
                }
            }
        }
    }
}
