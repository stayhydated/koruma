use stayhydated_xtask::web::WebBuildConfig;

pub fn run() -> anyhow::Result<()> {
    let workspace_root = stayhydated_xtask::workspace_root_from_xtask_manifest()?;
    stayhydated_xtask::web::build(
        WebBuildConfig::github_pages(workspace_root)
            .package("web")
            .sitemap_xml(web::sitemap_xml())
            .build(),
    )
}
