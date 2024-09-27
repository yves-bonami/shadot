use std::{fs, path::PathBuf, process::exit, sync::Arc};

use arc_swap::ArcSwapOption;
use figment::{
    providers::{Format, Json, Toml, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG: &str = include_str!("../default.toml");
static CONFIG: ArcSwapOption<Config> = ArcSwapOption::const_empty();

pub fn init_config() {
    let current = CONFIG.load();
    if current.is_none()
        && CONFIG
            .compare_and_swap(
                current,
                match load_config().extract() {
                    Ok(config) => Some(config),
                    Err(errors) => {
                        eprintln!("Configuration is invalid");
                        for err in errors {
                            eprintln!("  - {err}");
                        }
                        exit(1);
                    }
                },
            )
            .is_none()
    {
        return;
    }

    panic!("Configuration already initialized");
}

pub fn update<F: Fn(&mut Config)>(f: F) {
    CONFIG.rcu(|c| {
        let mut new = (**c
            .as_ref()
            .expect("Tried to update config; but not yet loaded!"))
        .clone();
        f(&mut new);
        Some(Arc::new(new))
    });
}

pub fn save() -> crate::Result<()> {
    let dir = directories::ProjectDirs::from("run", "dev dot run", "shadot").unwrap();
    if !dir.config_dir().exists() {
        fs::create_dir_all(dir.config_dir())?;
    }

    let config = config();
    let result_string = match config.preferred_config_type.as_str() {
        "json" => serde_json::to_string_pretty(&config)?,
        "yaml" => serde_yaml::to_string(&config)?,
        _ => toml::to_string_pretty(&config)?,
    };

    let mut file_path = dir.config_dir().join("shadot");
    file_path.set_extension(&config.preferred_config_type);
    println!("writing to {}", file_path.to_string_lossy());
    fs::write(&file_path, &result_string)?;

    Ok(())
}

pub fn config() -> Arc<Config> {
    let cur = CONFIG.load();
    if cur.is_none() {
        eprintln!("Configuration not yet initialized. Run `shadot init` first.");
        CONFIG.compare_and_swap(cur, Some(load_config().extract().unwrap()));
    } else {
        drop(cur);
    }

    CONFIG.load_full().unwrap()
}

fn load_config() -> Figment {
    let mut figment = Figment::from(Toml::string(DEFAULT_CONFIG));

    if let Some(proj_dir) = directories::ProjectDirs::from("run", "dev dot run", "shadot") {
        let config_dir = proj_dir.config_dir();
        figment = figment
            .admerge(Json::file(config_dir))
            .admerge(Yaml::file(config_dir))
            .admerge(Toml::file(config_dir));
    }

    figment
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    pub shadow_dir: PathBuf,
    pub preferred_config_type: String,
}
