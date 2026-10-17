use crate::{graph, knowledge, packs, workspace};
use serde::Serialize;
use std::path::PathBuf;

pub fn run(arguments: Vec<String>) -> Result<(), String> {
    if arguments.is_empty()
        || matches!(
            arguments.first().map(String::as_str),
            Some("--help") | Some("-h")
        )
    {
        print_help();
        return Ok(());
    }
    if arguments == ["--version"] || arguments == ["version"] {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    match arguments[0].as_str() {
        "init" => init(&arguments[1..]),
        "mode" => mode(&arguments[1..]),
        "graph" => graph_command(&arguments[1..]),
        "knowledge" => knowledge_command(&arguments[1..]),
        "pack" => pack(&arguments[1..]),
        command => Err(format!("unknown command: {command}")),
    }
}

fn graph_command(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "output"])?;
    if parsed.positionals.len() > 1 {
        return Err("usage: gibo graph [workspace] [--output PATH] [--json]".into());
    }
    let root = PathBuf::from(
        parsed
            .positionals
            .first()
            .map(String::as_str)
            .unwrap_or("."),
    );
    workspace::require_workspace(&root)?;
    // Resolve a relative output before passing it to the renderer so its result is stable.
    let owned_output = parsed.value("output").map(|value| {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            path
        } else {
            root.join(path)
        }
    });
    let result =
        graph::render(&root, owned_output.as_deref()).map_err(|error| error.to_string())?;
    emit(
        &result,
        parsed.json,
        Some(format!(
            "Rendered {} nodes and {} explicit relationships to {} ({} unresolved, {} skipped)",
            result.nodes,
            result.edges,
            result.output,
            result.unresolved_relationships,
            result.skipped_files
        )),
    )
}

fn knowledge_command(arguments: &[String]) -> Result<(), String> {
    if arguments.is_empty()
        || matches!(
            arguments.first().map(String::as_str),
            Some("--help") | Some("-h")
        )
    {
        print_knowledge_help();
        return Ok(());
    }
    let (command, rest) = arguments
        .split_first()
        .expect("knowledge arguments were checked above");
    match command.as_str() {
        "list" => knowledge_list(rest),
        "search" => knowledge_search(rest),
        "read" => knowledge_read(rest),
        "check" => knowledge_check(rest),
        "neighbors" => knowledge_neighbors(rest),
        "health" => knowledge_health(rest),
        other => Err(format!("unknown knowledge command: {other}")),
    }
}

fn knowledge_read(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace", "section"])?;
    let id = one_positional(
        &parsed,
        "usage: gibo knowledge read <id> [--section NAME] [--workspace PATH] [--json]",
    )?;
    let root = PathBuf::from(parsed.value("workspace").unwrap_or("."));
    let result = knowledge::read(&root, id, parsed.value("section"))?;
    if parsed.json {
        #[derive(Serialize)]
        struct ReadOutput<'a> {
            document: &'a knowledge::Document,
            section: &'a Option<knowledge::Section>,
            content: &'a str,
        }
        emit(
            &ReadOutput {
                document: &result.document,
                section: &result.section,
                content: &result.document.content,
            },
            true,
            None,
        )
    } else if let Some(section) = result.section {
        println!("{}", section.text);
        Ok(())
    } else {
        print!("{}", result.document.content);
        Ok(())
    }
}

fn knowledge_list(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace", "kind"])?;
    no_positionals(
        &parsed,
        "usage: gibo knowledge list [--kind KIND] [--workspace PATH] [--json]",
    )?;
    let root = PathBuf::from(parsed.value("workspace").unwrap_or("."));
    let mut documents = knowledge::catalog_local(&root);
    if let Some(kind) = parsed.value("kind") {
        if !knowledge::KNOWLEDGE_KINDS.contains(&kind) {
            return Err(format!("unknown knowledge kind: {kind}"));
        }
        documents.retain(|document| document.kind == kind);
    }
    if parsed.json {
        emit(&documents, true, None)
    } else if documents.is_empty() {
        println!("No Gibo knowledge records found.");
        Ok(())
    } else {
        for document in documents {
            println!(
                "{}\t{}\t{}\t{}",
                document.kind, document.id, document.as_of, document.title
            );
        }
        Ok(())
    }
}

