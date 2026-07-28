//! File-native knowledge records. Markdown is canonical; no index is kept.

use chrono::DateTime;
use regex::Regex;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub const RELATIONSHIP_TYPES: &[&str] = &["uses", "supports", "contradicts", "updates"];
const AUTHORED_RELATIONSHIP_TYPES: &[&str] =
    &["uses", "supports", "contradicts", "updates", "related"];
pub const KNOWLEDGE_KINDS: &[&str] = &["lens", "wiki", "evidence", "method", "decision"];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frontmatter {
    pub scalar: BTreeMap<String, String>,
    pub lists: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Document {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub as_of: String,
    pub path: String,
    pub entities: Vec<String>,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub source: Option<String>,
    pub published_at: Option<String>,
    pub period: Option<String>,
    pub origin: String,
    pub mutable: bool,
    pub relationships: Vec<Relationship>,
    #[serde(skip_serializing)]
    pub content: String,
    #[serde(skip_serializing)]
    pub body: String,
    #[serde(skip_serializing)]
    pub read_error: Option<String>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, Ord, PartialOrd)]
pub struct Relationship {
    #[serde(rename = "type")]
    pub kind: String,
    pub target: String,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Section {
    pub heading: String,
    pub heading_path: Vec<String>,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SearchResult {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub as_of: String,
    pub path: String,
    pub section: String,
    pub heading_path: Vec<String>,
    pub entities: Vec<String>,
    pub origin: String,
    pub mutable: bool,
    pub relationships: Vec<Relationship>,
    pub score: i32,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct KnowledgeIssue {
    pub path: String,
    pub field: String,
    pub message: String,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct KnowledgeCheck {
    pub valid: bool,
    pub documents: usize,
    pub errors: Vec<KnowledgeIssue>,
}

pub fn infer_kind(path: &Path, root: &Path) -> String {
    let base = root.join("guru-maker");
    path.strip_prefix(&base)
        .ok()
        .and_then(|p| p.components().next())
        .and_then(|p| match p.as_os_str().to_str()? {
            "lens" => Some("lens"),
            "wiki" => Some("wiki"),
            "evidence" => Some("evidence"),
            "method" => Some("method"),
            "decision" => Some("decision"),
            _ => None,
        })
        .unwrap_or("document")
        .to_owned()
}

pub fn catalog_local(root: &Path) -> Vec<Document> {
    let mut docs: Vec<_> = local_markdown_files(root)
        .into_iter()
        .map(|p| load_local(root, &p))
        .collect();
    docs.sort_by(|a, b| (&a.kind, &a.id).cmp(&(&b.kind, &b.id)));
    docs
}
pub fn search(
    root: &Path,
    query: &str,
    kind: Option<&str>,
    entity: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Err("knowledge search query must not be empty".into());
    }
    if let Some(k) = kind {
        if !KNOWLEDGE_KINDS.contains(&k) {
            return Err("unknown knowledge kind".into());
        }
    }
    if !(1..=50).contains(&limit) {
        return Err("knowledge search limit must be between 1 and 50".into());
    }
    let docs = catalog_local(root);
    let wanted_entity = entity.map(str::to_ascii_lowercase);
    let mut out = Vec::new();
    for doc in docs {
        if kind.is_some_and(|k| doc.kind != k)
            || wanted_entity
                .as_ref()
                .is_some_and(|e| !doc.entities.iter().any(|x| x.eq_ignore_ascii_case(e)))
        {
            continue;
        }
        for section in split_sections(&doc.body) {
            let score = section_score(&doc, &section, query);
            if score > 0 {
                out.push(SearchResult {
                    id: doc.id.clone(),
                    kind: doc.kind.clone(),
                    title: doc.title.clone(),
                    summary: doc.summary.clone(),
                    as_of: doc.as_of.clone(),
                    path: doc.path.clone(),
                    section: section.heading.clone(),
                    heading_path: section.heading_path.clone(),
                    entities: doc.entities.clone(),
                    origin: doc.origin.clone(),
                    mutable: doc.mutable,
                    relationships: doc.relationships.clone(),
                    score,
                    text: contextual(&doc, &section),
                });
            }
        }
    }
    out.sort_by(search_result_order);
    let mut seen = BTreeSet::new();
    out.retain(|result| seen.insert(result.id.clone()));
    out.truncate(limit);
    Ok(out)
}
pub fn check(root: &Path) -> KnowledgeCheck {
    let docs = catalog_local(root);
    let mut issues = layout_issues(root);
    let mut ids = BTreeMap::<String, String>::new();
    for doc in &docs {
        for issue in validate_document(doc) {
            issues.push(issue);
        }
        if !doc.id.is_empty() {
            if let Some(first) = ids.insert(doc.id.clone(), doc.path.clone()) {
                issues.push(KnowledgeIssue {
                    path: doc.path.clone(),
                    field: "id".into(),
                    message: format!("duplicates {first}"),
                });
            }
        }
    }
    let kinds = docs
        .iter()
        .filter(|doc| !doc.id.is_empty())
        .map(|doc| (doc.id.as_str(), doc.kind.as_str()))
        .collect::<BTreeMap<_, _>>();
    for doc in &docs {
        for relationship in &doc.relationships {
            let Some(target_kind) = kinds.get(relationship.target.as_str()) else {
                issues.push(issue(
                    doc,
                    &relationship.kind,
                    &format!("target does not exist: {}", relationship.target),
                ));
                continue;
            };
            if !relationship_target_allowed(&doc.kind, &relationship.kind, target_kind) {
                issues.push(issue(
                    doc,
                    &relationship.kind,
                    &format!(
                        "target {} has disallowed kind {}",
                        relationship.target, target_kind
                    ),
                ));
            }
        }
    }
    KnowledgeCheck {
        valid: issues.is_empty(),
        documents: docs.len(),
        errors: issues,
    }
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReadResult {
    pub document: Document,
    pub section: Option<Section>,
}
pub fn read(root: &Path, id: &str, section: Option<&str>) -> Result<ReadResult, String> {
    let document = catalog_local(root)
        .into_iter()
        .find(|doc| doc.id == id)
        .ok_or_else(|| format!("knowledge document not found: {id}"))?;
    let selected = if let Some(wanted) = section {
        let wanted = wanted.to_ascii_lowercase();
        let matches: Vec<_> = split_sections(&document.body)
            .into_iter()
            .filter(|item| {
                item.heading.eq_ignore_ascii_case(&wanted)
                    || item.heading_path.join(" / ").eq_ignore_ascii_case(&wanted)
            })
            .collect();
        match matches.len() {
            0 => return Err(format!("section not found in {id}: {section:?}")),
            1 => matches.into_iter().next(),
            _ => return Err(format!("section is ambiguous in {id}: {section:?}")),
        }
    } else {
        None
    };
    Ok(ReadResult {
        document,
        section: selected,
    })
}
pub fn split_sections(body: &str) -> Vec<Section> {
    let mut out = Vec::new();
    let mut stack: Vec<(usize, String)> = vec![];
    let mut heading = String::new();
    let mut path = vec![];
    let mut buf = Vec::new();
    let flush = |out: &mut Vec<Section>, heading: &str, path: &[String], buf: &mut Vec<&str>| {
        let text = buf.join("\n").trim().to_owned();
        if !text.is_empty() || !heading.is_empty() || out.is_empty() {
            out.push(Section {
                heading: heading.into(),
                heading_path: path.into(),
                text,
            });
        }
        buf.clear();
    };
    for line in body.lines() {
        let hashes = line.chars().take_while(|c| *c == '#').count();
        if hashes > 0 && line.as_bytes().get(hashes) == Some(&b' ') {
            flush(&mut out, &heading, &path, &mut buf);
            while stack.last().is_some_and(|(level, _)| *level >= hashes) {
                stack.pop();
            }
            let name = line[hashes + 1..].trim().to_owned();
            stack.push((hashes, name.clone()));
            heading = name;
            path = stack.iter().map(|(_, n)| n.clone()).collect();
        } else {
            buf.push(line);
        }
    }
    flush(&mut out, &heading, &path, &mut buf);
    out.into_iter()
        .filter(|s| !s.text.is_empty() || !s.heading.is_empty())
        .collect()
}

pub fn is_rfc3339_seconds(value: &str) -> bool {
    Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:Z|[+-]\d{2}:\d{2})$")
        .unwrap()
        .is_match(value)
        && DateTime::parse_from_rfc3339(value).is_ok()
}
pub fn document_is_valid(doc: &Document) -> bool {
    validate_document(doc).is_empty()
}

pub fn validation_issues(doc: &Document) -> Vec<KnowledgeIssue> {
    validate_document(doc)
}

pub fn load_pack_element(path: &Path, kind: &str) -> Result<Document, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("cannot read Markdown {}: {error}", path.display()))?;
    if split_frontmatter(&content).is_none() {
        return Err(format!(
            "{}: frontmatter must begin and end with ---",
            path.display()
        ));
    }
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    load_document(root, path, Some(kind.to_owned()))
        .map_err(|error| format!("cannot read Markdown {}: {error}", path.display()))
}

fn split_frontmatter(markdown: &str) -> Option<(Frontmatter, String)> {
    let mut lines = markdown.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut meta = Frontmatter::default();
    let mut current = None;
    let mut body = Vec::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if !line.chars().next().is_some_and(char::is_whitespace) {
            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim().to_owned();
                let value = val.trim();
                current = None;
                if list_field(&key) {
                    let values = inline_list(value);
                    if value.is_empty() {
                        current = Some(key.clone());
                    }
                    meta.lists.insert(key, values);
                } else {
                    meta.scalar.insert(key, clean(value));
                }
            }
        } else if let Some(key) = &current {
            if let Some(value) = line.trim().strip_prefix("- ") {
                meta.lists
                    .entry(key.clone())
                    .or_default()
                    .push(clean(value));
            }
        }
    }
    if !closed {
        return None;
    }
    body.extend(lines);
    Some((meta, body.join("\n")))
}
fn load_local(root: &Path, path: &Path) -> Document {
    match load_document(root, path, None) {
        Ok(doc) => doc,
        Err(error) => unreadable_document(root, path, error.to_string()),
    }
}
fn load_document(
    root: &Path,
    path: &Path,
    forced_kind: Option<String>,
) -> Result<Document, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let parsed = split_frontmatter(&content);
    let frontmatter_error = parsed
        .is_none()
        .then(|| "frontmatter must begin and end with ---".to_owned());
    let (m, body) = parsed.unwrap_or_default();
    let relationships = AUTHORED_RELATIONSHIP_TYPES
        .iter()
        .flat_map(|kind| {
            m.lists
                .get(*kind)
                .into_iter()
                .flatten()
                .map(move |target| Relationship {
                    kind: (*kind).into(),
                    target: target.clone(),
                })
        })
        .collect();
    Ok(Document {
        id: m.scalar.get("id").cloned().unwrap_or_default(),
        kind: forced_kind.unwrap_or_else(|| infer_kind(path, root)),
        title: m.scalar.get("title").cloned().unwrap_or_default(),
        summary: m.scalar.get("summary").cloned().unwrap_or_default(),
        as_of: m.scalar.get("as_of").cloned().unwrap_or_default(),
        path: display_path(path, root),
        entities: m.lists.get("entities").cloned().unwrap_or_default(),
        aliases: m.lists.get("aliases").cloned().unwrap_or_default(),
        tags: m.lists.get("tags").cloned().unwrap_or_default(),
        source: m.scalar.get("source").cloned(),
        published_at: m.scalar.get("published_at").cloned(),
        period: m.scalar.get("period").cloned(),
        origin: "local".into(),
        mutable: true,
        relationships,
        content,
        body,
        read_error: frontmatter_error,
    })
}
fn unreadable_document(root: &Path, path: &Path, message: String) -> Document {
    Document {
        id: String::new(),
        kind: infer_kind(path, root),
        title: String::new(),
        summary: String::new(),
        as_of: String::new(),
        path: display_path(path, root),
        entities: vec![],
        aliases: vec![],
        tags: vec![],
        source: None,
        published_at: None,
        period: None,
        origin: "local".into(),
        mutable: true,
        relationships: vec![],
        content: String::new(),
        body: String::new(),
        read_error: Some(format!("cannot read Markdown: {message}")),
    }
}
fn validate_document(doc: &Document) -> Vec<KnowledgeIssue> {
    let mut out = vec![];
    if let Some(error) = &doc.read_error {
        out.push(issue(doc, "frontmatter", error));
    }
    for (field, value) in [
        ("id", &doc.id),
        ("title", &doc.title),
        ("summary", &doc.summary),
        ("as_of", &doc.as_of),
    ] {
        if value.trim().is_empty() {
            out.push(issue(doc, field, "is required"));
        }
    }
    if !doc.as_of.is_empty() && !is_rfc3339_seconds(&doc.as_of) {
        out.push(issue(
            doc,
            "as_of",
            "must be RFC3339 with seconds and timezone",
        ));
    }
    if doc.kind == "evidence" && doc.source.as_deref().is_none_or(|s| s.trim().is_empty()) {
        out.push(issue(
            doc,
            "source",
            "evidence requires a non-empty source locator",
        ));
    }
    for relationship in &doc.relationships {
        let allowed = match doc.kind.as_str() {
            "evidence" => matches!(relationship.kind.as_str(), "updates" | "contradicts"),
            "decision" => matches!(
                relationship.kind.as_str(),
                "uses" | "supports" | "updates" | "contradicts"
            ),
            _ => false,
        };
        if !allowed {
            out.push(issue(
                doc,
                &relationship.kind,
                "relationship is not permitted for this memory kind",
            ));
        }
    }
    out
}
fn issue(doc: &Document, field: &str, message: &str) -> KnowledgeIssue {
    KnowledgeIssue {
        path: doc.path.clone(),
        field: field.into(),
        message: message.into(),
    }
}

