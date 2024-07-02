use crate::helpers::*;
use crate::toolchain_toml::ToolchainToml;
use extism_pdk::*;
use proto_pdk::*;
use std::fs;
use std::path::PathBuf;

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
    fn set_env_var(name: String, value: String);
}

static NAME: &str = "Rust";

fn get_toolchain_dir(env: &HostEnvironment) -> AnyResult<VirtualPath> {
    Ok(get_rustup_home(env)?.join("toolchains"))
}

#[plugin_fn]
pub fn register_tool(Json(_): Json<ToolMetadataInput>) -> FnResult<Json<ToolMetadataOutput>> {
    let env = get_host_environment()?;

    Ok(Json(ToolMetadataOutput {
        name: NAME.into(),
        type_of: PluginType::Language,
        default_version: Some(UnresolvedVersionSpec::Alias("stable".into())),
        inventory: ToolInventoryMetadata {
            disable_progress_bars: true,
            override_dir: Some(get_toolchain_dir(&env)?),
            version_suffix: Some(format!("-{}", get_target_triple(&env, NAME)?)),
        },
        plugin_version: Some(env!("CARGO_PKG_VERSION").into()),
        ..ToolMetadataOutput::default()
    }))
}

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let tags = load_git_tags("https://github.com/rust-lang/rust")?;

    let tags = tags
        .into_iter()
        .filter_map(|tag| {
            // Filter out old versions
            if tag.starts_with("release-") || tag.starts_with("0.") {
                None
            } else {
                Some(tag)
            }
        })
        .collect::<Vec<_>>();

    Ok(Json(LoadVersionsOutput::from(tags)?))
}

#[plugin_fn]
pub fn resolve_version(
    Json(input): Json<ResolveVersionInput>,
) -> FnResult<Json<ResolveVersionOutput>> {
    let mut output = ResolveVersionOutput::default();

    // Allow channels as explicit aliases
    if is_non_version_channel(&input.initial) {
        output.version = Some(VersionSpec::Alias(input.initial.into()));
    } else if input.initial.is_canary() {
        output.version = Some(VersionSpec::Alias("nightly".into()));
    }

    Ok(Json(output))
}

#[plugin_fn]
pub fn detect_version_files(_: ()) -> FnResult<Json<DetectVersionOutput>> {
    Ok(Json(DetectVersionOutput {
        files: vec!["rust-toolchain.toml".into(), "rust-toolchain".into()],
        ignore: vec![],
    }))
}

#[plugin_fn]
pub fn parse_version_file(
    Json(input): Json<ParseVersionFileInput>,
) -> FnResult<Json<ParseVersionFileOutput>> {
    let mut output = ParseVersionFileOutput::default();

    if input.file == "rust-toolchain" {
        if !input.content.is_empty() {
            output.version = Some(UnresolvedVersionSpec::parse(input.content)?);
        }
    } else if input.file == "rust-toolchain.toml" {
        let config: ToolchainToml = toml::from_str(&input.content)?;

        if let Some(channel) = config.toolchain.channel {
            output.version = Some(UnresolvedVersionSpec::parse(channel)?);
        }
    }

    Ok(Json(output))
}