fn knowledge_search(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse_repeatable(
        arguments,
        &["json", "workspace", "kind", "entity", "limit", "candidates"],
        &["kind"],
    )?;
    let query = one_positional(&parsed, "usage: gibo knowledge search <query> [--kind KIND]... [--entity ID] [--limit N] [--candidates] [--workspace PATH] [--json]")?;
    let root = PathBuf::from(parsed.value("workspace").unwrap_or("."));
    let kinds = parsed.values("kind");
    let limit = parsed
        .value("limit")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "--limit must be an integer".to_string())
        })
        .transpose()?
        .unwrap_or(8);
    if parsed.has_flag("candidates") {
        let results = knowledge::search_candidates_with_kinds(
            &root,
            query,
            &kinds,
            parsed.value("entity"),
            limit,
        )?;
        if parsed.json {
            return emit(&results, true, None);
        }
        if results.is_empty() {
            println!("No matching Gibo knowledge records found.");
            return Ok(());
        }
        for result in results {
            println!(
                "{}\t{}\t{}\t{}\t{}",
                result.score,
                result.id,
                result.match_tier.as_str(),
                result.section,
                result.title
            );
            println!("{}", result.summary);
        }
        return Ok(());
    }
    let results =
        knowledge::search_with_kinds(&root, query, &kinds, parsed.value("entity"), limit)?;
    if parsed.json {
        emit(&results, true, None)
    } else if results.is_empty() {
        println!("No matching Gibo knowledge records found.");
        Ok(())
    } else {
        for result in results {
            println!(
                "{}\t{}\t{}\t{}",
                result.score, result.id, result.section, result.as_of
            );
            if !result.relationships.is_empty() {
                println!(
                    "relationships\t{}",
                    result
                        .relationships
                        .iter()
                        .map(|relationship| format!(
                            "{}:{}",
                            relationship.kind, relationship.target
                        ))
                        .collect::<Vec<_>>()
                        .join("\t")
                );
            }
            println!("{}", result.text);
        }
        Ok(())
    }
}

fn knowledge_neighbors(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace"])?;
    let id = one_positional(
        &parsed,
        "usage: gibo knowledge neighbors <wiki-id> [--workspace PATH] [--json]",
    )?;
    let result =
        knowledge::neighbors(&PathBuf::from(parsed.value("workspace").unwrap_or(".")), id)?;
    if parsed.json {
        emit(&result, true, None)
    } else {
        println!(
            "{}\t{}\t{}",
            result.document.id, result.document.as_of, result.document.title
        );
        for document in result.outgoing {
            println!("outgoing\t{}\t{}", document.id, document.title);
        }
        for document in result.incoming {
            println!("incoming\t{}\t{}", document.id, document.title);
        }
        Ok(())
    }
}

fn knowledge_health(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace", "kind"])?;
    no_positionals(
        &parsed,
        "usage: gibo knowledge health [--kind KIND] [--workspace PATH] [--json]",
    )?;
    let result = knowledge::health(
        &PathBuf::from(parsed.value("workspace").unwrap_or(".")),
        parsed.value("kind"),
    )?;
    if parsed.json {
        emit(&result, true, None)
    } else {
        for health in result.kinds {
            println!(
                "{}\t{}\t{}\t{}\t{}",
                health.kind, health.documents, health.folders, health.max_depth, health.review_band
            );
            for advisory in health.advisories {
                println!("advisory\t{}\t{}", advisory.code, advisory.ids.join("\t"));
            }
        }
        Ok(())
    }
}

