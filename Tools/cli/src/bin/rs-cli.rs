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
    #[arg(short = 'r', long, visible_alias("pattern"), num_args(0..), verbatim_doc_comment)]
    regex_pattern: Vec<String>,
    /// the path to a JSON file containing an array of item ids
    #[arg(short = 'i', long, num_args(0..))]
    ids_json: Vec<PathBuf>,
    /// the path to a JSON file containing an array of item ids and names as tuples
    #[arg(short = 'n', long, num_args(0..))]
    id_name_tuples_json: Vec<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum)]
enum PrintFormat {
    /// plain text
    Basic,
    /// array of item ids in JSON format
    JsonId,
    /// array of item ids and names in JSON format
    JsonIdNameTuple,
}

fn main() {
    let cli = Cli::parse();

    initialize_logging(cli.log_level);

    let result = match &cli.command {
        Commands::Debug => debug(),
        Commands::PrintItems { format, find_items } => print_items(format, find_items),
    };

    match result {
        Ok(_) => log::info!("done, command executed successfully!"),
        Err(e) => {
            log::error!("command execution failed:\nerror: {0}\nsource: {1:#?}\nroot cause: {2}\nbacktrace: {3}", e, e.source(), e.root_cause(), e.backtrace());
            exit(1);
        }
    }
}

fn print_items(format: &PrintFormat, item_search: &FindItems) -> Result<()> {
    let items = fetch_items(item_search)?;
    let mut sorted_items = items.iter().collect_vec();
    sorted_items.sort_by_key(|i| i.id);

    let s = match format {
        PrintFormat::Basic => sorted_items
            .iter()
            .map(|item| {
                format!(
                    "{0} | {1} | {2}",
                    item.id,
                    item.name.as_deref().unwrap_or(""),
                    item.description.as_deref().unwrap_or("")
                )
            })
            .collect_vec()
            .join("\n"),
        PrintFormat::JsonId => {
            serde_json::to_string(&sorted_items.iter().map(|item| item.id).collect_vec())?
        }
        PrintFormat::JsonIdNameTuple => serde_json::to_string(
            &sorted_items
                .iter()
                .map(|item| (item.id, item.name.as_deref().unwrap_or("unnamed")))
                .collect_vec(),
        )?,
    };

    println!("{s}");
    Ok(())
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

    let mut desired_ids = HashSet::new();
    for json_path in &find_items.ids_json {
        let ids: Vec<i32> = serde_json::from_str(&fs::read_to_string(json_path)?)?;
        desired_ids.extend(ids);
    }

    for json_path in &find_items.id_name_tuples_json {
        let tuples: Vec<(i32, String)> = serde_json::from_str(&fs::read_to_string(json_path)?)?;
        let ids: Vec<i32> = tuples.iter().map(|t| t.0).collect();
        desired_ids.extend(ids);
    }

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
            // match against regex patterns
            for pattern in patterns {
                if pattern.is_match(name.as_deref().unwrap_or("")) {
                    items.insert(definition.clone());
                }
            }
            // match against ids from id jsons
            if desired_ids.contains(&definition.id) {
                items.insert(definition.clone());
            }
        }
        pb.inc(1);
    }

    Ok(items)
}

fn debug() -> Result<()> {
    Ok(())
}
