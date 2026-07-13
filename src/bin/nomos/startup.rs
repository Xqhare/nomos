use std::{path::PathBuf, process::exit};

use areia::{BaseDirs, Hidden};
use athena::Object;
use brigid::{Brigid, content::Content};
use mawu::XffValue::{self};
use nemesis::{NemesisError, NemesisResultExt};
use nomos::{NomosError, NomosResult};

pub struct Startup {
    /// Will probably be used in TUI later
    pub _config: XffValue,
    /// The global config file
    ///
    /// Either `~/.config/nomos/config.json` or `~/.nomos/config.json`
    pub global_config_file: PathBuf,
}

pub fn startup() -> NomosResult<Startup> {
    let (config, global_config_file) =
        make_and_get_config().add_ctx("Startup of CLI failed during config getting.")?;
    Ok(Startup {
        _config: config,
        global_config_file,
    })
}

fn print_init_done(global_config_file: PathBuf) {
    println!("");
    println!("Nomos successfully installed!");
    println!("");
    println!("Config file: {global_config_file:?}");
    println!("");
    println!("Please update your global config file with your project paths.");
    println!("");
    println!(
        "Afterwards, run `nomos validate` to validate your config and project specific setups."
    );
    println!("Also available is `nomos next` and `nomos all` to show the next tasks to work on.");
    println!("");
}

fn validate_config(
    config: XffValue,
    global_config_file: PathBuf,
) -> NomosResult<(XffValue, PathBuf)> {
    if config == make_default_config() {
        print_init_done(global_config_file);
        exit(0);
    }
    if let Some(obj) = config.as_object() {
        if obj.get("search_bases").is_none() {
            Err(NemesisError::new(
                "nomos::startup::validate_config",
                NomosError::Config(
                    "Invalid global config: Key 'search_bases' does not exist".to_string(),
                ),
            )
            .add_ctx(format!("Got global config: {config}")))
        } else {
            Ok((config, global_config_file))
        }
    } else {
        Err(NemesisError::new(
            "nomos::startup::validate_config",
            NomosError::Config("Invalid global config: Not an object".to_string()),
        )
        .add_ctx(format!("Got global config: {config}")))
    }
}

/// Returns the path to the global config file as well as the config itself
fn make_and_get_config() -> NomosResult<(XffValue, PathBuf)> {
    let root = make_base_dir().add_ctx("Failed to make base dir during CLI startup.")?;
    let brigid = Brigid::new(&root)
        .file("config.json", |file| {
            file.with_default_content(Content::JSON(make_default_config()))
                .with_fallback();
        })
        .add_license(include_str!("../../../LICENSE"), root.join("LICENSE"))
        .establish()?;
    validate_config(brigid.get_file("config.json")?, root.join("config.json"))
}

fn make_default_config() -> XffValue {
    let mut files_obj = Object::new();
    files_obj.insert("project_name", XffValue::from("complete/path/to/file.md"));
    let mut obj = Object::new();
    obj.insert(
        "search_bases",
        XffValue::from(vec![XffValue::from("complete/path/to/dir")]),
    );
    obj.insert("files", XffValue::from(files_obj));
    obj.insert("version", XffValue::from(1));
    obj.insert("default_prio_level", XffValue::from(5));
    obj.into()
}

/// Make the base directory path.
fn make_base_dir() -> NomosResult<PathBuf> {
    let areia = BaseDirs::new()?;
    let ext = "nomos";
    if areia.config_dir().exists() {
        Ok(areia.config_dir().to_path_buf().join(ext))
    } else {
        let mut path = areia.data_dir().to_path_buf().join(ext);
        path.hide()?;
        // `.hide()` creates the directory (and modifies the path itself)
        if path.exists() {
            Ok(areia.home_dir().to_path_buf().join(path))
        } else {
            Err(NemesisError::new(
                "nomos::startup::make_base_dir",
                NomosError::CLI("Could not find or create base directory".to_string()),
            )
            .add_ctx("Config dir was already not found, home dir usage went wrong"))
        }
    }
}
