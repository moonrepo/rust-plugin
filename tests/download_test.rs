use proto_pdk_test_utils::*;
use starbase_sandbox::create_empty_sandbox;
use std::collections::HashMap;

// We use a fake home directory but rustup requires a real one!
// generate_download_install_tests!("rust-test", "1.70.0");

#[test]
fn locates_linux_bin() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin_with_config(
        "rust-test",
        sandbox.path(),
        HashMap::from_iter([map_config_environment(HostOS::Linux, HostArch::Arm64)]),
    );

    assert_eq!(
        plugin
            .locate_executables(LocateExecutablesInput {
                context: ToolContext {
                    version: VersionSpec::parse("1.69.0").unwrap(),
                    ..Default::default()
                },
            })
            .primary
            .unwrap()
            .exe_path,
        Some("bin/cargo".into())
    );
}

#[test]
fn locates_macos_bin() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin_with_config(
        "rust-test",
        sandbox.path(),
        HashMap::from_iter([map_config_environment(HostOS::MacOS, HostArch::X64)]),
    );

    assert_eq!(
        plugin
            .locate_executables(LocateExecutablesInput {
                context: ToolContext {
                    version: VersionSpec::parse("1.69.0").unwrap(),
                    ..Default::default()
                },
            })
            .primary
            .unwrap()
            .exe_path,
        Some("bin/cargo".into())
    );
}

#[test]
fn locates_windows_bin() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin_with_config(
        "rust-test",
        sandbox.path(),
        HashMap::from_iter([map_config_environment(HostOS::Windows, HostArch::X86)]),
    );

    assert_eq!(
        plugin
            .locate_executables(LocateExecutablesInput {
                context: ToolContext {
                    version: VersionSpec::parse("1.69.0").unwrap(),
                    ..Default::default()
                },
            })
            .primary
            .unwrap()
            .exe_path,
        Some("bin/cargo.exe".into())
    );
}