#[plugin_fn]
pub fn native_install(
    Json(input): Json<NativeInstallInput>,
) -> FnResult<Json<NativeInstallOutput>> {
    let env = get_host_environment()?;

    // Install rustup if it does not exist
    if !command_exists(&env, "rustup") {
        debug!("Installing <shell>rustup</shell>");

        let is_windows = env.os.is_windows();
        let script_path = PathBuf::from("/proto/temp").join(if is_windows {
            "rustup-init.exe"
        } else {
            "rustup-init.sh"
        });

        if !script_path.exists() {
            fs::write(
                &script_path,
                fetch(
                    HttpRequest::new(if is_windows {
                        "https://win.rustup.rs"
                    } else {
                        "https://sh.rustup.rs"
                    }),
                    None,
                )?
                .body(),
            )?;
        }

        exec_command!(
            input,
            ExecCommandInput {
                command: script_path.to_string_lossy().to_string(),
                args: vec!["--default-toolchain".into(), "none".into(), "-y".into()],
                set_executable: true,
                stream: true,
                ..ExecCommandInput::default()
            }
        );

        // Update PATH explicitly, since we can't "reload the shell"
        // on the host side. This is good enough since it's deterministic.
        host_env!(
            "PATH",
            format!("{}:$HOME/.cargo/bin", get_cargo_home(&env)?.join("bin"))
        );
    }

    let version = &input.context.version;
    let channel = get_channel_from_version(version);

    let triple = format!("{}-{}", channel, get_target_triple(&env, NAME)?);

    debug!("Installing target <id>{}</id> with rustup", triple);

    // Install if not already installed
    let installed_list = exec_command!(pipe, "rustup", ["toolchain", "list"]);
    let mut do_install = true;

    if installed_list
        .stdout
        .lines()
        .any(|line| line.starts_with(&triple))
    {
        // Ensure the bins exist and that this isn't just an empty folder
        if input.context.tool_dir.join("bin").exists() {
            debug!("Target already installed in toolchain");

            do_install = false;

        // Otherwise empty folders cause issues with rustup, so force uninstall it
        } else {
            debug!("Detected a broken toolchain, uninstalling it");

            exec_command!(inherit, "rustup", ["toolchain", "uninstall", &triple]);
        }
    }

    if do_install {
        exec_command!(
            inherit,
            "rustup",
            ["toolchain", "install", &triple, "--force"]
        );
    }

    // Always mark as installed so that binaries can be located!
    Ok(Json(NativeInstallOutput {
        installed: true,
        ..NativeInstallOutput::default()
    }))
}

#[plugin_fn]
pub fn native_uninstall(
    Json(input): Json<NativeUninstallInput>,
) -> FnResult<Json<NativeUninstallOutput>> {
    let env = get_host_environment()?;
    let channel = get_channel_from_version(&input.context.version);
    let triple = format!("{}-{}", channel, get_target_triple(&env, NAME)?);

    exec_command!(inherit, "rustup", ["toolchain", "uninstall", &triple]);

    Ok(Json(NativeUninstallOutput {
        uninstalled: true,
        ..NativeUninstallOutput::default()
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;

    // Binaries are provided by Cargo (`~/.cargo/bin`), so don't create
    // our own shim and bin. But we do need to ensure that the install
    // worked, so we still check for the `cargo` binary.
    let mut primary = ExecutableConfig::new(env.os.get_exe_name("bin/cargo"));
    primary.no_bin = true;
    primary.no_shim = true;

    Ok(Json(LocateExecutablesOutput {
        exes_dir: Some("bin".into()),
        globals_lookup_dirs: vec![
            "$CARGO_INSTALL_ROOT/bin".into(),
            "$CARGO_HOME/bin".into(),
            "$HOME/.cargo/bin".into(),
        ],
        globals_prefix: Some("cargo-".into()),
        primary: Some(primary),
        ..LocateExecutablesOutput::default()
    }))
}

#[plugin_fn]
pub fn sync_manifest(Json(_): Json<SyncManifestInput>) -> FnResult<Json<SyncManifestOutput>> {
    let env = get_host_environment()?;
    let triple = get_target_triple(&env, NAME)?;
    let toolchain_dir = get_toolchain_dir(&env)?;
    let mut output = SyncManifestOutput::default();
    let mut versions = vec![];

    // Path may not be whitelisted, so exit early instead of failing
    let Ok(dirs) = fs::read_dir(toolchain_dir) else {
        return Ok(Json(output));
    };

    for dir in dirs {
        let dir = dir?.path();

        if !dir.is_dir() {
            continue;
        }

        let name = dir.file_name().unwrap_or_default().to_string_lossy();

        if !name.ends_with(&triple) {
            continue;
        }

        let Ok(spec) = VersionSpec::parse(name.replace(&format!("-{triple}"), "")) else {
            continue;
        };

        if is_non_version_channel(&spec.to_unresolved_spec()) {
            continue;
        }

        versions.push(spec);
    }

    if !versions.is_empty() {
        output.versions = Some(versions);
    }

    Ok(Json(output))
}
