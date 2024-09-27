use std::{fs, path::PathBuf, process::exit};

use clap::Parser;
use config::init_config;
use inquire::{ui::RenderConfig, Confirm, CustomType, Select};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;

type Result<T> = color_eyre::eyre::Result<T>;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    color_eyre::install()?;

    let cli = Cli::parse();

    tracing::info!("running shadot cli");

    match cli.command {
        Command::Init => initialize()?,
    }

    Ok(())
}

fn initialize() -> Result<()> {
    init_config();

    let proj_dir = directories::ProjectDirs::from("run", "dev dot run", "shadot").unwrap();

    if proj_dir.config_dir().read_dir()?.count() > 0 {
        let overwrite =
            Confirm::new("Shadot already initialized, are you sure you want to overwrite?")
                .prompt()?;

        if !overwrite {
            exit(1);
        }

        fs::remove_dir_all(proj_dir.config_dir())?;
    }

    let shadow_dir_prompt: CustomType<PathBuf> = CustomType {
        message: "Choose your shadow directory:",
        starting_input: None,
        default: Some(proj_dir.data_dir().to_path_buf()),
        placeholder: None,
        help_message: "This is where shadow copies of your config files will be kept".into(),
        formatter: &|i| i.to_string_lossy().into(),
        default_value_formatter: &|i| i.to_string_lossy().into(),
        parser: &|i| Ok(PathBuf::from(i)),
        validators: vec![],
        error_message: "".into(),
        render_config: RenderConfig::default(),
    };
    let shadow_dir = shadow_dir_prompt.prompt()?;

    let type_options: Vec<&str> = vec!["toml", "yaml", "json"];
    let config_type =
        Select::new("Choose your preferred config file type:", type_options).prompt()?;

    config::update(|c| {
        c.shadow_dir = shadow_dir.to_path_buf();
        c.preferred_config_type = config_type.to_string();
    });

    config::save()?;

    Ok(())
}
