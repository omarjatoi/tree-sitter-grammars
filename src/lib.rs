use futures::future::join_all;
use git2::Oid;
use git2::Repository;
use indicatif::{ProgressBar, ProgressStyle};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Language {
    pub name: String,
    pub git: String,
    pub hash: Option<String>,
}

impl Language {
    pub fn new(name: String, git: String, hash: Option<String>) -> Self {
        Self { name, git, hash }
    }
}

#[derive(Serialize, Deserialize)]
struct LanguageGrammarsTOML {
    languages: BTreeMap<String, Language>,
}

pub fn add_language_grammar_to_toml(name: String, language: Language, file_path: PathBuf) {
    let toml_contents = fs::read_to_string(&file_path).expect("Failed to read TOML file");
    let mut languages: LanguageGrammarsTOML =
        toml::from_str(&toml_contents).expect("Failed to parse TOML");

    if let Some(existing_language) = languages.languages.get_mut(&name) {
        if existing_language.hash != language.hash {
            existing_language.hash = language.hash;
        }
        if existing_language.git != language.git {
            existing_language.git = language.git;
        }
    } else {
        languages.languages.insert(name, language);
    }

    let comment =
        "# Automatically generated, DO NOT EDIT! Use `tree-sitter-grammars add` to modify.\n\n";

    let updated_toml = format!(
        "{}{}",
        comment,
        toml::to_string_pretty(&languages).expect("Failed to serialize to TOML")
    );
    fs::write(&file_path, updated_toml).expect("Failed to write updated TOML file");
}

pub async fn update_language(
    name: Option<String>,
    all: bool,
    wasm: bool,
    file_path: PathBuf,
    directory: PathBuf,
) -> () {
    if let Some(language_name) = name {
        let toml_contents = fs::read_to_string(&file_path).expect("Failed to read TOML file");
        let languages: LanguageGrammarsTOML =
            toml::from_str(&toml_contents).expect("Failed to parse TOML");

        if let Some(language) = languages.languages.get(&language_name) {
            let destination_directory = format!("{}{}", directory.display(), &language.name);
            clone_repository(language.clone(), destination_directory.clone()).await;

            // compiling to wasm is enabled
            if wasm {
                let target = format!("../../{}{}.wasm", "wasm/", language_name);
                Command::new("tree-sitter")
                    .current_dir(destination_directory.clone())
                    .arg("build")
                    .arg("--wasm")
                    .arg("-o")
                    .arg(target)
                    .spawn()
                    .expect("Failed to compile grammar to WebAssembly");
            }
        } else {
            eprintln!("Language not found: {}", language_name);
        }
    } else if all {
        println!("Updating all languages");
        let toml_contents = fs::read_to_string(&file_path).expect("Failed to read TOML file");
        let languages: LanguageGrammarsTOML =
            toml::from_str(&toml_contents).expect("Failed to parse TOML");

        let grammars_to_update: Vec<_> = languages
            .languages
            .iter()
            .map(|(_, language)| {
                let destination_directory = format!("{}{}", directory.display(), &language.name);
                (language.clone(), destination_directory)
            })
            .collect();

        let async_clones: Vec<_> = grammars_to_update
            .clone()
            .into_iter()
            .map(|(language, destination_directory)| {
                tokio::spawn(async move {
                    clone_repository(language.clone(), destination_directory.clone()).await
                })
            })
            .collect();

        for task in async_clones {
            if let Err(err) = task.await {
                eprintln!("Async task error: {:?}", err);
            }
        }

        let compile_grammars_to_wasm: Vec<_> = grammars_to_update
            .clone()
            .into_iter()
            .map(|(language, destination_directory)| {
                let wasm_file_path = format!("../../{}{}.wasm", "wasm/", language.name);
                tokio::spawn(async move {
                    let status = tokio::process::Command::new("tree-sitter")
                        .current_dir(destination_directory)
                        .arg("build")
                        .arg("--wasm")
                        .arg("-o")
                        .arg(wasm_file_path)
                        .status()
                        .await;
                    match status {
                        Ok(status) if status.success() => Ok(()),
                        Ok(status) => Err(format!(
                            "Command exited with status: {}, for language: {}",
                            status, language.name
                        )),
                        Err(e) => Err(format!(
                            "Failed to execute command: {}, for language: {}",
                            e, language.name
                        )),
                    }
                })
            })
            .collect();

        let results = join_all(compile_grammars_to_wasm).await;

        for result in results {
            match result {
                Ok(Ok(())) => (),
                Ok(Err(e)) => eprintln!("Error: {}", e),
                Err(e) => eprintln!("Join error: {:?}", e),
            }
        }
    } else {
        eprintln!("Please provide a language name or use the --all option.");
    }
}

async fn clone_repository(language: Language, directory: String) {
    let progress = ProgressBar::new_spinner();
    progress.set_style(ProgressStyle::default_spinner().tick_strings(&["-", "\\", "|", "/"]));
    progress.set_message(format!("Updating {}", language.name));

    if let Err(e) = fs::remove_dir_all(&directory) {
        if e.kind() != std::io::ErrorKind::NotFound {
            progress.finish_with_message(format!("Failed update {}: {:?}", language.name, e));
            return;
        }
    }
    match Repository::clone(&language.git, Path::new(&directory)) {
        Ok(repo) => {
            if let Some(commit_hash) = language.hash {
                let _ = repo.set_head_detached(
                    Oid::from_str(&commit_hash)
                        .ok()
                        .expect("Failed to checkout the specific commit"),
                );
            }
            let git_folder = format!("{}/{}", &directory, ".git");

            fs::remove_dir_all(&git_folder).expect("Could not remove .git folder");
            progress.finish_with_message(format!("Successfully updated {}", language.name));
        }
        Err(e) => {
            progress.finish_with_message(format!(
                "Failed to update {}: {:?}",
                language.name,
                e.message()
            ));
        }
    }
}
