use proto_pdk_test_utils::*;
use starbase_sandbox::create_empty_sandbox;
use std::path::PathBuf;

#[test]
fn registers_metadata() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin("rust-test", sandbox.path());

    let metadata = plugin.register_tool(ToolMetadataInput {
        id: "rust-test".into(),
    });

    assert_eq!(metadata.name, "Rust");
    assert_eq!(metadata.default_version, Some("stable".to_owned()));
    assert!(metadata.inventory.disable_progress_bars);
    assert_eq!(
        metadata.inventory.override_dir,
        Some(PathBuf::from("/workspace/.home/.rustup/toolchains"))
    );
    assert!(metadata.inventory.version_suffix.is_some());
}
