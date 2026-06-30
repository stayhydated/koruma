use crate::components::{ContributePanel, FooterPanel, PageHeader};
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use stayhydated_dioxus::{
    FeatureCardItem, HeroListPanel, HeroPanelItem, HeroPanelListKind, LinkTarget, Project,
    ProjectHero, ProjectHeroActions, ProjectHomeShell, SkillFeatureSection, hero_reveal_style,
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
                        demos: LinkTarget::route(crate::site::routing::app_route(PageKind::Demos)),
                        primary_label: "Read the book",
                        secondary_label: "Open demos",
                    }
                }),
            }

            SkillFeatureSection {
                label: "Core surfaces",
                title: "A small API with typed failure data",
                repo: Project::Koruma,
                items: vec![
                    FeatureCardItem::new(
                        "derive",
                        "Generated accessors",
                        "Koruma derives field-level error types so application code can match the exact validator that failed.",
                    ),
                    FeatureCardItem::new(
                        "validators",
                        "Reusable structs",
                        "Validator definitions stay explicit, testable, and shareable across fields and data models.",
                    ),
                    FeatureCardItem::new(
                        "fluent",
                        "Fluent-ready output",
                        "Display messages and Project Fluent messages can be generated from the same validation model.",
                    ),
                ],
            }

            ContributePanel {}
        }
    }
}
