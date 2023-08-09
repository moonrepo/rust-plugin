use proto_pdk_test_utils::create_plugin;
use starbase_sandbox::create_empty_sandbox;

#[tokio::test]
async fn doesnt_create_global_shims() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin("rust-test", sandbox.path());

    plugin.tool.create_shims(false).await.unwrap();

    assert!(!sandbox.path().join(".proto/bin/rustc").exists());
    assert!(!sandbox.path().join(".proto/bin/cargo").exists());
}
