use crate::site::constants::SITE_URL;
use stayhydated_site::routing::{Href, SiteUrl};

pub(crate) fn render_sitemap() -> String {
    let mut paths = crate::site::routing::all_routes()
        .into_iter()
        .map(|route| route.path())
        .collect::<Vec<_>>();
    paths.extend([
        Href::new("/book/"),
        Href::new("/llms.txt"),
        Href::new("/llms-full.txt"),
    ]);

    stayhydated_site::sitemap::render(&SiteUrl::new(SITE_URL), paths)
}