fn knowledge_check(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace"])?;
    no_positionals(
        &parsed,
        "usage: gibo knowledge check [--workspace PATH] [--json]",
    )?;
    let result = knowledge::check(&PathBuf::from(parsed.value("workspace").unwrap_or(".")));
    if parsed.json {
        emit(&result, true, None)?;
    } else if result.valid {
        println!("Valid Gibo knowledge: {} documents", result.documents);
    } else {
        for issue in &result.errors {
            eprintln!("gibo: {}: {}: {}", issue.path, issue.field, issue.message);
        }
    }
    if result.valid {
        Ok(())
    } else {
        Err("knowledge check failed".into())
    }
}

fn init(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json"])?;
    if parsed.positionals.len() > 1 {
        return Err("usage: gibo init [workspace] [--json]".into());
    }
    let workspace = parsed
        .positionals
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let result = workspace::initialize_workspace(&workspace)?;
    emit(
        &result,
        parsed.json,
        Some(format!("Initialized Gibo in {}", result.root)),
    )
}

fn mode(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace"])?;
    if parsed.positionals.len() > 1 {
        return Err("usage: gibo mode [auto|off] [--workspace PATH] [--json]".into());
    }
    let root = PathBuf::from(parsed.value("workspace").unwrap_or("."));
    let result = if let Some(mode) = parsed.positionals.first() {
        workspace::set_record_mode(&root, mode)?
    } else {
        workspace::read_record_mode(&root)?
    };
    emit(
        &result,
        parsed.json,
        Some(format!("Record Mode: {}", result.mode)),
    )
}

fn pack(arguments: &[String]) -> Result<(), String> {
    let (command, rest) = arguments
        .split_first()
        .ok_or_else(|| "usage: gibo pack <command>".to_string())?;
    if matches!(command.as_str(), "--help" | "-h") {
        print_pack_help();
        return Ok(());
    }
    if matches!(rest, [argument] if matches!(argument.as_str(), "--help" | "-h")) {
        return print_pack_command_help(command);
    }
    match command.as_str() {
        "install" => pack_install(rest),
        "list" => pack_list(rest),
        "inspect" => pack_inspect(rest),
        "update" => pack_update(rest),
        "remove" => pack_remove(rest),
        "create" => pack_create(rest),
        "check" => pack_check(rest),
        other => Err(format!("unknown pack command: {other}")),
    }
}

fn pack_install(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace", "ref"])?;
    let source = one_positional(
        &parsed,
        "usage: gibo pack install <github-url> [--ref REF] [--workspace PATH] [--json]",
    )?;
    let workspace = PathBuf::from(parsed.value("workspace").unwrap_or("."));
    workspace::require_workspace(&workspace)?;
    let info = packs::install_pack(&workspace, source, parsed.value("ref"))?;
    emit(
        &info,
        parsed.json,
        Some(format!(
            "Installed {} Pack from {source}; local memory was seeded in gibo/{}/",
            info.manifest.kind, info.manifest.kind
        )),
    )
}

fn pack_list(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace"])?;
    no_positionals(&parsed, "usage: gibo pack list [--workspace PATH] [--json]")?;
    let installed = packs::list_packs(&PathBuf::from(parsed.value("workspace").unwrap_or(".")))?;
    if parsed.json {
        emit(&installed, true, None)
    } else if installed.is_empty() {
        println!("No Gibo Packs are installed in this project.");
        Ok(())
    } else {
        for info in installed {
            let commit = info
                .commit
                .as_deref()
                .map(|value| &value[..value.len().min(12)])
                .unwrap_or("unversioned");
            println!(
                "{}\t{}\t{}\t{}",
                info.manifest.kind,
                commit,
                info.manifest.elements.len(),
                info.repository.as_deref().unwrap_or("no-origin")
            );
        }
        Ok(())
    }
}

fn pack_inspect(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace"])?;
    let kind = one_positional(
        &parsed,
        "usage: gibo pack inspect <wiki|lens|method> [--workspace PATH] [--json]",
    )?;
    let workspace = PathBuf::from(parsed.value("workspace").unwrap_or("."));
    let info = packs::inspect_pack_slot(&workspace, kind)?;
    emit(
        &info,
        parsed.json,
        Some(format!(
            "{} Pack at {} with {} elements",
            info.manifest.kind,
            info.commit.as_deref().unwrap_or("unknown commit"),
            info.manifest.elements.len()
        )),
    )
}

