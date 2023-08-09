use proto_pdk_test_utils::*;
use starbase_sandbox::create_empty_sandbox;
use std::path::PathBuf;

#[test]
fn registers_metadata() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin("rust-test", sandbox.path());

    let metadata = plugin.register_tool(ToolMetadataInput {
        id: "rust-test".into(),
        env: plugin.tool.create_environment().unwrap(),
        home_dir: PathBuf::from("/home"),
    });

    assert_eq!(metadata.name, "Rust");
    assert_eq!(metadata.default_version, Some("stable".to_owned()));
    assert_eq!(metadata.inventory.disable_progress_bars, true);
    assert_eq!(
        metadata.inventory.override_dir,
        Some(PathBuf::from("/home/.rustup/toolchains"))
    );
    assert!(metadata.inventory.version_suffix.is_some());
}
