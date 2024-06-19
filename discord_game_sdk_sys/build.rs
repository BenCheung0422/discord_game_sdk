use std::{env, fs, path::*};
use std::fs::{create_dir_all, File};
use std::io::Write;
use zip::ZipArchive;

const SDK_VERSION: &str = "3.2.1";

fn main() {
    // DO NOT RELY ON THIS
    if cfg!(feature = "private-docs-rs") {
        return generate_ffi_bindings(bindgen::builder().header("discord_game_sdk.h"));
    }

    let sdk_path = fetch_discord_game_sdk();
    // println!("cargo:rustc-env=LD_LIBRARY_PATH={}", fs::canonicalize(sdk_path.join("lib/x86_64")).unwrap().display());
    // println!("cargo:rustc-env=DYLD_LIBRARY_PATH={}", fs::canonicalize(sdk_path.join("lib/x86_64")).unwrap().display());
    println!("cargo:rustc-link-search={}", fs::canonicalize(sdk_path.join("lib/x86_64")).unwrap().to_str().unwrap());
    println!("cargo:rerun-if-changed={}", sdk_path.display());

    generate_ffi_bindings(
        bindgen::builder().header(sdk_path.join("c/discord_game_sdk.h").to_str().unwrap()),
    );

    if cfg!(feature = "link") {
        let target = env::var("TARGET").unwrap();

        verify_installation(&target, &sdk_path);
        configure_linkage(&target, &sdk_path);
    }
}

fn fetch_discord_game_sdk() -> PathBuf {
    let mut sdk_version = SDK_VERSION.to_string();
    println!("Target Discord GameSDK: {sdk_version}");
    let main_dir = PathBuf::from("libraries");
    let extract_dir = main_dir.join("discord_game_sdk");
    create_dir_all(&extract_dir).unwrap();
    println!("Checking local cache...");

    // Check cached version
    let version_file_path = extract_dir.join("VERSION");
    if version_file_path.exists() {
        match fs::read_to_string(&version_file_path) {
            Ok(ver) => {
                if ver == sdk_version {
                    println!("Version matched, no upgrade required.");
                    return extract_dir;
                } else {
                    println!("Version not matched, an upgrade is required.");
                }
            }
            Err(err) => eprintln!("Error opening VERSION: {err}"),
        }
    }

    println!("Clearing cache directory...");
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir).expect("remove directory");
        fs::create_dir(&extract_dir).expect("create directory");
    }

    println!("Fetching {sdk_version} SDK...");
    let download_path = main_dir.join("fetch.zip");
    reqwest::blocking::get(format!(
        "https://dl-game-sdk.discordapp.net/{sdk_version}/discord_game_sdk.zip"))
        .expect("request Discord Game SDK")
        .copy_to(&mut File::create(&download_path).expect("Discord Game SDK local cache"))
        .expect("download Discord Game SDK");
    ZipArchive::new(File::open(&download_path).expect("read download cache"))
        .expect("valid zip")
        .extract(&extract_dir)
        .expect("extract cached zip");
    fs::remove_file(download_path).expect("remove cached zip");
    File::create(version_file_path).expect("create VERSION")
        .write_all(unsafe { sdk_version.as_bytes_mut() }).expect("write VERSION");
    println!("SDK fetched.");

    // Prepare for library

    fs::rename(extract_dir.join("lib/x86_64/discord_game_sdk.so"),
               extract_dir.join("lib/x86_64/libdiscord_game_sdk.so")).unwrap();
    fs::rename(extract_dir.join("lib/x86_64/discord_game_sdk.dylib"),
               extract_dir.join("lib/x86_64/libdiscord_game_sdk.dylib")).unwrap();
    fs::rename(extract_dir.join("lib/x86_64/discord_game_sdk.dll.lib"),
               extract_dir.join("lib/x86_64/discord_game_sdk.lib")).unwrap();
    fs::rename(extract_dir.join("lib/x86/discord_game_sdk.dll.lib"),
               extract_dir.join("lib/x86/discord_game_sdk.lib")).unwrap();

    extract_dir
}