fn pack_update(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace", "ref"])?;
    let kind = one_positional(
        &parsed,
        "usage: gibo pack update <wiki|lens|method> [--ref REF] [--workspace PATH] [--json]",
    )?;
    let workspace = PathBuf::from(parsed.value("workspace").unwrap_or("."));
    let path = packs::pack_target(&workspace, kind)?;
    let before = packs::inspect_pack_slot(&workspace, kind)?;
    let after = packs::update_pack(&path, parsed.value("ref"))?;
    let before_commit = before.commit.as_deref().unwrap_or("unknown");
    let after_commit = after.commit.as_deref().unwrap_or("unknown");
    #[derive(Serialize)]
    struct Update<'a> {
        before: &'a packs::PackInfo,
        after: &'a packs::PackInfo,
    }
    emit(
        &Update {
            before: &before,
            after: &after,
        },
        parsed.json,
        Some(format!(
            "Updated {kind} Pack source: {} -> {}; local memory was not changed",
            before_commit, after_commit
        )),
    )
}

fn pack_remove(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "workspace"])?;
    let kind = one_positional(
        &parsed,
        "usage: gibo pack remove <wiki|lens|method> [--workspace PATH] [--json]",
    )?;
    let removed = packs::remove_pack(
        &PathBuf::from(parsed.value("workspace").unwrap_or(".")),
        kind,
    )?;
    #[derive(Serialize)]
    struct Removed<'a> {
        removed: &'a str,
    }
    emit(
        &Removed { removed: &removed },
        parsed.json,
        Some(format!(
            "Removed the {kind} Pack source; local gibo/{kind}/ memory was preserved."
        )),
    )
}

fn pack_create(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json", "element"])?;
    if parsed.positionals.len() != 2 {
        return Err("usage: gibo pack create <wiki|lens|method> <output-dir> --element PATH [--element PATH] [--json]".into());
    }
    let kind = &parsed.positionals[0];
    let output = PathBuf::from(&parsed.positionals[1]);
    let elements = parsed.values("element");
    let manifest = packs::create_pack(&output, kind, &elements)?;
    #[derive(Serialize)]
    struct Created<'a> {
        path: String,
        manifest: &'a packs::PackManifest,
    }
    emit(
        &Created {
            path: output.display().to_string(),
            manifest: &manifest,
        },
        parsed.json,
        Some(format!(
            "Created {} Pack with {} elements at {}",
            manifest.kind,
            manifest.elements.len(),
            output.display()
        )),
    )
}

fn pack_check(arguments: &[String]) -> Result<(), String> {
    let parsed = Options::parse(arguments, &["json"])?;
    let path = PathBuf::from(one_positional(
        &parsed,
        "usage: gibo pack check <path> [--json]",
    )?);
    let result = packs::check_pack(&path);
    if parsed.json {
        emit(&result, true, None)?;
    } else if result.valid {
        println!("Valid Gibo Pack: {}", result.path);
    } else {
        for error in &result.errors {
            eprintln!("gibo: {error}");
        }
    }
    if result.valid {
        Ok(())
    } else {
        Err("pack check failed".into())
    }
}

fn emit<T: Serialize>(value: &T, json: bool, message: Option<String>) -> Result<(), String> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(value).map_err(|error| error.to_string())?
        );
    } else if let Some(message) = message {
        println!("{message}");
    }
    Ok(())
}

struct Options {
    positionals: Vec<String>,
    named: Vec<(String, String)>,
    flags: Vec<String>,
    json: bool,
}

impl Options {
    fn parse(arguments: &[String], allowed: &[&str]) -> Result<Self, String> {
        Self::parse_repeatable(arguments, allowed, &["element"])
    }

