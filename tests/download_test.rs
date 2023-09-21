use proto_pdk_test_utils::*;
use starbase_sandbox::create_empty_sandbox;

// We use a fake home directory but rustup requires a real one!
// generate_download_install_tests!("rust-test", "1.70.0");

#[test]
fn locates_linux_bin() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_plugin("rust-test", sandbox.path());

    plugin.set_environment(HostEnvironment {
        arch: HostArch::Arm64,
        os: HostOS::Linux,
        ..Default::default()
    });

    assert_eq!(
        plugin
            .locate_bins(LocateBinsInput {
                context: ToolContext {
                    version: "1.69.0".into(),
                    ..Default::default()
                },
            })
            .bin_path,
        Some("bin/cargo".into())
    );
}

#[test]
fn locates_macos_bin() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_plugin("rust-test", sandbox.path());

    plugin.set_environment(HostEnvironment {
        arch: HostArch::X64,
        os: HostOS::MacOS,
        ..Default::default()
    });

    assert_eq!(
        plugin
            .locate_bins(LocateBinsInput {
                context: ToolContext {
                    version: "1.69.0".into(),
                    ..Default::default()
                },
            })
            .bin_path,
        Some("bin/cargo".into())
    );
}

#[test]
fn locates_windows_bin() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_plugin("rust-test", sandbox.path());

    plugin.set_environment(HostEnvironment {
        arch: HostArch::X86,
        os: HostOS::Windows,
        ..Default::default()
    });

    assert_eq!(
        plugin
            .locate_bins(LocateBinsInput {
                context: ToolContext {
                    version: "1.69.0".into(),
                    ..Default::default()
                },
            })
            .bin_path,
        Some("bin/cargo.exe".into())
    );
}
