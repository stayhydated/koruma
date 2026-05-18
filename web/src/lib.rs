mod components;
mod pages;
mod site;

pub use site::app::App;

pub fn sitemap_xml() -> String {
    site::render::render_sitemap()
}

#[cfg(test)]
mod tests {
    use crate::site::i18n::SiteLanguage;
    use crate::site::routing::{PageKind, page_href, site_route_from_path_with_base_path};
    use es_fluent_manager_dioxus::ManagedI18n;

    #[test]
    fn computes_links_without_cli_base_path() {
        assert_eq!(page_href(PageKind::Home), "/");
        assert_eq!(page_href(PageKind::Demos), "/demos/");
    }

    #[test]
    fn parses_site_routes_with_base_path() {
        assert_eq!(
            site_route_from_path_with_base_path("/koruma/demos/", Some("koruma")),
            PageKind::Demos
        );
        assert_eq!(
            site_route_from_path_with_base_path("/koruma/collection-dioxus-web/", Some("koruma")),
            PageKind::CollectionDioxus
        );
        assert_eq!(
            site_route_from_path_with_base_path("/unknown", None),
            PageKind::Home
        );
    }

    #[test]
    fn sitemap_includes_static_book_root() {
        let sitemap = crate::sitemap_xml();
        assert!(sitemap.contains("<loc>https://stayhydated.github.io/koruma/</loc>"));
        assert!(sitemap.contains("<loc>https://stayhydated.github.io/koruma/book/</loc>"));
        assert!(
            sitemap
                .contains("<loc>https://stayhydated.github.io/koruma/collection-dioxus-web/</loc>")
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
        let init = ManagedI18n::new_with_discovered_modules(selected);
        match init {
            Ok(_) => println!("managed init ok"),
            Err(error) => panic!("managed init failed: {error}"),
        }
    }
}
