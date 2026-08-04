use stayhydated_dioxus::{Project, ProjectSite};

pub(crate) const PROJECT: Project = Project::new("koruma", "Rust validation")
    .with_skill_command("npx skills add stayhydated/koruma");
pub(crate) const SITE_URL: &str = "https://stayhydated.github.io/koruma/";
pub(crate) const RUSTDOC_URL: &str = "https://docs.rs/koruma/";
pub(crate) const SOURCE_URL: &str = "https://github.com/stayhydated/koruma";
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn site() -> ProjectSite {
    ProjectSite::builder()
        .project(PROJECT)
        .site_url(SITE_URL)
        .rustdoc_url(RUSTDOC_URL)
        .source_url(SOURCE_URL)
        .version(VERSION)
        .site_stylesheet_path("assets/site.css")
        .build()
}
