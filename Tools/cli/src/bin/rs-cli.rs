use std::{collections::HashSet, fs, path::PathBuf, process::exit};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use indicatif::ProgressBar;
use itertools::Itertools;
use log::LevelFilter;
use regex::Regex;
use rs_cli::core::{item_definition::ItemDefinition, log::initialize_logging};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    /// overrides the log level
    #[arg(short = 'l', long, default_value = "INFO", verbatim_doc_comment)]
    log_level: LevelFilter,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// prints items found via the specified arguments
    PrintItems {
        #[arg(short = 'f', long, default_value = "basic")]
        format: PrintFormat,
        #[command(flatten)]
        find_items: FindItems,
    },
    /// command for testing
    #[cfg(debug_assertions)]
    Debug,
}

#[derive(Args, Debug)]
struct FindItems {
    /// the directory containing item definitions
    #[arg(short = 'p', long, default_value = "./data/item_definitions")]
    items_path: PathBuf,
    /// the regular expression to match against item names
    /// can be specified multiple times to match against any of the given patterns
    #[arg(short = 'r', long, visible_alias("pattern"), num_args(0..))]
    regex_pattern: Vec<String>,
}

#[derive(Debug, Clone, ValueEnum)]
enum PrintFormat {
    /// plain text
    Basic,
    /// json format
    Json,
}

fn main() {
    let cli = Cli::parse();

    initialize_logging(cli.log_level);

    let result = match &cli.command {
        Commands::Debug => debug(),
        Commands::PrintItems { format, find_items } => print_items(format, find_items),
    };

    match result {
        Ok(_) => println!("done, command executed successfully!"),
        Err(e) => {
            println!("command execution failed:\nerror: {0}\nsource: {1:#?}\nroot cause: {2}\nbacktrace: {3}", e, e.source(), e.root_cause(), e.backtrace());
            exit(1);
        }
    }
}

fn print_items(format: &PrintFormat, item_search: &FindItems) -> Result<()> {
    let items = fetch_items(item_search)?;
    let mut sorted_items = items.iter().collect_vec();
    sorted_items.sort_by_key(|i| i.id);
    for item in sorted_items {
        println!("{}", print_item(format, &item));
    }
    Ok(())
}

fn print_item(format: &PrintFormat, item: &ItemDefinition) -> String {
    match format {
        PrintFormat::Basic => format!(
            "{0} | {1} | {2}",
            item.id,
            item.name.as_deref().unwrap_or(""),
            item.description.as_deref().unwrap_or("")
        ),
        PrintFormat::Json => todo!(),
    }
}

fn fetch_items(find_items: &FindItems) -> Result<HashSet<ItemDefinition>> {
    let data_dir = find_items.items_path.as_path();
    let patterns = &find_items
        .regex_pattern
        .iter()
        .map(|p| match Regex::new(p) {
            Ok(regex) => regex,
            Err(e) => {
                log::error!("could not parse pattern as regular expression: {p} due to error {e}");
                panic!("")
            }
        })
        .collect_vec();

    log::info!("searching items...");
    let mut items = HashSet::new();
    let filepaths = std::fs::read_dir(data_dir)?
        .map(|entry| entry.unwrap().path())
        .collect_vec();
    let pb = ProgressBar::new(filepaths.len().try_into().unwrap());
    for path in filepaths {
        if path.extension().map(|x| x.to_string_lossy().to_string()) == Some("json".to_string()) {
            let definition: ItemDefinition = serde_json::from_str(&fs::read_to_string(path)?)?;
            let name = &definition.name;
            for pattern in patterns {
                if pattern.is_match(name.as_deref().unwrap_or("")) {
                    items.insert(definition.clone());
                }
            }
        }
        pb.inc(1);
    }

    Ok(items)
}

fn debug() -> Result<()> {
    Ok(())
}
