use crate::toolchain_toml::ToolchainToml;
use extism_pdk::*;
use proto_pdk::*;
use std::fs;

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
    fn host_log(input: Json<HostLogInput>);
}

static NAME: &str = "Rust";

#[plugin_fn]
pub fn register_tool(Json(input): Json<ToolMetadataInput>) -> FnResult<Json<ToolMetadataOutput>> {
    Ok(Json(ToolMetadataOutput {
        name: NAME.into(),
        type_of: PluginType::Language,
        default_version: Some("stable".into()),
        inventory: ToolInventoryMetadata {
            disable_progress_bars: true,
            override_dir: Some(input.home_dir.join(".rustup/toolchains")),
            version_suffix: Some(format!("-{}", get_triple_target(&input.env)?)),
        },
        plugin_version: Some(env!("CARGO_PKG_VERSION").into()),
        ..ToolMetadataOutput::default()
    }))
}

fn is_musl() -> bool {
    unsafe {
        match exec_command(Json(ExecCommandInput::pipe("ldd", ["--version"]))) {
            Ok(res) => res.0.stdout.contains("musl"),
            Err(_) => false,
        }
    }
}

fn get_triple_target(env: &Environment) -> Result<String, PluginError> {
    let arch = env.arch.to_rust_arch();

    Ok(match &env.os {
        HostOS::Linux => format!(
            "{}-unknown-linux-{}",
            arch,
            if is_musl() { "musl" } else { "gnu" }
        ),
        HostOS::MacOS => format!("{}-apple-darwin", arch),
        HostOS::Windows => format!("{}-pc-windows-msvc", arch),
        _ => {
            return Err(PluginError::UnsupportedTarget {
                tool: NAME.into(),
                arch: env.arch.to_string(),
                os: env.os.to_string(),
            })
        }
    })
}

#[plugin_fn]
pub fn native_install(
    Json(input): Json<NativeInstallInput>,
) -> FnResult<Json<NativeInstallOutput>> {
    // Check if rustup is installed
    let result = exec_command!(
        if input.env.os == HostOS::Windows {
            "Get-Command"
        } else {
            "which"
        },
        ["rustup"]
    );

    if result.exit_code != 0 || result.stdout.is_empty() {
        return err!(
            "proto requires `rustup` to be installed and available on PATH to use Rust. Please install it and try again.".into(),
            result.exit_code
        );
    }

    let triple = format!("{}-{}", input.env.version, get_triple_target(&input.env)?);

    host_log!("Installing target \"{}\" with rustup", triple);

    // Install if not already installed
    let installed_list = exec_command!("rustup", ["toolchain", "list"]);

    if installed_list.stdout.contains(&triple) {
        host_log!("Target already installed in toolchain");
    } else {
        exec_command!(ExecCommandInput::inherit(
            "rustup",
            ["toolchain", "install", &input.env.version]
        ));
    }

    // Always mark as installed so that binaries can be located!
    Ok(Json(NativeInstallOutput { installed: true }))
}

#[plugin_fn]
pub fn locate_bins(Json(input): Json<LocateBinsInput>) -> FnResult<Json<LocateBinsOutput>> {
    Ok(Json(LocateBinsOutput {
        bin_path: Some(format_bin_name("bin/rustc", input.env.os).into()),
        fallback_last_globals_dir: true,
        globals_lookup_dirs: vec![
            "$CARGO_INSTALL_ROOT".into(),
            "$CARGO_HOME/bin".into(),
            "$HOME/.cargo/bin".into(),
        ],
        globals_prefix: Some("cargo-".into()),
    }))
}

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let tags = load_git_tags("https://github.com/rust-lang/rust")?;

    let tags = tags
        .iter()
        // Filter out old versions
        .filter(|t| !t.ends_with("^{}") && !t.starts_with("release-") && !t.starts_with("0."))
        .map(|t| t.to_owned())
        .collect::<Vec<_>>();

    Ok(Json(LoadVersionsOutput::from(tags)?))
}

fn is_non_version_channel(value: &str) -> bool {
    value == "stable" || value == "beta" || value == "nightly" || value.starts_with("nightly")
}

#[plugin_fn]
pub fn resolve_version(
    Json(input): Json<ResolveVersionInput>,
) -> FnResult<Json<ResolveVersionOutput>> {
    let mut output = ResolveVersionOutput::default();

    // Allow channels as explicit aliases
    if is_non_version_channel(&input.initial) {
        output.version = Some(input.initial);
    }

    Ok(Json(output))
}

#[plugin_fn]
pub fn detect_version_files(_: ()) -> FnResult<Json<DetectVersionOutput>> {
    Ok(Json(DetectVersionOutput {
        files: vec!["rust-toolchain.toml".into(), "rust-toolchain".into()],
    }))
}

#[plugin_fn]
pub fn parse_version_file(
    Json(input): Json<ParseVersionFileInput>,
) -> FnResult<Json<ParseVersionFileOutput>> {
    let mut output = ParseVersionFileOutput::default();

    if input.file == "rust-toolchain" {
        if !input.content.is_empty() {
            output.version = Some(input.content);
        }
    } else if input.file == "rust-toolchain.toml" {
        let config: ToolchainToml = toml::from_str(&input.content)?;

        if let Some(channel) = config.toolchain.channel {
            output.version = Some(channel);
        }
    }

    Ok(Json(output))
}

#[plugin_fn]
pub fn create_shims(Json(_): Json<CreateShimsInput>) -> FnResult<Json<CreateShimsOutput>> {
    Ok(Json(CreateShimsOutput {
        no_primary_global: true,
        ..CreateShimsOutput::default()
    }))
}

#[plugin_fn]
pub fn install_global(
    Json(input): Json<InstallGlobalInput>,
) -> FnResult<Json<InstallGlobalOutput>> {
    let result = exec_command!("cargo", ["install", "--force", &input.dependency]);

    Ok(Json(InstallGlobalOutput::from_exec_command(result)))
}

#[plugin_fn]
pub fn uninstall_global(
    Json(input): Json<UninstallGlobalInput>,
) -> FnResult<Json<UninstallGlobalOutput>> {
    let result = exec_command!("cargo", ["uninstall", &input.dependency]);

    Ok(Json(UninstallGlobalOutput::from_exec_command(result)))
}

#[plugin_fn]
pub fn sync_manifest(Json(input): Json<SyncManifestInput>) -> FnResult<Json<SyncManifestOutput>> {
    let triple = get_triple_target(&input.env)?;
    let mut output = SyncManifestOutput::default();
    let mut versions = vec![];

    for dir in fs::read_dir(input.home_dir.join(".rustup/toolchains"))? {
        let dir = dir?.path();

        if !dir.is_dir() {
            continue;
        }

        let name = dir.file_name().unwrap_or_default().to_string_lossy();

        if !name.ends_with(&triple) {
            continue;
        }

        let name = name.replace(&format!("-{triple}"), "");

        if is_non_version_channel(&name) {
            continue;
        }

        versions.push(Version::parse(&name)?);
    }

    if !versions.is_empty() {
        output.versions = Some(versions);
    }

    Ok(Json(output))
}
