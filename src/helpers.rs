use extism_pdk::*;
use proto_pdk::*;
use std::path::PathBuf;

#[host_fn]
extern "ExtismHost" {
    fn get_env_var(name: String) -> String;
}

pub fn get_rustup_home(env: &HostEnvironment) -> Result<PathBuf, Error> {
    // Cargo sets the RUSTUP_HOME env var when running tests,
    // which causes a ton of issues, so intercept it here!
    if let Some(test_env) = get_test_environment()? {
        return Ok(test_env.sandbox.join(".home/.rustup"));
    }

    // Variable returns a real path
    Ok(host_env!("RUSTUP_HOME")
        .map(PathBuf::from)
        // So we need our fallback to also be a real path
        .unwrap_or_else(|| env.home_dir.real_path().unwrap().join(".rustup")))
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
