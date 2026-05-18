use crate::site::constants::SITE_URL;
use std::fmt::Write as _;

pub(crate) fn render_sitemap() -> String {
    let mut entries = String::new();

    for page in crate::site::routing::PageKind::all() {
        let path = page.path();
        let url = if path == "/" {
            SITE_URL.to_string()
        } else {
            format!("{SITE_URL}{}", path.trim_start_matches('/'))
        };
        let _ = writeln!(entries, "  <url><loc>{url}</loc></url>");
    }

    let _ = writeln!(entries, "  <url><loc>{SITE_URL}book/</loc></url>");
    let _ = writeln!(entries, "  <url><loc>{SITE_URL}llms.txt</loc></url>");
    let _ = writeln!(entries, "  <url><loc>{SITE_URL}llms-full.txt</loc></url>");

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{entries}</urlset>\n"
    )
}
