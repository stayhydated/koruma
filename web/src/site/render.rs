use crate::site::constants::SITE_URL;

pub(crate) fn render_sitemap() -> String {
    let mut paths = crate::site::routing::PageKind::all()
        .into_iter()
        .map(|page| page.path())
        .collect::<Vec<_>>();
    paths.extend([
        "/book/".to_string(),
        "/llms.txt".to_string(),
        "/llms-full.txt".to_string(),
    ]);

    stayhydated_site::sitemap::render(SITE_URL, paths)
}
