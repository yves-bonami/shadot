use std::path::PathBuf;

use clap::Parser;
use inquire::{ui::RenderConfig, CustomType, Select};

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
    color_eyre::install()?;

    let cli = Cli::parse();
    match cli.command {
        Command::Init => initialize()?,
    }

    Ok(())
}

fn initialize() -> Result<()> {
    let shadow_dir_prompt: CustomType<PathBuf> = CustomType {
        message: "Choose your shadow directory:",
        starting_input: None,
        default: Some(
            directories::ProjectDirs::from("run", "dev dot run", "shadot")
                .unwrap()
                .data_dir()
                .to_path_buf(),
        ),
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

    let type_options: Vec<&str> = vec!["TOML", "YAML", "JSON"];
    let config_type =
        Select::new("Choose your preferred config file type:", type_options).prompt()?;

    println!(
        "Selected directory {} with file type {}",
        shadow_dir.to_string_lossy(),
        config_type
    );

    Ok(())
}
