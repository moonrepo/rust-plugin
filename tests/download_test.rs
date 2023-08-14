use proto_pdk_test_utils::*;
use starbase_sandbox::create_empty_sandbox;
use std::path::PathBuf;

// We use a fake home directory but rustup requires a real one!
// generate_download_install_tests!("rust-test", "1.70.0");

#[test]
fn locates_linux_bin() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin("rust-test", sandbox.path());

    assert_eq!(
        plugin
            .locate_bins(LocateBinsInput {
                env: Environment {
                    arch: HostArch::Arm64,
                    os: HostOS::Linux,
                    version: "1.69.0".into(),
                    ..Default::default()
                },
                home_dir: PathBuf::new(),
                tool_dir: PathBuf::new(),
            })
            .bin_path,
        Some("bin/rustc".into())
    );
}

#[test]
fn locates_macos_bin() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin("rust-test", sandbox.path());

    assert_eq!(
        plugin
            .locate_bins(LocateBinsInput {
                env: Environment {
                    arch: HostArch::X64,
                    os: HostOS::MacOS,
                    version: "1.69.0".into(),
                    ..Default::default()
                },
                home_dir: PathBuf::new(),
                tool_dir: PathBuf::new(),
            })
            .bin_path,
        Some("bin/rustc".into())
    );
}

#[test]
fn locates_windows_bin() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin("rust-test", sandbox.path());

    assert_eq!(
        plugin
            .locate_bins(LocateBinsInput {
                env: Environment {
                    arch: HostArch::X86,
                    os: HostOS::Windows,
                    version: "1.69.0".into(),
                    ..Default::default()
                },
                home_dir: PathBuf::new(),
                tool_dir: PathBuf::new(),
            })
            .bin_path,
        Some("bin/rustc.exe".into())
    );
}
