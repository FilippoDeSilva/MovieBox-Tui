use moviebox_tui::providers::addons::models::AddonManifest;

#[test]
fn test_addon_manifest_fixture_deserialization() {
    let fixture_content = include_str!("fixtures/addons/manifest.json");
    let manifest: AddonManifest = serde_json::from_str(fixture_content)
        .expect("failed to deserialize Cinemeta addon manifest");

    assert_eq!(manifest.id, "org.stremio.cinemeta");
    assert_eq!(manifest.name, "Cinemeta");
    assert_eq!(manifest.version.as_deref(), Some("3.0.12"));
    assert_eq!(manifest.resources.len(), 3);
    assert_eq!(manifest.resources[0].name(), "catalog");
    assert_eq!(manifest.resources[1].name(), "meta");
    assert_eq!(manifest.resources[2].name(), "stream");

    assert_eq!(manifest.catalogs.len(), 2);
    assert_eq!(manifest.catalogs[0].r#type, "movie");
    assert_eq!(manifest.catalogs[0].id, "top");
    assert_eq!(manifest.catalogs[0].name.as_deref(), Some("Popular Movies"));
    assert_eq!(manifest.catalogs[0].extra.len(), 1);
    assert_eq!(manifest.catalogs[0].extra[0].name, "genre");
}