fn verify_installation(target: &str, sdk_path: &Path) {
    match target {
        "x86_64-unknown-linux-gnu" => {
            assert!(
                sdk_path.join("lib/x86_64/libdiscord_game_sdk.so").exists(),
                "{}",
                MISSING_SETUP
            );
        }

        "x86_64-apple-darwin" => {
            assert!(
                sdk_path
                    .join("lib/x86_64/libdiscord_game_sdk.dylib")
                    .exists(),
                "{}",
                MISSING_SETUP
            );
        }

        "x86_64-pc-windows-gnu" | "x86_64-pc-windows-msvc" => {
            assert!(
                sdk_path.join("lib/x86_64/discord_game_sdk.lib").exists(),
                "{}",
                MISSING_SETUP
            );
        }

        "i686-pc-windows-gnu" | "i686-pc-windows-msvc" => {
            assert!(
                sdk_path.join("lib/x86/discord_game_sdk.lib").exists(),
                "{}",
                MISSING_SETUP
            );
        }

        _ => panic!("{}", INCOMPATIBLE_PLATFORM),
    }
}

fn configure_linkage(target: &str, sdk_path: &Path) {
    match target {
        "x86_64-unknown-linux-gnu"
        | "x86_64-apple-darwin"
        | "x86_64-pc-windows-gnu"
        | "x86_64-pc-windows-msvc" => {
            println!("cargo:rustc-link-lib=discord_game_sdk");
            println!(
                "cargo:rustc-link-search={}",
                sdk_path.join("lib/x86_64").display()
            );
        }

        "i686-pc-windows-gnu" | "i686-pc-windows-msvc" => {
            println!("cargo:rustc-link-lib=discord_game_sdk");
            println!(
                "cargo:rustc-link-search={}",
                sdk_path.join("lib/x86").display()
            );
        }

        _ => {}
    }
}

fn generate_ffi_bindings(builder: bindgen::Builder) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    builder
        .ctypes_prefix("ctypes")
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_partialeq(true)
        .generate_comments(false)
        .impl_debug(true)
        .impl_partialeq(true)
        .parse_callbacks(Box::new(Callbacks))
        .prepend_enum_name(false)
        .allowlist_function("Discord.+")
        .allowlist_type("[EI]?Discord.+")
        .allowlist_var("DISCORD_.+")
        .generate()
        .expect("discord_game_sdk_sys: bindgen could not generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("discord_game_sdk_sys: could not write bindings to file");
}

#[derive(Debug)]
struct Callbacks;

impl bindgen::callbacks::ParseCallbacks for Callbacks {
    fn int_macro(&self, name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        // Must match sys::DiscordVersion
        if name.ends_with("_VERSION") {
            Some(bindgen::callbacks::IntKind::I32)
        } else {
            None
        }
    }
}

const MISSING_SETUP: &str = r#"

discord_game_sdk_sys: Hello,

You are trying to link to the Discord Game SDK.
Some additional set-up is required, namely some files need to be copied for the linker:

# Linux: prepend with `lib` and add to library search path
$ cp $DISCORD_GAME_SDK_PATH/lib/x86_64/{,lib}discord_game_sdk.so
$ export LD_LIBRARY_PATH=${LD_LIBRARY_PATH:+${LD_LIBRARY_PATH}:}$DISCORD_GAME_SDK_PATH/lib/x86_64

# Mac OS: prepend with `lib` and add to library search path
$ cp $DISCORD_GAME_SDK_PATH/lib/x86_64/{,lib}discord_game_sdk.dylib
$ export DYLD_LIBRARY_PATH=${DYLD_LIBRARY_PATH:+${DYLD_LIBRARY_PATH}:}$DISCORD_GAME_SDK_PATH/lib/x86_64

# Windows: copy `*.dll.lib` to `*.lib` (won't affect library search)
$ cp $DISCORD_GAME_SDK_PATH/lib/x86_64/discord_game_sdk.{dll.lib,lib}
$ cp $DISCORD_GAME_SDK_PATH/lib/x86/discord_game_sdk.{dll.lib,lib}

After all this, `cargo build` and `cargo run` should function as expected.

Please report any issues you have at:
https://github.com/ldesgoui/discord_game_sdk

Thanks, and apologies for the inconvenience

"#;

const INCOMPATIBLE_PLATFORM: &str = r#"

discord_game_sdk_sys: Hello,

You are trying to link to the Discord Game SDK.
Unfortunately, the platform you are trying to target is not supported.

Please report any issues you have at:
https://github.com/ldesgoui/discord_game_sdk

Thanks, and apologies for the inconvenience

"#;
