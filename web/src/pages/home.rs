use crate::components::{FooterPanel, PageHeader};
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use stayhydated_dioxus::{
    FeatureCard, HeroListPanel, HeroPanelItem, HeroPanelListKind, LinkTarget, Project, ProjectHero,
    ProjectHeroActions, ProjectHomeShell, ProjectSurfaceSection, feature_card_reveal_style,
    hero_reveal_style,
};

#[component]
pub(crate) fn HomePage() -> Element {
    let hero_style = hero_reveal_style();

    rsx! {
        ProjectHomeShell {
            header: rsx!(PageHeader { current_page: PageKind::Home }),
            footer: rsx!(FooterPanel {}),
            ProjectHero {
                eyebrow: "per-field validation",
                title: "koruma",
                body: "Type-safe Rust validation built around explicit validator types, derive macros, generated error accessors, and optional Fluent messages.",
                style: hero_style,
                side: Some(rsx! {
                    HeroListPanel {
                        label: "Validation flow",
                        kind: HeroPanelListKind::Ordered,
                        items: vec![
                            HeroPanelItem::new("Define", "validators as ordinary Rust structs."),
                            HeroPanelItem::new("Attach", "validators to fields with #[koruma(...)]."),
                            HeroPanelItem::new("Inspect", "typed accessors on generated error structs."),
                        ],
                    }
                }),
                actions: Some(rsx! {
                    ProjectHeroActions::<crate::site::routing::AppRoute> {
                        book: crate::site::routing::book_href().as_str(),
                        docs: Project::Koruma.rustdoc_href(),
                        demos: LinkTarget::route(crate::site::routing::app_route(PageKind::Demos)),
                        book_label: "Read the book",
                        docs_label: "Read the docs",
                        demos_label: "View demos",
                    }
                }),
            }

            ProjectSurfaceSection {
                label: "Core surfaces",
                title: "A small API with typed failure data",
                FeatureCard {
                    label: "derive",
                    title: "Generated accessors",
                    body: "Koruma derives field-level error types so application code can match the exact validator that failed.",
                    style: feature_card_reveal_style(0),
                }
                FeatureCard {
                    label: "validators",
                    title: "Reusable structs",
                    body: "Validator definitions stay explicit, testable, and shareable across fields and data models.",
                    style: feature_card_reveal_style(1),
                }
                FeatureCard {
                    label: "fluent",
                    title: "Fluent-ready output",
                    body: "Display messages and Project Fluent messages can be generated from the same validation model.",
                    style: feature_card_reveal_style(2),
                }
            }
        }
    }
}