pub fn relationship_target_allowed(source: &str, relationship: &str, target: &str) -> bool {
    match (source, relationship) {
        ("evidence", "updates" | "contradicts") => target == "evidence",
        ("decision", "uses") => matches!(target, "wiki" | "lens" | "method"),
        ("decision", "supports") => target == "evidence",
        ("decision", "updates" | "contradicts") => target == "decision",
        _ => false,
    }
}

fn layout_issues(root: &Path) -> Vec<KnowledgeIssue> {
    let mut issues = Vec::new();
    for kind in KNOWLEDGE_KINDS {
        let directory = root.join("guru-maker").join(kind);
        if let Ok(metadata) = fs::symlink_metadata(&directory) {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                issues.push(KnowledgeIssue {
                    path: display_path(&directory, root),
                    field: "layout".into(),
                    message: format!("{kind} collection must be a non-symlink directory"),
                });
                continue;
            }
        }
        collect_layout_issues(root, kind, &directory, &mut issues);
    }
    issues
}

fn collect_layout_issues(
    root: &Path,
    kind: &str,
    directory: &Path,
    issues: &mut Vec<KnowledgeIssue>,
) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let relative = display_path(&path, root);
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            issues.push(KnowledgeIssue {
                path: relative,
                field: "layout".into(),
                message: format!("{kind} elements and directories must not be symlinks"),
            });
        } else if metadata.is_dir() {
            collect_layout_issues(root, kind, &path, issues);
        } else if !metadata.is_file() || !is_markdown(&path) {
            issues.push(KnowledgeIssue {
                path: relative,
                field: "layout".into(),
                message: format!("{kind} elements must be non-symlink Markdown files"),
            });
        }
    }
}

