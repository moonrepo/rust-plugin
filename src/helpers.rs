use std::path::PathBuf;

use extism_pdk::*;
use proto_pdk::*;

#[host_fn]
extern "ExtismHost" {
    fn get_env_var(name: String) -> String;
    fn to_virtual_path(input: String) -> String;
}

pub fn get_rustup_home(env: &HostEnvironment) -> Result<VirtualPath, Error> {
    // Cargo sets the RUSTUP_HOME env var when running tests,
    // which causes a ton of issues, so intercept it here!
    if let Some(test_env) = get_test_environment()? {
        return Ok(virtual_path!(buf, test_env.sandbox).join(".home/.rustup"));
    }

    Ok(match host_env!("RUSTUP_HOME") {
        Some(path) => {
            let path = PathBuf::from(path);

            // Variable returns a real path, so convert to virtual
            if path.is_absolute() {
                virtual_path!(buf, path)
            } else {
                virtual_path!("/cwd").join(path)
            }
        }
        None => env.home_dir.join(".rustup"),
    })
}

pub fn get_channel_from_version(spec: &VersionSpec) -> String {
    if spec.is_canary() {
        "nightly".to_owned()
    } else {
        spec.to_string()
    }
}

pub fn is_non_version_channel(spec: &UnresolvedVersionSpec) -> bool {
    match spec {
        UnresolvedVersionSpec::Canary => true,
        UnresolvedVersionSpec::Alias(value) => {
            value == "stable"
                || value == "beta"
                || value == "nightly"
                || value.starts_with("nightly")
        }
        _ => false,
    }
}