    fn parse_repeatable(
        arguments: &[String],
        allowed: &[&str],
        repeatable: &[&str],
    ) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut named = Vec::new();
        let mut flags = Vec::new();
        let mut json = false;
        let mut index = 0;
        while index < arguments.len() {
            let argument = &arguments[index];
            if argument == "--json" {
                if !allowed.contains(&"json") || json {
                    return Err("--json may be supplied only once".into());
                }
                json = true;
            } else if argument == "--candidates" {
                if !allowed.contains(&"candidates") || flags.iter().any(|flag| flag == "candidates")
                {
                    return Err("--candidates may be supplied only once".into());
                }
                flags.push("candidates".into());
            } else if let Some(name) = argument.strip_prefix("--") {
                if !allowed.contains(&name) {
                    return Err(format!("unknown option: {argument}"));
                }
                if !repeatable.contains(&name) && named.iter().any(|(key, _)| key == name) {
                    return Err(format!("{argument} may be supplied only once"));
                }
                index += 1;
                let value = arguments
                    .get(index)
                    .ok_or_else(|| format!("{argument} requires a value"))?;
                if value.starts_with("--") {
                    return Err(format!("{argument} requires a value"));
                }
                named.push((name.into(), value.clone()));
            } else {
                positionals.push(argument.clone());
            }
            index += 1;
        }
        Ok(Self {
            positionals,
            named,
            flags,
            json,
        })
    }

    fn value(&self, name: &str) -> Option<&str> {
        self.named
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
    fn values(&self, name: &str) -> Vec<String> {
        self.named
            .iter()
            .filter(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
            .collect()
    }
    fn has_flag(&self, name: &str) -> bool {
        self.flags.iter().any(|flag| flag == name)
    }
}

fn one_positional<'a>(options: &'a Options, usage: &str) -> Result<&'a str, String> {
    if options.positionals.len() == 1 {
        Ok(&options.positionals[0])
    } else {
        Err(usage.into())
    }
}

fn no_positionals(options: &Options, usage: &str) -> Result<(), String> {
    if options.positionals.is_empty() {
        Ok(())
    } else {
        Err(usage.into())
    }
}

fn print_help() {
    println!(
        "gibo {}\n\nUSAGE:\n    gibo init [workspace] [--json]\n    gibo mode [auto|off] [--workspace PATH] [--json]\n    gibo knowledge <list|search|read|check|neighbors|health> ...\n    gibo graph [workspace] [--output PATH] [--json]\n    gibo pack <install|list|inspect|update|remove|create|check> ...",
        env!("CARGO_PKG_VERSION")
    );
}

fn print_pack_help() {
    println!("Gibo Pack commands: install, list, inspect, update, remove, create, check");
}

fn print_pack_command_help(command: &str) -> Result<(), String> {
    let usage = match command {
        "install" => {
            "gibo pack install <github-url> [--ref REF] [--workspace PATH] [--json]"
        }
        "list" => "gibo pack list [--workspace PATH] [--json]",
        "inspect" => {
            "gibo pack inspect <wiki|lens|method> [--workspace PATH] [--json]"
        }
        "update" => {
            "gibo pack update <wiki|lens|method> [--ref REF] [--workspace PATH] [--json]"
        }
        "remove" => {
            "gibo pack remove <wiki|lens|method> [--workspace PATH] [--json]"
        }
        "create" => "gibo pack create <wiki|lens|method> <output-dir> --element PATH [--element PATH] [--json]",
        "check" => "gibo pack check <path> [--json]",
        other => return Err(format!("unknown pack command: {other}")),
    };
    println!("Usage:\n    {usage}");
    Ok(())
}

fn print_knowledge_help() {
    println!(
        "Knowledge commands:\n    gibo knowledge search <query> [--kind KIND]... [--entity ID] [--limit N] [--candidates] [--workspace PATH] [--json]\n    gibo knowledge read <id> [--section NAME] [--workspace PATH] [--json]\n    gibo knowledge list [--kind KIND] [--workspace PATH] [--json]\n    gibo knowledge check [--workspace PATH] [--json]\n    gibo knowledge neighbors <wiki-id> [--workspace PATH] [--json]\n    gibo knowledge health [--kind KIND] [--workspace PATH] [--json]"
    );
}