fn local_markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![];
    for dir in ["wiki", "lens", "method", "evidence", "decision"] {
        collect_markdown_files(&root.join("guru-maker").join(dir), &mut out);
    }
    out.sort();
    out
}

fn collect_markdown_files(directory: &Path, out: &mut Vec<PathBuf>) {
    let Ok(metadata) = fs::symlink_metadata(directory) else {
        return;
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_markdown_files(&path, out);
        } else if metadata.is_file() && is_markdown(&path) {
            out.push(path);
        }
    }
}

fn is_markdown(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("md") | Some("markdown")
    )
}
fn list_field(key: &str) -> bool {
    matches!(key, "entities" | "aliases" | "tags") || AUTHORED_RELATIONSHIP_TYPES.contains(&key)
}
fn inline_list(value: &str) -> Vec<String> {
    let value = clean(value);
    if value == "[]" {
        vec![]
    } else if value.starts_with('[') && value.ends_with(']') {
        value[1..value.len() - 1]
            .split(',')
            .map(clean)
            .filter(|x| !x.is_empty())
            .collect()
    } else if value.is_empty() {
        vec![]
    } else {
        vec![value]
    }
}
fn clean(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].trim().into()
    } else {
        value.into()
    }
}
fn tokens(value: &str) -> BTreeSet<String> {
    let identifiers = Regex::new(r"[\w:./+@#-]+").unwrap();
    let components = Regex::new(r"\w+").unwrap();
    let mut out = BTreeSet::new();
    for identifier in identifiers.find_iter(value) {
        let value = identifier.as_str().to_ascii_lowercase();
        out.insert(value.clone());
        out.extend(
            components
                .find_iter(&value)
                .map(|component| component.as_str().to_owned()),
        );
    }
    out
}
fn section_score(doc: &Document, section: &Section, query: &str) -> i32 {
    let q = query.to_ascii_lowercase();
    let terms = tokens(query);
    let aliases = doc.aliases.join(" ");
    let entities = doc.entities.join(" ");
    let tags = doc.tags.join(" ");
    let relationships = doc
        .relationships
        .iter()
        .map(|relationship| relationship.target.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let period = doc.period.as_deref().unwrap_or_default();
    let mut score = 0;
    for (v, w) in [
        (doc.id.as_str(), 14),
        (doc.title.as_str(), 12),
        (aliases.as_str(), 12),
        (entities.as_str(), 12),
        (tags.as_str(), 8),
        (period, 8),
        (doc.summary.as_str(), 7),
        (section.heading.as_str(), 8),
        (relationships.as_str(), 4),
        (section.text.as_str(), 4),
    ] {
        if v.to_ascii_lowercase().contains(&q) {
            score += w
        } else {
            score += tokens(v).intersection(&terms).count() as i32 * w / 2
        }
    }
    score
}
fn search_result_order(a: &SearchResult, b: &SearchResult) -> std::cmp::Ordering {
    b.score
        .cmp(&a.score)
        .then_with(|| search_timestamp(&b.as_of).cmp(&search_timestamp(&a.as_of)))
        .then_with(|| a.id.cmp(&b.id))
        .then_with(|| a.heading_path.cmp(&b.heading_path))
}
fn search_timestamp(value: &str) -> i64 {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.timestamp())
        .unwrap_or(i64::MIN)
}
fn contextual(doc: &Document, section: &Section) -> String {
    format!(
        "{}\n\n{}",
        [doc.title.as_str(), doc.summary.as_str(), doc.as_of.as_str()].join("\n"),
        section.text
    )
    .trim()
    .into()
}
fn display_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "guru-maker-knowledge-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ))
    }
    fn write(root: &Path, relative: &str, value: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, value).unwrap();
    }
    #[test]
    fn check_reports_required_time_evidence_and_duplicates() {
        let root = root();
        write(
            &root,
            "guru-maker/lens/a.md",
            "---\nid: duplicate\ntitle: A\nsummary: A\nas_of: 2026-01-01T00:00Z\n---",
        );
        write(
            &root,
            "guru-maker/method/b.md",
            "---\nid: duplicate\ntitle: B\nsummary: B\nas_of: 2026-01-01T00:00:00Z\n---",
        );
        write(
            &root,
            "guru-maker/evidence/e.md",
            "---\nid: e\ntitle: E\nsummary: E\nas_of: 2026-01-01T00:00:00Z\n---",
        );
        write(&root, "guru-maker/wiki/plain.md", "# no frontmatter\n");
        let check = check(&root);
        assert!(!check.valid);
        assert!(check
            .errors
            .iter()
            .any(|i| i.field == "id" && i.message.contains("duplicates")));
        assert!(check.errors.iter().any(|i| i.field == "as_of"));
        assert!(check.errors.iter().any(|i| i.field == "source"));
        assert!(check.errors.iter().any(|i| i.field == "frontmatter"));
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn exact_read_selects_one_local_element_section() {
        let root = root();
        write(
            &root,
            "guru-maker/wiki/a.md",
            "---\nid: wiki:a\ntitle: A\nsummary: A\nas_of: 2026-01-01T00:00:00Z\n---\n# Chosen\nBody",
        );
        let result = read(&root, "wiki:a", Some("Chosen")).unwrap();
        assert_eq!(result.section.unwrap().text, "Body");
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn search_normalizes_compounds_and_scores_financial_metadata() {
        let root = root();
        write(
            &root,
            "guru-maker/evidence/cash-flow.md",
            "---\nid: evidence:cash-flow\ntitle: Cash generation\nsummary: Quarterly observation.\nas_of: 2026-07-27T09:00:00Z\nsource: https://example.com/report\nperiod: 2026-Q2\ntags:\n  - free-cash-flow\n---\n\n# Observation\nConversion improved.",
        );

        let compound = search(&root, "free cash flow", None, None, 5).unwrap();
        assert_eq!(compound.len(), 1);
        assert_eq!(compound[0].id, "evidence:cash-flow");

        let period = search(&root, "2026 Q2", None, None, 5).unwrap();
        assert_eq!(period.len(), 1);
        assert_eq!(period[0].id, "evidence:cash-flow");
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn search_returns_one_section_per_record_and_breaks_ties_by_as_of() {
        let root = root();
        write(
            &root,
            "guru-maker/wiki/older.md",
            "---\nid: wiki:older\ntitle: Older signal\nsummary: Cash flow signal.\nas_of: 2026-01-01T00:00:00Z\n---\n\n# First\nCash flow.\n\n# Second\nCash flow.",
        );
        write(
            &root,
            "guru-maker/wiki/newer.md",
            "---\nid: wiki:newer\ntitle: Newer signal\nsummary: Cash flow signal.\nas_of: 2026-07-01T00:00:00Z\n---\n\n# First\nCash flow.\n\n# Second\nCash flow.",
        );

        let results = search(&root, "cash flow signal", None, None, 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "wiki:newer");
        assert_eq!(results[1].id, "wiki:older");
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn catalog_does_not_follow_symlinked_files_directories_or_loops() {
        use std::os::unix::fs::symlink;

        let root = root();
        let outside = root.join("outside");
        write(
            &root,
            "outside/escaped.md",
            "---\nid: escaped\ntitle: Escaped\nsummary: Escaped\nas_of: 2026-01-01T00:00:00Z\n---",
        );
        let wiki = root.join("guru-maker/wiki");
        fs::create_dir_all(&wiki).unwrap();
        symlink(&outside, wiki.join("escaped-directory")).unwrap();
        symlink(outside.join("escaped.md"), wiki.join("escaped-file.md")).unwrap();
        symlink(&wiki, wiki.join("loop")).unwrap();
        fs::create_dir_all(root.join("guru-maker")).unwrap();
        symlink(&outside, root.join("guru-maker/lens")).unwrap();

        assert!(catalog_local(&root).is_empty());
        assert_eq!(check(&root).errors.len(), 4);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn check_enforces_the_minimal_relationship_matrix() {
        let root = root();
        write(
            &root,
            "guru-maker/wiki/w.md",
            "---\nid: wiki:w\ntitle: W\nsummary: W\nas_of: 2026-01-01T00:00:00Z\nuses:\n  - method:m\n---",
        );
        write(
            &root,
            "guru-maker/method/m.md",
            "---\nid: method:m\ntitle: M\nsummary: M\nas_of: 2026-01-01T00:00:00Z\n---",
        );
        write(
            &root,
            "guru-maker/evidence/e.md",
            "---\nid: evidence:e\ntitle: E\nsummary: E\nas_of: 2026-01-01T00:00:00Z\nsource: https://example.test/e\nupdates:\n  - method:m\n---",
        );
        write(
            &root,
            "guru-maker/decision/d.md",
            "---\nid: decision:d\ntitle: D\nsummary: D\nas_of: 2026-01-01T00:00:00Z\nuses:\n  - method:m\nsupports:\n  - evidence:e\nupdates:\n  - decision:missing\n---",
        );
        let result = check(&root);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|issue| issue.path.ends_with("wiki/w.md") && issue.field == "uses"));
        assert!(result
            .errors
            .iter()
            .any(|issue| issue.path.ends_with("evidence/e.md")
                && issue.message.contains("disallowed kind")));
        assert!(result
            .errors
            .iter()
            .any(|issue| issue.message.contains("decision:missing")));
        let _ = fs::remove_dir_all(root);
    }
}
