use extism_pdk::*;
use proto_pdk::*;

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
    fn host_log(input: Json<HostLogInput>);
}

static NAME: &str = "Rust";

#[plugin_fn]
pub fn register_tool(Json(_): Json<ToolMetadataInput>) -> FnResult<Json<ToolMetadataOutput>> {
    Ok(Json(ToolMetadataOutput {
        name: NAME.into(),
        type_of: PluginType::Language,
        ..ToolMetadataOutput::default()
    }))
}

fn is_musl() -> bool {
    unsafe {
        match exec_command(Json(ExecCommandInput::new("ldd", ["--version"]))) {
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
pub fn download_prebuilt(
    Json(_): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    Ok(Json(DownloadPrebuiltOutput {
        download_url: "https://www.rust-lang.org/tools/install".into(),
        skip_download: true,
        ..DownloadPrebuiltOutput::default()
    }))
}

#[plugin_fn]
pub fn native_install(Json(input): Json<NativeInstallInput>) -> FnResult<()> {
    // Check if rustup is installed
    let output = exec_command!(
        if input.env.os == HostOS::Windows {
            "Get-Command"
        } else {
            "which"
        },
        ["rustup"]
    );

    if output.exit_code != 0 || output.stdout.is_empty() {
        return err!(
            "proto requires `rustup` to be installed and available on PATH to use Rust. Please install it and try again.".into(),
            output.exit_code
        );
    }

    let triple = format!("{}-{}", input.env.version, get_triple_target(&input.env)?);

    host_log!("Installing target \"{}\" with rustup", triple);

    // Install if not already installed
    let installed_list = exec_command!("rustup", ["toolchain", "list"]);

    if installed_list.stdout.contains(&triple) {
        host_log!("Target already installed in toolchain");
    } else {
        exec_command!(ExecCommandInput::stream(
            "rustup",
            ["toolchain", "install", &input.env.version]
        ));
    }

    Ok(())
}

#[plugin_fn]
pub fn locate_bins(Json(_): Json<LocateBinsInput>) -> FnResult<Json<LocateBinsOutput>> {
    Ok(Json(LocateBinsOutput {
        bin_path: None,
        fallback_last_globals_dir: true,
        globals_lookup_dirs: vec![
            "$CARGO_INSTALL_ROOT".into(),
            "$CARGO_HOME/bin".into(),
            "$HOME/.cargo/bin".into(),
        ],
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

    Ok(Json(LoadVersionsOutput::from_tags(&tags)?))
}

#[plugin_fn]
pub fn resolve_version(
    Json(input): Json<ResolveVersionInput>,
) -> FnResult<Json<ResolveVersionOutput>> {
    let mut output = ResolveVersionOutput::default();

    if input.initial == "stable"
        || input.initial == "beta"
        || input.initial == "nightly"
        || input.initial.starts_with("nightly")
    {
        output.version = Some(input.initial);
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
