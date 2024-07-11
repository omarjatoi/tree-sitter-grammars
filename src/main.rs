use std::path::PathBuf;

use clap::{Parser, Subcommand};

use tree_sitter_grammars::add_language_grammar_to_toml;
use tree_sitter_grammars::update_language;
use tree_sitter_grammars::Language;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to file containing languages and their grammar repositories
    #[arg(short, long, default_value = "./languages.toml", global = true)]
    file: PathBuf,

    /// Path to directory containing grammar repositories
    #[arg(short, long, default_value = "./grammars/", global = true)]
    directory: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new tree-sitter grammar to the `languages.toml` file
    Add {
        /// Name of the language being added, e.g. 'rust'
        #[arg(short, long)]
        name: String,
        /// URL to the tree-sitter grammar, e.g. 'git@github.com:tree-sitter/tree-sitter-rust.git'
        #[arg(short, long)]
        git: String,
        /// Optional git hash to checkout from the grammar repository
        #[arg(long)]
        hash: Option<String>,
        /// Whether we want to compile the grammar to WebAssembly
        #[arg(short, long)]
        wasm: bool,
    },
    /// Fetch the tree-sitter grammar(s)
    Fetch {
        /// Name of the language grammar to update, e.g. 'rust'
        #[arg(short, long)]
        name: Option<String>,
        /// Use this flag to update all grammars for all languages listed
        #[arg(long, default_value_t = false)]
        all: bool,
        /// Whether we want to compile the grammar to WebAssembly
        #[arg(short, long)]
        wasm: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let dir = cli.directory;
    let file_path = cli.file;

    match &cli.command {
        Some(Commands::Add {
            name,
            git,
            hash,
            wasm,
        }) => {
            let tree_sitter_name = format!("{}{}", "tree-sitter-", name);
            let language = Language::new(tree_sitter_name, git.clone(), hash.clone());
            add_language_grammar_to_toml(name.clone(), language, file_path.clone());
            update_language(
                Some(name.clone()),
                false,
                wasm.clone(),
                file_path.clone(),
                dir,
            )
            .await;
        }
        Some(Commands::Fetch { name, all, wasm }) => {
            update_language(name.clone(), all.clone(), wasm.clone(), file_path, dir).await;
        }
        None => {}
    }
}
