mod components;
mod pages;
mod site;

pub use site::app::App;
use std::path::Path;

pub fn sitemap_xml() -> String {
    site::render::render_sitemap()
}

pub fn cleanup_generated_route_cache(public_dir: impl AsRef<Path>) -> std::io::Result<()> {
    site::routing::cleanup_generated_route_cache(public_dir.as_ref())
}

pub fn mark_generated_route_cache(public_dir: impl AsRef<Path>) -> std::io::Result<()> {
    site::routing::mark_generated_route_cache(public_dir.as_ref())
}

pub fn route_cache() -> stayhydated_site::RouteCacheHooks {
    stayhydated_site::RouteCacheHooks::new(
        |public_dir| cleanup_generated_route_cache(public_dir),
        |public_dir| mark_generated_route_cache(public_dir),
    )
}

#[cfg(test)]
mod tests {
    use crate::site::i18n::SiteLanguage;
    use crate::site::routing::{
        PageKind, SiteRoute, page_href, site_route_from_path, site_route_from_path_with_base_path,
    };
    use std::fs;

    #[test]
    fn computes_links_without_cli_base_path() {
        assert_eq!(page_href(PageKind::Home), "/");
        assert_eq!(page_href(PageKind::Demos), "/demos/");
        assert_eq!(
            page_href(PageKind::CollectionDioxus),
            "/demos/koruma-collection/"
        );
        assert_eq!(page_href(PageKind::SalesForm), "/demos/sales-form/");
    }

    #[test]
    fn parses_site_routes_with_base_path() {
        assert_eq!(
            site_route_from_path_with_base_path("/koruma/demos/", Some("koruma")),
            SiteRoute::new(PageKind::Demos)
        );
        assert_eq!(
            site_route_from_path_with_base_path("/koruma/demos/koruma-collection/", Some("koruma")),
            SiteRoute::new(PageKind::CollectionDioxus)
        );
        assert_eq!(
            site_route_from_path_with_base_path("/koruma/demos/sales-form/", Some("koruma")),
            SiteRoute::new(PageKind::SalesForm)
        );
        assert_eq!(
            site_route_from_path("/unknown"),
            SiteRoute::new(PageKind::Home)
        );
    }

    #[test]
    fn sitemap_includes_static_book_root() {
        let sitemap = crate::sitemap_xml();
        assert!(sitemap.contains("<loc>https://stayhydated.github.io/koruma/</loc>"));
        assert!(sitemap.contains("<loc>https://stayhydated.github.io/koruma/book/</loc>"));
        assert!(
            sitemap.contains(
                "<loc>https://stayhydated.github.io/koruma/demos/koruma-collection/</loc>"
            )
        );
        assert!(
            sitemap.contains("<loc>https://stayhydated.github.io/koruma/demos/sales-form/</loc>")
        );
    }

    #[test]
    fn i18n_language_switcher_initialization_is_supported() {
        println!("default language: {}", SiteLanguage::default().lang());
        let available = SiteLanguage::all().collect::<Vec<_>>();
        let rendered = available
            .iter()
            .map(|language| language.lang().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!("supported language enums: {rendered}");

        let selected = SiteLanguage::default().lang();
        let selected = selected.clone();
        let init = es_fluent_manager_dioxus::ssr::SsrI18nRuntime::new(
            crate::site::i18n::dioxus_i18n_asset_modules(),
        )
        .request_blocking(selected);
        match init {
            Ok(_) => println!("asset i18n init ok"),
            Err(error) => panic!("asset i18n init failed: {error}"),
        }
    }

    #[test]
    fn cleans_generated_route_cache_without_touching_static_assets() {
        let temp = tempfile::tempdir().expect("tempdir");
        let public_dir = temp.path();

        fs::write(public_dir.join("index.html"), "root").expect("write root index");
        fs::write(public_dir.join("404.html"), "not found").expect("write root 404");
        fs::create_dir_all(public_dir.join("demos")).expect("create demos dir");
        fs::write(public_dir.join("demos").join("index.html"), "stale demos")
            .expect("write demos index");
        fs::create_dir_all(public_dir.join("book")).expect("create book dir");
        fs::write(public_dir.join("book").join("index.html"), "book").expect("write book");
        fs::create_dir_all(public_dir.join("llms")).expect("create llms dir");
        fs::write(public_dir.join("llms").join("getting-started.md"), "llms").expect("write llms");
        fs::create_dir_all(public_dir.join("assets")).expect("create assets dir");
        fs::write(public_dir.join("assets").join("site.css"), "body {}").expect("write asset");

        crate::mark_generated_route_cache(public_dir).expect("mark route cache");
        crate::cleanup_generated_route_cache(public_dir).expect("cleanup route cache");

        assert!(!public_dir.join("index.html").exists());
        assert!(!public_dir.join("404.html").exists());
        assert!(!public_dir.join("demos").exists());
        assert!(public_dir.join("book").join("index.html").exists());
        assert!(public_dir.join("llms").join("getting-started.md").exists());
        assert!(public_dir.join("assets").join("site.css").exists());
    }
}
