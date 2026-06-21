use std::path::PathBuf;

use areia::{BaseDirs, Hidden};
use athena::Object;
use brigid::{Brigid, content::Content};
use mawu::XffValue::{self};
use nemesis::{NemesisError, NemesisResultExt};
use nomos::{NomosError, NomosResult};

pub struct Startup {
    pub config: XffValue,
}

pub fn startup() -> NomosResult<Startup> {
    let config = make_and_get_config().add_ctx("Startup of CLI failed during config getting.")?;
    Ok(Startup { config })
}

fn validate_config(config: XffValue) -> NomosResult<XffValue> {
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
            Ok(config)
        }
    } else {
        Err(NemesisError::new(
            "nomos::startup::validate_config",
            NomosError::Config("Invalid global config: Not an object".to_string()),
        )
        .add_ctx(format!("Got global config: {config}")))
    }
}

fn make_and_get_config() -> NomosResult<XffValue> {
    let root = make_base_dir().add_ctx("Failed to make base dir during CLI startup.")?;
    let brigid = Brigid::new(root)
        .file("config.json", |file| {
            file.with_default_content(Content::JSON(XffValue::from(
                Object::new().push("search_bases", vec![].into()),
            )))
            .with_fallback();
        })
        .add_license(include_str!("../../LICENSE"), root.join("LICENSE"))
        .establish()?;
    validate_config(brigid.get_file("config.json")?)
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
