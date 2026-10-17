//! File-native knowledge records. Markdown is canonical; no index is kept.

use chrono::DateTime;
use regex::Regex;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
    thread,
};

pub const RELATIONSHIP_TYPES: &[&str] = &["uses", "supports", "contradicts", "updates"];
const AUTHORED_RELATIONSHIP_TYPES: &[&str] =
    &["uses", "supports", "contradicts", "updates", "related"];
pub const KNOWLEDGE_KINDS: &[&str] = &["lens", "wiki", "evidence", "method", "decision"];
const PARALLEL_SCAN_MIN_FILES: usize = 256;
const MAX_SCAN_WORKERS: usize = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frontmatter {
    pub scalar: BTreeMap<String, String>,
    pub lists: BTreeMap<String, Vec<String>>,
    pub declared_fields: BTreeSet<String>,
    pub duplicate_fields: BTreeSet<String>,
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
    pub see_also: Vec<String>,
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
    #[serde(skip_serializing)]
    pub declared_fields: BTreeSet<String>,
    #[serde(skip_serializing)]
    pub duplicate_fields: BTreeSet<String>,
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
    #[serde(skip_serializing)]
    pub aliases: Vec<String>,
    #[serde(skip_serializing)]
    pub tags: Vec<String>,
    #[serde(skip_serializing)]
    pub match_tier: MatchTier,
    #[serde(skip_serializing)]
    pub matched_fields: Vec<String>,
    #[serde(skip_serializing)]
    pub matched_terms: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchTier {
    ExactId,
    ExactMetadata,
    Phrase,
    AllTerms,
    #[default]
    Partial,
}
impl MatchTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExactId => "exact_id",
            Self::ExactMetadata => "exact_metadata",
            Self::Phrase => "phrase",
            Self::AllTerms => "all_terms",
            Self::Partial => "partial",
        }
    }
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CandidateResult {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub as_of: String,
    pub path: String,
    pub section: String,
    pub heading_path: Vec<String>,
    pub entities: Vec<String>,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub relationships: Vec<Relationship>,
    pub score: i32,
    pub match_tier: MatchTier,
    pub matched_fields: Vec<String>,
    pub matched_terms: Vec<String>,
}
#[derive(Clone, Debug)]
struct RankedSearchResult {
    result: SearchResult,
    document: Document,
    section: Section,
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
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct KnowledgeCard {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub as_of: String,
    pub path: String,
    pub entities: Vec<String>,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct KnowledgeNeighbors {
    pub document: KnowledgeCard,
    pub outgoing: Vec<KnowledgeCard>,
    pub incoming: Vec<KnowledgeCard>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, Ord, PartialOrd)]
pub struct HealthAdvisory {
    pub code: String,
    pub ids: Vec<String>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct KindHealth {
    pub kind: String,
    pub documents: usize,
    pub folders: usize,
    pub max_depth: usize,
    pub review_band: String,
    pub advisories: Vec<HealthAdvisory>,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct KnowledgeHealth {
    pub kinds: Vec<KindHealth>,
}

pub fn infer_kind(path: &Path, root: &Path) -> String {
    let base = root.join("gibo");
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
pub fn search_with_kinds(
    root: &Path,
    query: &str,
    kinds: &[String],
    entity: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    search_internal(root, query, kinds, entity, limit, false)
}

pub fn search_candidates_with_kinds(
    root: &Path,
    query: &str,
    kinds: &[String],
    entity: Option<&str>,
    limit: usize,
) -> Result<Vec<CandidateResult>, String> {
    search_internal(root, query, kinds, entity, limit, true)
        .map(|results| candidate_results(&results))
}

fn search_internal(
    root: &Path,
    query: &str,
    kinds: &[String],
    entity: Option<&str>,
    limit: usize,
    explain: bool,
) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Err("knowledge search query must not be empty".into());
    }
    for k in kinds {
        if !KNOWLEDGE_KINDS.contains(&k.as_str()) {
            return Err(format!("unknown knowledge kind: {k}"));
        }
    }
    if !(1..=50).contains(&limit) {
        return Err("knowledge search limit must be between 1 and 50".into());
    }
    let paths = local_markdown_files_for_kinds(root, kinds);
    let query = SearchText::new(query.trim());
    let out = search_paths(root, &paths, &query, entity, limit, explain)?;
    Ok(finalize_search_results(out, limit))
}

fn search_paths(
    root: &Path,
    paths: &[PathBuf],
    query: &SearchText,
    entity: Option<&str>,
    limit: usize,
    explain: bool,
) -> Result<Vec<SearchResult>, String> {
    Ok(scan_chunks(paths, "search", |chunk| {
        search_chunk(root, chunk, query, entity, limit, explain)
    })?
    .into_iter()
    .flatten()
    .collect())
}

#[cfg(test)]
fn search_paths_with_workers(
    root: &Path,
    paths: &[PathBuf],
    query: &SearchText,
    entity: Option<&str>,
    limit: usize,
    explain: bool,
    workers: usize,
) -> Result<Vec<SearchResult>, String> {
    Ok(scan_chunks_with_workers(paths, workers, "search", |chunk| {
        search_chunk(root, chunk, query, entity, limit, explain)
    })?
    .into_iter()
    .flatten()
    .collect())
}

fn search_chunk(
    root: &Path,
    paths: &[PathBuf],
    query: &SearchText,
    entity: Option<&str>,
    limit: usize,
    explain: bool,
) -> Vec<SearchResult> {
    let mut top = Vec::with_capacity(limit);
    for path in paths {
        if let Some(candidate) = search_document(load_local(root, path), query, entity) {
            retain_top_ranked_result(&mut top, candidate, limit);
        }
    }
    top.into_iter()
        .map(|mut item| {
            if explain {
                let (match_tier, matched_fields, matched_terms) =
                    explain_match(&item.document, &item.section, query);
                item.result.match_tier = match_tier;
                item.result.matched_fields = matched_fields;
                item.result.matched_terms = matched_terms;
                item.result.text.clear();
            }
            item.result
        })
        .collect()
}

fn retain_top_ranked_result(
    top: &mut Vec<RankedSearchResult>,
    candidate: RankedSearchResult,
    limit: usize,
) {
    // A result below a chunk's top `limit` unique IDs cannot enter the global
    // top `limit`: that same chunk already contains `limit` distinct IDs ahead
    // of it. Keeping the best duplicate per ID preserves the final dedup order.
    if let Some(index) = top
        .iter()
        .position(|item| item.result.id == candidate.result.id)
    {
        if search_result_order(&candidate.result, &top[index].result).is_lt() {
            top[index] = candidate;
            top.sort_by(|a, b| search_result_order(&a.result, &b.result));
        }
        return;
    }
    top.push(candidate);
    top.sort_by(|a, b| search_result_order(&a.result, &b.result));
    top.truncate(limit);
}

fn search_document(
    doc: Document,
    query: &SearchText,
    entity: Option<&str>,
) -> Option<RankedSearchResult> {
    if entity.is_some_and(|wanted| {
        !doc.entities
            .iter()
            .any(|value| value.eq_ignore_ascii_case(wanted))
    }) {
        return None;
    }

    let document_relevance = document_relevance(&doc, query);
    let document_numbers = document_sensitive_numbers(&doc);
    let mut matched_section = false;
    let mut best = None;
    for section in split_sections(&doc.body) {
        let section_relevance = section_relevance(&section, query);
        let section_numbers = section_sensitive_numbers(&section);
        let numbers_match = query
            .sensitive_numbers
            .iter()
            .all(|number| document_numbers.contains(number) || section_numbers.contains(number));
        let complete_concise_document_match =
            !query.tokens.is_empty() && query.tokens.is_subset(&document_relevance.concise_terms);
        let weak_section_for_strong_document = (document_relevance.relevance.tier >= 3
            || complete_concise_document_match)
            && section_relevance.relevance.tier == 1;
        if section_relevance.relevance.is_match()
            && numbers_match
            && !weak_section_for_strong_document
        {
            matched_section = true;
            let relevance = combined_relevance(&document_relevance, &section_relevance, query);
            if has_minimum_query_coverage(relevance, &document_relevance, &section_relevance, query)
            {
                let result = search_result(&doc, &section, relevance.score());
                if best
                    .as_ref()
                    .is_none_or(|(current, _)| search_result_order(&result, current).is_lt())
                {
                    best = Some((result, section));
                }
            }
        }
    }
    if !matched_section
        && document_relevance.relevance.is_match()
        && query.sensitive_numbers.is_subset(&document_numbers)
    {
        let document_level = Section {
            heading: String::new(),
            heading_path: vec![],
            text: String::new(),
        };
        let section_relevance = SectionRelevance::default();
        if has_minimum_query_coverage(
            document_relevance.relevance,
            &document_relevance,
            &section_relevance,
            query,
        ) {
            best = Some((
                search_result(&doc, &document_level, document_relevance.relevance.score()),
                document_level,
            ));
        }
    }
    best.map(|(result, section)| RankedSearchResult {
        result,
        document: doc,
        section,
    })
}

fn finalize_search_results(mut out: Vec<SearchResult>, limit: usize) -> Vec<SearchResult> {
    out.sort_by(search_result_order);
    let mut seen = BTreeSet::new();
    out.retain(|item| seen.insert(item.id.clone()));
    out.truncate(limit);
    out
}

fn scan_worker_count(file_count: usize) -> usize {
    if file_count < PARALLEL_SCAN_MIN_FILES {
        return 1;
    }
    thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(MAX_SCAN_WORKERS)
        .min(file_count)
}

fn scan_chunks<T, F>(paths: &[PathBuf], operation: &str, scan: F) -> Result<Vec<T>, String>
where
    T: Send,
    F: Fn(&[PathBuf]) -> T + Sync,
{
    scan_chunks_with_workers(paths, scan_worker_count(paths.len()), operation, scan)
}

fn scan_chunks_with_workers<T, F>(
    paths: &[PathBuf],
    workers: usize,
    operation: &str,
    scan: F,
) -> Result<Vec<T>, String>
where
    T: Send,
    F: Fn(&[PathBuf]) -> T + Sync,
{
    let workers = workers.max(1).min(paths.len().max(1));
    if workers == 1 {
        return Ok(vec![scan(paths)]);
    }

    let chunk_size = paths.len().div_ceil(workers);
    thread::scope(|scope| {
        let mut chunks = Vec::new();
        for chunk in paths.chunks(chunk_size) {
            let scan = &scan;
            match thread::Builder::new().spawn_scoped(scope, move || scan(chunk)) {
                Ok(handle) => chunks.push(Ok(handle)),
                Err(_) => chunks.push(Err(scan(chunk))),
            }
        }

        let mut results = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            match chunk {
                Ok(handle) => match handle.join() {
                    Ok(result) => results.push(result),
                    Err(_) => return Err(format!("knowledge {operation} worker failed")),
                },
                Err(result) => results.push(result),
            }
        }
        Ok(results)
    })
}

fn candidate_results(results: &[SearchResult]) -> Vec<CandidateResult> {
    results
        .iter()
        .map(|result| CandidateResult {
            id: result.id.clone(),
            kind: result.kind.clone(),
            title: result.title.clone(),
            summary: result.summary.clone(),
            as_of: result.as_of.clone(),
            path: result.path.clone(),
            section: result.section.clone(),
            heading_path: result.heading_path.clone(),
            entities: result.entities.clone(),
            aliases: result.aliases.clone(),
            tags: result.tags.clone(),
            relationships: result.relationships.clone(),
            score: result.score,
            match_tier: result.match_tier.clone(),
            matched_fields: result.matched_fields.clone(),
            matched_terms: result.matched_terms.clone(),
        })
        .collect()
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
        for target in &doc.see_also {
            let Some(target_kind) = kinds.get(target.as_str()) else {
                issues.push(issue(
                    doc,
                    "see_also",
                    &format!("target does not exist: {target}"),
                ));
                continue;
            };
            if *target_kind != "wiki" {
                issues.push(issue(
                    doc,
                    "see_also",
                    &format!("target {target} has disallowed kind {target_kind}"),
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

pub fn neighbors(root: &Path, id: &str) -> Result<KnowledgeNeighbors, String> {
    let docs = catalog_local(root);
    let mut matches = docs.iter().filter(|doc| doc.id == id);
    let document = matches
        .next()
        .ok_or_else(|| format!("knowledge document not found: {id}"))?;
    if matches.next().is_some() {
        return Err(format!("knowledge document ID is ambiguous: {id}"));
    }
    if document.kind != "wiki" {
        return Err(format!("knowledge neighbors requires a Wiki ID: {id}"));
    }

    let by_id = docs
        .iter()
        .filter(|doc| !doc.id.is_empty())
        .map(|doc| (doc.id.as_str(), doc))
        .collect::<BTreeMap<_, _>>();
    let mut outgoing = Vec::new();
    for target in &document.see_also {
        let target_document = by_id
            .get(target.as_str())
            .ok_or_else(|| format!("see_also target does not exist: {target}"))?;
        if target_document.kind != "wiki" {
            return Err(format!("see_also target is not Wiki memory: {target}"));
        }
        outgoing.push(knowledge_card(target_document));
    }
    outgoing.sort_by(|a, b| a.id.cmp(&b.id));
    outgoing.dedup_by(|a, b| a.id == b.id);

    let mut incoming = docs
        .iter()
        .filter(|candidate| {
            candidate.kind == "wiki"
                && candidate.id != document.id
                && candidate.see_also.iter().any(|target| target == id)
        })
        .map(knowledge_card)
        .collect::<Vec<_>>();
    incoming.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(KnowledgeNeighbors {
        document: knowledge_card(document),
        outgoing,
        incoming,
    })
}

pub fn health(root: &Path, kind: Option<&str>) -> Result<KnowledgeHealth, String> {
    if let Some(kind) = kind {
        if !KNOWLEDGE_KINDS.contains(&kind) {
            return Err(format!("unknown knowledge kind: {kind}"));
        }
    }
    let docs = catalog_local(root);
    let kinds = KNOWLEDGE_KINDS
        .iter()
        .filter(|candidate| kind.is_none_or(|wanted| wanted == **candidate))
        .map(|kind| kind_health(&docs, kind))
        .collect();
    Ok(KnowledgeHealth { kinds })
}

fn knowledge_card(doc: &Document) -> KnowledgeCard {
    KnowledgeCard {
        id: doc.id.clone(),
        kind: doc.kind.clone(),
        title: doc.title.clone(),
        summary: doc.summary.clone(),
        as_of: doc.as_of.clone(),
        path: doc.path.clone(),
        entities: doc.entities.clone(),
        aliases: doc.aliases.clone(),
        tags: doc.tags.clone(),
    }
}

fn kind_health(docs: &[Document], kind: &str) -> KindHealth {
    let documents = docs
        .iter()
        .filter(|document| document.kind == kind)
        .collect::<Vec<_>>();
    let mut folder_names = BTreeSet::new();
    let mut max_depth = 0;
    let mut advisories = Vec::new();
    let mut deep = Vec::new();
    let mut large = Vec::new();
    let mut excessive_links = Vec::new();

    for document in &documents {
        let folders = subject_folders(document);
        max_depth = max_depth.max(folders.len());
        for depth in 1..=folders.len() {
            folder_names.insert(folders[..depth].join("/"));
        }
        if folders.len() > 2 {
            deep.push(document_key(document));
        }
        if document.content.chars().count() > 12_000 || split_sections(&document.body).len() > 8 {
            large.push(document_key(document));
        }
        if kind == "wiki" && document.see_also.len() > 5 {
            excessive_links.push(document_key(document));
        }
    }
    add_advisory(&mut advisories, "deep_folder", deep);
    add_advisory(&mut advisories, "large_document", large);
    add_duplicate_advisories(
        &mut advisories,
        "duplicate_title",
        documents
            .iter()
            .filter_map(|document| {
                let key = normalized_health_key(&document.title);
                (!key.is_empty()).then(|| (key, document_key(document)))
            })
            .collect(),
    );
    add_name_collision_advisories(&mut advisories, &documents);
    add_advisory(&mut advisories, "excessive_see_also", excessive_links);
    if kind == "evidence" {
        add_duplicate_advisories(
            &mut advisories,
            "duplicate_evidence_candidate",
            evidence_signatures(&documents),
        );
    }
    if kind == "decision" {
        add_decision_fork_advisories(&mut advisories, &documents);
    }
    advisories.sort();

    KindHealth {
        kind: kind.to_owned(),
        documents: documents.len(),
        folders: folder_names.len(),
        max_depth,
        review_band: review_band(kind, &documents).to_owned(),
        advisories,
    }
}

fn subject_folders(doc: &Document) -> Vec<String> {
    let components = doc.path.split('/').collect::<Vec<_>>();
    let start = components
        .windows(2)
        .position(|window| window == ["gibo", doc.kind.as_str()])
        .map_or(components.len(), |index| index + 2);
    components
        .get(start..components.len().saturating_sub(1))
        .unwrap_or_default()
        .iter()
        .map(|component| (*component).to_owned())
        .collect()
}

fn document_key(doc: &Document) -> String {
    if doc.id.is_empty() {
        doc.path.clone()
    } else {
        doc.id.clone()
    }
}

fn normalized_health_key(value: &str) -> String {
    normalize_search_text(value)
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn add_advisory(advisories: &mut Vec<HealthAdvisory>, code: &str, mut ids: Vec<String>) {
    ids.sort();
    ids.dedup();
    if !ids.is_empty() {
        advisories.push(HealthAdvisory {
            code: code.to_owned(),
            ids,
        });
    }
}

fn add_duplicate_advisories(
    advisories: &mut Vec<HealthAdvisory>,
    code: &str,
    keyed_ids: Vec<(String, String)>,
) {
    let mut groups = BTreeMap::<String, Vec<String>>::new();
    for (key, id) in keyed_ids {
        groups.entry(key).or_default().push(id);
    }
    for ids in groups.into_values() {
        let unique = ids.into_iter().collect::<BTreeSet<_>>();
        if unique.len() > 1 {
            add_advisory(advisories, code, unique.into_iter().collect());
        }
    }
}

fn add_name_collision_advisories(advisories: &mut Vec<HealthAdvisory>, documents: &[&Document]) {
    let mut groups = BTreeMap::<String, Vec<(String, bool)>>::new();
    for document in documents {
        let id = document_key(document);
        let title_key = normalized_health_key(&document.title);
        if !title_key.is_empty() {
            groups
                .entry(title_key)
                .or_default()
                .push((id.clone(), false));
        }
        for alias in &document.aliases {
            let alias_key = normalized_health_key(alias);
            if !alias_key.is_empty() {
                groups
                    .entry(alias_key)
                    .or_default()
                    .push((id.clone(), true));
            }
        }
    }
    for entries in groups.into_values() {
        let ids = entries
            .iter()
            .map(|(id, _)| id.clone())
            .collect::<BTreeSet<_>>();
        if ids.len() > 1 && entries.iter().any(|(_, is_alias)| *is_alias) {
            add_advisory(advisories, "duplicate_alias", ids.into_iter().collect());
        }
    }
}

fn evidence_signatures(documents: &[&Document]) -> Vec<(String, String)> {
    let mut signatures = Vec::new();
    for document in documents {
        let Some(source) = document.source.as_deref().filter(|value| !value.is_empty()) else {
            continue;
        };
        let Some(period) = document.period.as_deref().filter(|value| !value.is_empty()) else {
            continue;
        };
        for entity in &document.entities {
            signatures.push((
                format!(
                    "{}\u{0}{}\u{0}{}",
                    normalize_search_text(source),
                    normalize_search_text(period),
                    normalize_search_text(entity)
                ),
                document_key(document),
            ));
        }
    }
    signatures
}

fn add_decision_fork_advisories(advisories: &mut Vec<HealthAdvisory>, documents: &[&Document]) {
    let mut revisions = BTreeMap::<String, Vec<String>>::new();
    for document in documents {
        for relationship in &document.relationships {
            if matches!(relationship.kind.as_str(), "updates" | "contradicts") {
                revisions
                    .entry(relationship.target.clone())
                    .or_default()
                    .push(document_key(document));
            }
        }
    }
    for (target, revisions) in revisions {
        let revisions = revisions.into_iter().collect::<BTreeSet<_>>();
        if revisions.len() > 1 {
            let mut ids = vec![target];
            ids.extend(revisions);
            add_advisory(advisories, "decision_revision_fork", ids);
        }
    }
}

fn review_band(kind: &str, documents: &[&Document]) -> &'static str {
    let count = documents.len();
    let (organize, curate, scale, entity_threshold) = match kind {
        "wiki" => (25, 75, 200, None),
        "lens" | "method" => (15, 30, 100, None),
        "evidence" => (200, 500, 1_000, Some(50)),
        "decision" => (50, 150, 300, Some(10)),
        _ => (usize::MAX, usize::MAX, usize::MAX, None),
    };
    if count >= scale {
        "scale"
    } else if count >= curate {
        "curate"
    } else if count >= organize
        || entity_threshold.is_some_and(|threshold| {
            let mut counts = BTreeMap::<String, usize>::new();
            for document in documents {
                for entity in document
                    .entities
                    .iter()
                    .map(|entity| entity.to_ascii_lowercase())
                    .collect::<BTreeSet<_>>()
                {
                    *counts.entry(entity).or_default() += 1;
                }
            }
            counts.into_values().any(|count| count >= threshold)
        })
    {
        "organize"
    } else {
        "normal"
    }
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReadResult {
    pub document: Document,
    pub section: Option<Section>,
}
pub fn read(root: &Path, id: &str, section: Option<&str>) -> Result<ReadResult, String> {
    read_internal(root, id, section, None)
}

fn read_internal(
    root: &Path,
    id: &str,
    section: Option<&str>,
    workers: Option<usize>,
) -> Result<ReadResult, String> {
    let document = find_document_by_id(root, id, workers)?
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

fn find_document_by_id(
    root: &Path,
    id: &str,
    workers: Option<usize>,
) -> Result<Option<Document>, String> {
    let preferred_kind = id
        .split_once(':')
        .map(|(kind, _)| kind)
        .filter(|kind| KNOWLEDGE_KINDS.contains(kind));
    if let Some(kind) = preferred_kind {
        let paths = local_markdown_files_for_kinds(root, &[kind.to_owned()]);
        if let Some(document) = find_document_in_paths(root, &paths, id, workers)? {
            return Ok(Some(document));
        }
    }

    let mut paths = local_markdown_files(root);
    if let Some(kind) = preferred_kind {
        paths.retain(|path| infer_kind(path, root) != kind);
    }
    find_document_in_paths(root, &paths, id, workers)
}

fn find_document_in_paths(
    root: &Path,
    paths: &[PathBuf],
    id: &str,
    workers: Option<usize>,
) -> Result<Option<Document>, String> {
    let chunks = if let Some(workers) = workers {
        scan_chunks_with_workers(paths, workers, "read", |chunk| {
            find_document_in_chunk(root, chunk, id)
        })?
    } else {
        scan_chunks(paths, "read", |chunk| {
            find_document_in_chunk(root, chunk, id)
        })?
    };
    Ok(chunks.into_iter().flatten().next())
}

fn find_document_in_chunk(root: &Path, paths: &[PathBuf], id: &str) -> Option<Document> {
    paths
        .iter()
        .map(|path| load_local(root, path))
        .find(|document| document.id == id)
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
                if !meta.declared_fields.insert(key.clone()) {
                    meta.duplicate_fields.insert(key.clone());
                }
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
        see_also: m.lists.get("see_also").cloned().unwrap_or_default(),
        source: m.scalar.get("source").cloned(),
        published_at: m.scalar.get("published_at").cloned(),
        period: m.scalar.get("period").cloned(),
        origin: "local".into(),
        mutable: true,
        relationships,
        content,
        body,
        read_error: frontmatter_error,
        declared_fields: m.declared_fields,
        duplicate_fields: m.duplicate_fields,
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
        see_also: vec![],
        source: None,
        published_at: None,
        period: None,
        origin: "local".into(),
        mutable: true,
        relationships: vec![],
        content: String::new(),
        body: String::new(),
        read_error: Some(format!("cannot read Markdown: {message}")),
        declared_fields: BTreeSet::new(),
        duplicate_fields: BTreeSet::new(),
    }
}
fn validate_document(doc: &Document) -> Vec<KnowledgeIssue> {
    let mut out = vec![];
    if let Some(error) = &doc.read_error {
        out.push(issue(doc, "frontmatter", error));
    }
    for field in &doc.duplicate_fields {
        out.push(issue(
            doc,
            field,
            "frontmatter field must be declared only once",
        ));
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
    if doc.declared_fields.contains("see_also") && doc.kind != "wiki" {
        out.push(issue(
            doc,
            "see_also",
            "see_also is permitted only for Wiki memory",
        ));
    }
    let mut seen = BTreeSet::new();
    for target in &doc.see_also {
        if target == &doc.id {
            out.push(issue(doc, "see_also", "must not reference itself"));
        }
        if !target.starts_with("wiki:") {
            out.push(issue(
                doc,
                "see_also",
                &format!("target must use a wiki: ID: {target}"),
            ));
        }
        if !seen.insert(target) {
            out.push(issue(
                doc,
                "see_also",
                &format!("duplicate target: {target}"),
            ));
        }
    }
    for relationship in AUTHORED_RELATIONSHIP_TYPES {
        if doc.declared_fields.contains(*relationship)
            && !relationship_field_allowed(&doc.kind, relationship)
        {
            out.push(issue(
                doc,
                relationship,
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

fn relationship_field_allowed(source: &str, relationship: &str) -> bool {
    match source {
        "evidence" => matches!(relationship, "updates" | "contradicts"),
        "decision" => matches!(
            relationship,
            "uses" | "supports" | "updates" | "contradicts"
        ),
        _ => false,
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
        let directory = root.join("gibo").join(kind);
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
    local_markdown_files_for_kinds(root, &[])
}

fn local_markdown_files_for_kinds(root: &Path, kinds: &[String]) -> Vec<PathBuf> {
    let mut out = vec![];
    let directories = if kinds.is_empty() {
        KNOWLEDGE_KINDS.iter().copied().collect::<BTreeSet<_>>()
    } else {
        kinds.iter().map(String::as_str).collect::<BTreeSet<_>>()
    };
    for dir in directories {
        collect_markdown_files(&root.join("gibo").join(dir), &mut out);
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
    matches!(key, "entities" | "aliases" | "tags" | "see_also")
        || AUTHORED_RELATIONSHIP_TYPES.contains(&key)
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
#[derive(Clone, Debug)]
struct SearchText {
    normalized: String,
    compact: String,
    compact_eligible: bool,
    sensitive_numbers: BTreeSet<String>,
    tokens: BTreeSet<String>,
}

impl SearchText {
    fn new(value: &str) -> Self {
        let normalized = normalize_search_text(value);
        let compact_eligible = normalized.chars().any(is_spacing_insensitive_script)
            && normalized.chars().all(|character| {
                !character.is_alphanumeric() || is_spacing_insensitive_script(character)
            });
        let compact = normalized
            .chars()
            .filter(|character| character.is_alphanumeric())
            .collect();
        let sensitive_numbers = sensitive_numbers(&normalized);
        let tokens = search_tokens(&normalized);
        Self {
            normalized,
            compact,
            compact_eligible,
            sensitive_numbers,
            tokens,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Relevance {
    tier: i32,
    points: i32,
}

impl Relevance {
    fn add(&mut self, other: Self) {
        self.tier = self.tier.max(other.tier);
        self.points = self.points.saturating_add(other.points);
    }

    fn combined(mut self, other: Self) -> Self {
        self.add(other);
        self
    }

    fn is_match(self) -> bool {
        self.tier > 0
    }

    fn score(self) -> i32 {
        self.tier * 1_000 + self.points.min(999)
    }
}

#[derive(Clone, Debug, Default)]
struct ScoredField {
    relevance: Relevance,
    matched_terms: BTreeSet<String>,
}

#[derive(Clone, Debug, Default)]
struct DocumentRelevance {
    relevance: Relevance,
    concise_terms: BTreeSet<String>,
    authored_anchor: bool,
}

#[derive(Clone, Debug, Default)]
struct SectionRelevance {
    relevance: Relevance,
    concise_terms: BTreeSet<String>,
    all_terms: BTreeSet<String>,
}

fn normalize_search_text(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\u{2212}' | '\u{fe63}' | '\u{ff0d}' => '-',
            '\u{fe62}' | '\u{ff0b}' => '+',
            '\u{ff05}' => '%',
            _ => character,
        })
        .flat_map(char::to_lowercase)
        .collect()
}

fn identifier_pattern() -> &'static Regex {
    static IDENTIFIERS: OnceLock<Regex> = OnceLock::new();
    IDENTIFIERS.get_or_init(|| Regex::new(r"[\w:./+@#-]+").unwrap())
}

fn component_pattern() -> &'static Regex {
    static COMPONENTS: OnceLock<Regex> = OnceLock::new();
    COMPONENTS.get_or_init(|| Regex::new(r"\w+").unwrap())
}

fn sensitive_number_pattern() -> &'static Regex {
    static SENSITIVE_NUMBERS: OnceLock<Regex> = OnceLock::new();
    SENSITIVE_NUMBERS.get_or_init(|| {
        Regex::new(r"(?:^|[^\w])([+-]?\d+(?:\.\d+)?(?:/\d+(?:\.\d+)?)?%?)").unwrap()
    })
}

fn is_spacing_insensitive_script(character: char) -> bool {
    let codepoint = character as u32;
    (0x1100..=0x11ff).contains(&codepoint)
        || (0x3040..=0x30ff).contains(&codepoint)
        || (0x3130..=0x318f).contains(&codepoint)
        || (0x31f0..=0x31ff).contains(&codepoint)
        || (0x3400..=0x4dbf).contains(&codepoint)
        || (0x4e00..=0x9fff).contains(&codepoint)
        || (0xa960..=0xa97f).contains(&codepoint)
        || (0xac00..=0xd7af).contains(&codepoint)
        || (0xd7b0..=0xd7ff).contains(&codepoint)
        || (0xf900..=0xfaff).contains(&codepoint)
        || (0xff65..=0xff9f).contains(&codepoint)
        || (0x20000..=0x323af).contains(&codepoint)
}

fn sensitive_numbers(normalized: &str) -> BTreeSet<String> {
    sensitive_number_pattern()
        .captures_iter(normalized)
        .filter_map(|captures| captures.get(1))
        .map(|value| value.as_str())
        .filter(|value| {
            value
                .chars()
                .any(|character| matches!(character, '+' | '-' | '.' | '/' | '%'))
        })
        .map(str::to_owned)
        .collect()
}

fn contains_bounded_phrase(value: &str, query: &str) -> bool {
    let Some(first) = query.chars().next() else {
        return false;
    };
    let last = query
        .chars()
        .next_back()
        .expect("a query with a first character has a last character");
    value.match_indices(query).any(|(start, matched)| {
        let end = start + matched.len();
        let starts_at_boundary = !first.is_alphanumeric()
            || value[..start]
                .chars()
                .next_back()
                .is_none_or(|character| !character.is_alphanumeric());
        let ends_at_boundary = !last.is_alphanumeric()
            || value[end..]
                .chars()
                .next()
                .is_none_or(|character| !character.is_alphanumeric());
        starts_at_boundary && ends_at_boundary
    })
}

fn search_tokens(normalized: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for identifier in identifier_pattern().find_iter(normalized) {
        let value = identifier.as_str().to_owned();
        out.insert(value.clone());
        out.extend(
            component_pattern()
                .find_iter(&value)
                .map(|component| component.as_str().to_owned()),
        );
    }
    out
}

fn score_field(value: &str, weight: i32, query: &SearchText, allow_compact: bool) -> ScoredField {
    if value.is_empty() {
        return ScoredField::default();
    }
    let value = SearchText::new(value);
    let compact_match =
        allow_compact && query.compact_eligible && query.compact.chars().count() >= 3;
    let exact =
        value.normalized == query.normalized || (compact_match && value.compact == query.compact);
    let phrase = (query.tokens.len() > 1
        && contains_bounded_phrase(&value.normalized, &query.normalized))
        || (compact_match && value.compact.contains(&query.compact));
    let mut matched_terms = value
        .tokens
        .intersection(&query.tokens)
        .cloned()
        .collect::<BTreeSet<_>>();
    let matched = matched_terms.len() as i32;
    let query_terms = query.tokens.len() as i32;
    let tier = if exact {
        4
    } else if phrase {
        3
    } else if query_terms > 0 && matched == query_terms {
        2
    } else if matched > 0 {
        1
    } else {
        0
    };
    if compact_match && value.compact.contains(&query.compact) && matched_terms.is_empty() {
        matched_terms.extend(query.tokens.iter().cloned());
    }
    ScoredField {
        relevance: Relevance {
            tier,
            points: if tier > 0 { matched.max(1) * weight } else { 0 },
        },
        matched_terms,
    }
}

fn add_document_field(
    result: &mut DocumentRelevance,
    value: &str,
    weight: i32,
    query: &SearchText,
    allow_compact: bool,
) {
    let field = score_field(value, weight, query, allow_compact);
    result.relevance.add(field.relevance);
    result.concise_terms.extend(field.matched_terms);
}

fn contains_authored_anchor(value: &str, query: &SearchText) -> bool {
    let value = normalize_search_text(value);
    !value.is_empty()
        && value != query.normalized
        && contains_bounded_phrase(&query.normalized, &value)
}

fn document_relevance(doc: &Document, query: &SearchText) -> DocumentRelevance {
    let id = score_field(&doc.id, 14, query, false);
    let authored_anchor = contains_authored_anchor(&doc.id, query)
        || doc
            .aliases
            .iter()
            .any(|value| contains_authored_anchor(value, query))
        || doc
            .entities
            .iter()
            .any(|value| contains_authored_anchor(value, query));
    let mut result = DocumentRelevance {
        relevance: id.relevance,
        concise_terms: id.matched_terms,
        authored_anchor,
    };
    if normalize_search_text(&doc.id) == query.normalized {
        result.relevance.tier = 5;
    }
    add_document_field(&mut result, &doc.title, 12, query, true);
    for value in &doc.aliases {
        add_document_field(&mut result, value, 12, query, true);
    }
    for value in &doc.entities {
        add_document_field(&mut result, value, 12, query, false);
    }
    for value in &doc.tags {
        add_document_field(&mut result, value, 8, query, true);
    }
    add_document_field(
        &mut result,
        doc.period.as_deref().unwrap_or_default(),
        8,
        query,
        false,
    );
    add_document_field(&mut result, &doc.summary, 7, query, false);
    for relationship in &doc.relationships {
        add_document_field(&mut result, &relationship.target, 4, query, false);
    }
    if result.authored_anchor {
        result.relevance.tier = result.relevance.tier.max(3);
    }
    if !query.tokens.is_empty() && query.tokens.is_subset(&result.concise_terms) {
        result.relevance.tier = result.relevance.tier.max(2);
        result.relevance.points = result
            .relevance
            .points
            .saturating_add(query.tokens.len() as i32 * 6);
    }
    result
}

fn section_relevance(section: &Section, query: &SearchText) -> SectionRelevance {
    let heading = score_field(&section.heading, 8, query, true);
    let mut body = score_field(&section.text, 4, query, false);
    if body.relevance.tier == 4 {
        body.relevance.tier = if query.tokens.len() > 1 { 3 } else { 1 };
    } else if body.relevance.tier == 2 {
        body.relevance.tier = 1;
    }
    let mut all_terms = heading.matched_terms.clone();
    all_terms.extend(body.matched_terms);
    SectionRelevance {
        relevance: heading.relevance.combined(body.relevance),
        concise_terms: heading.matched_terms,
        all_terms,
    }
}

fn combined_relevance(
    document: &DocumentRelevance,
    section: &SectionRelevance,
    query: &SearchText,
) -> Relevance {
    let mut relevance = document.relevance.combined(section.relevance);
    if !query.tokens.is_empty()
        && query.tokens.iter().all(|term| {
            document.concise_terms.contains(term) || section.concise_terms.contains(term)
        })
    {
        relevance.tier = relevance.tier.max(2);
        relevance.points = relevance
            .points
            .saturating_add(query.tokens.len() as i32 * 4);
    }
    relevance
}

fn document_sensitive_numbers(doc: &Document) -> BTreeSet<String> {
    let mut numbers = BTreeSet::new();
    for value in [
        doc.id.as_str(),
        doc.title.as_str(),
        doc.summary.as_str(),
        doc.period.as_deref().unwrap_or_default(),
    ]
    .into_iter()
    .chain(doc.aliases.iter().map(String::as_str))
    .chain(doc.entities.iter().map(String::as_str))
    .chain(doc.tags.iter().map(String::as_str))
    .chain(
        doc.relationships
            .iter()
            .map(|relationship| relationship.target.as_str()),
    ) {
        numbers.extend(sensitive_numbers(&normalize_search_text(value)));
    }
    numbers
}

fn section_sensitive_numbers(section: &Section) -> BTreeSet<String> {
    let mut numbers = sensitive_numbers(&normalize_search_text(&section.heading));
    numbers.extend(sensitive_numbers(&normalize_search_text(&section.text)));
    numbers
}

fn search_result(doc: &Document, section: &Section, score: i32) -> SearchResult {
    SearchResult {
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
        text: contextual(doc, section),
        aliases: doc.aliases.clone(),
        tags: doc.tags.clone(),
        match_tier: MatchTier::Partial,
        matched_fields: Vec::new(),
        matched_terms: Vec::new(),
    }
}

fn has_minimum_query_coverage(
    relevance: Relevance,
    document: &DocumentRelevance,
    section: &SectionRelevance,
    query: &SearchText,
) -> bool {
    query.tokens.len() <= 1
        || document.authored_anchor
        || relevance.tier >= 2
        || query
            .tokens
            .iter()
            .filter(|term| {
                document.concise_terms.contains(*term) || section.all_terms.contains(*term)
            })
            .take(2)
            .count()
            >= 2
}

fn explain_match(
    doc: &Document,
    section: &Section,
    query: &SearchText,
) -> (MatchTier, Vec<String>, Vec<String>) {
    let mut fields = BTreeSet::new();
    let mut terms = BTreeSet::new();
    let mut exact_metadata = false;
    let mut phrase = false;
    let named_fields = [
        ("id", doc.id.as_str(), false),
        ("title", doc.title.as_str(), true),
        ("summary", doc.summary.as_str(), false),
        ("period", doc.period.as_deref().unwrap_or_default(), false),
    ];
    for (name, value, compact) in named_fields {
        collect_match_details(
            name,
            value,
            compact,
            query,
            &mut fields,
            &mut terms,
            &mut exact_metadata,
            &mut phrase,
        );
    }
    for (name, values, compact) in [
        ("aliases", &doc.aliases, true),
        ("entities", &doc.entities, false),
        ("tags", &doc.tags, true),
    ] {
        for value in values {
            collect_match_details(
                name,
                value,
                compact,
                query,
                &mut fields,
                &mut terms,
                &mut exact_metadata,
                &mut phrase,
            );
        }
    }
    for relationship in &doc.relationships {
        collect_match_details(
            "relationships",
            &relationship.target,
            false,
            query,
            &mut fields,
            &mut terms,
            &mut exact_metadata,
            &mut phrase,
        );
    }
    collect_match_details(
        "heading",
        &section.heading,
        true,
        query,
        &mut fields,
        &mut terms,
        &mut exact_metadata,
        &mut phrase,
    );
    let concise_terms = terms.clone();
    let mut body = score_field(&section.text, 4, query, false);
    if body.relevance.tier == 4 {
        body.relevance.tier = if query.tokens.len() > 1 { 3 } else { 1 };
    } else if body.relevance.tier == 2 {
        body.relevance.tier = 1;
    }
    if body.relevance.is_match() {
        fields.insert("body".to_owned());
        terms.extend(body.matched_terms);
        phrase |= body.relevance.tier >= 3;
    }

    let match_tier = if normalize_search_text(&doc.id) == query.normalized {
        MatchTier::ExactId
    } else if exact_metadata {
        MatchTier::ExactMetadata
    } else if phrase {
        MatchTier::Phrase
    } else if !query.tokens.is_empty() && query.tokens.is_subset(&concise_terms) {
        MatchTier::AllTerms
    } else {
        MatchTier::Partial
    };
    (
        match_tier,
        fields.into_iter().collect(),
        terms.into_iter().collect(),
    )
}

#[allow(clippy::too_many_arguments)]
fn collect_match_details(
    name: &str,
    value: &str,
    allow_compact: bool,
    query: &SearchText,
    fields: &mut BTreeSet<String>,
    terms: &mut BTreeSet<String>,
    exact_metadata: &mut bool,
    phrase: &mut bool,
) {
    let field = score_field(value, 1, query, allow_compact);
    if field.relevance.is_match() {
        fields.insert(name.to_owned());
        terms.extend(field.matched_terms);
        *exact_metadata |= field.relevance.tier >= 4;
        *phrase |= field.relevance.tier == 3;
    }
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
            "gibo-knowledge-{}-{}-{}",
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

    fn search_with_workers_for_test(
        root: &Path,
        query: &str,
        kinds: &[String],
        entity: Option<&str>,
        limit: usize,
        explain: bool,
        workers: usize,
    ) -> Vec<SearchResult> {
        let paths = local_markdown_files_for_kinds(root, kinds);
        let query = SearchText::new(query);
        let ranked =
            search_paths_with_workers(root, &paths, &query, entity, limit, explain, workers)
                .unwrap();
        finalize_search_results(ranked, limit)
    }

    #[test]
    fn check_reports_required_time_evidence_and_duplicates() {
        let root = root();
        write(
            &root,
            "gibo/lens/a.md",
            "---\nid: duplicate\ntitle: A\nsummary: A\nas_of: 2026-01-01T00:00Z\n---",
        );
        write(
            &root,
            "gibo/method/b.md",
            "---\nid: duplicate\ntitle: B\nsummary: B\nas_of: 2026-01-01T00:00:00Z\n---",
        );
        write(
            &root,
            "gibo/evidence/e.md",
            "---\nid: e\ntitle: E\nsummary: E\nas_of: 2026-01-01T00:00:00Z\n---",
        );
        write(&root, "gibo/wiki/plain.md", "# no frontmatter\n");
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
            "gibo/wiki/a.md",
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
            "gibo/evidence/cash-flow.md",
            "---\nid: evidence:cash-flow\ntitle: Cash generation\nsummary: Quarterly observation.\nas_of: 2026-07-27T09:00:00Z\nsource: https://example.com/report\nperiod: 2026-Q2\ntags:\n  - free-cash-flow\n---\n\n# Observation\nConversion improved.",
        );

        let compound = search_with_kinds(&root, "free cash flow", &[], None, 5).unwrap();
        assert_eq!(compound.len(), 1);
        assert_eq!(compound[0].id, "evidence:cash-flow");

        let period = search_with_kinds(&root, "2026 Q2", &[], None, 5).unwrap();
        assert_eq!(period.len(), 1);
        assert_eq!(period[0].id, "evidence:cash-flow");
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn search_returns_one_section_per_record_and_breaks_ties_by_as_of() {
        let root = root();
        write(
            &root,
            "gibo/wiki/older.md",
            "---\nid: wiki:older\ntitle: Older signal\nsummary: Cash flow signal.\nas_of: 2026-01-01T00:00:00Z\n---\n\n# First\nCash flow.\n\n# Second\nCash flow.",
        );
        write(
            &root,
            "gibo/wiki/newer.md",
            "---\nid: wiki:newer\ntitle: Newer signal\nsummary: Cash flow signal.\nas_of: 2026-07-01T00:00:00Z\n---\n\n# First\nCash flow.\n\n# Second\nCash flow.",
        );

        let results = search_with_kinds(&root, "cash flow signal", &[], None, 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "wiki:newer");
        assert_eq!(results[1].id, "wiki:older");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sequential_and_parallel_scans_preserve_search_candidates_filters_and_reads() {
        let root = root();
        for index in 0..320 {
            let special = if index == 257 {
                "aliases: [durable conversion signal]\nentities: [ticker:FILT]\nsee_also: [wiki:record-001]\n"
            } else {
                ""
            };
            let numeric = if index == 257 {
                "The signed operating-margin change was +5.0%."
            } else {
                "The record has no signed operating-margin observation."
            };
            write(
                &root,
                &format!("gibo/wiki/record-{index:03}.md"),
                &format!(
                    "---\nid: wiki:record-{index:03}\ntitle: Record {index:03}\nsummary: Common liquidity observation.\nas_of: 2026-01-01T00:00:00Z\n{special}---\n\n# Common liquidity\nBroad liquidity context for record {index}.\n\n# Detail\nRecord-specific detail {index}. {numeric}"
                ),
            );
        }
        write(
            &root,
            "gibo/lens/union.md",
            "---\nid: lens:union\ntitle: Union signal lens\nsummary: A repeated-kind filter target.\nas_of: 2026-01-01T00:00:00Z\n---\n\n# Application\nUnion signal.",
        );
        write(
            &root,
            "gibo/evidence/excluded.md",
            "---\nid: evidence:excluded\ntitle: Union signal evidence\nsummary: An excluded repeated-kind target.\nas_of: 2026-01-01T00:00:00Z\nsource: https://example.test/excluded\n---\n\n# Observation\nUnion signal.",
        );
        write(
            &root,
            "gibo/lens/legacy.md",
            "---\nid: custom:legacy\ntitle: Legacy identifier\nsummary: Exact-read fallback fixture.\nas_of: 2026-01-01T00:00:00Z\n---\n\n# Detail\nFallback body.",
        );
        write(
            &root,
            "gibo/lens/misplaced.md",
            "---\nid: wiki:misplaced\ntitle: Misplaced identifier\nsummary: Known-prefix read fallback fixture.\nas_of: 2026-01-01T00:00:00Z\n---\n\n# Detail\nKnown-prefix fallback body.",
        );
        write(
            &root,
            "gibo/wiki/a-duplicate.md",
            "---\nid: wiki:duplicate\ntitle: Duplicate loser\nsummary: A duplicate winner query appears only in summary.\nas_of: 2026-01-01T00:00:00Z\n---\n\n# Notes\nDuplicate winner.",
        );
        write(
            &root,
            "gibo/wiki/z-duplicate.md",
            "---\nid: wiki:duplicate\ntitle: Duplicate winner\nsummary: The higher-ranked duplicate.\nas_of: 2026-01-01T00:00:00Z\n---\n\n# Notes\nWinner body.",
        );

        assert_eq!(scan_worker_count(PARALLEL_SCAN_MIN_FILES - 1), 1);
        assert!((1..=MAX_SCAN_WORKERS).contains(&scan_worker_count(320)));

        let no_kinds = Vec::<String>::new();
        for query in [
            "common liquidity",
            "durable conversion signal",
            "+5.0%",
            "wiki:record-257",
            "duplicate winner",
        ] {
            let sequential =
                search_with_workers_for_test(&root, query, &no_kinds, None, 50, false, 1);
            let parallel =
                search_with_workers_for_test(&root, query, &no_kinds, None, 50, false, 8);
            assert_eq!(parallel, sequential, "default search parity for {query:?}");
            assert_eq!(
                search_with_kinds(&root, query, &no_kinds, None, 50).unwrap(),
                sequential,
                "automatic scan parity for {query:?}"
            );

            let sequential =
                search_with_workers_for_test(&root, query, &no_kinds, None, 50, true, 1);
            let parallel = search_with_workers_for_test(&root, query, &no_kinds, None, 50, true, 8);
            assert_eq!(
                parallel, sequential,
                "candidate search parity for {query:?}"
            );
            assert_eq!(
                search_candidates_with_kinds(&root, query, &no_kinds, None, 50).unwrap(),
                candidate_results(&sequential),
                "automatic candidate parity for {query:?}"
            );
        }
        assert_eq!(
            search_with_kinds(&root, "duplicate winner", &no_kinds, None, 50).unwrap()[0].title,
            "Duplicate winner"
        );
        assert!(search_with_kinds(&root, "-5.0%", &no_kinds, None, 50)
            .unwrap()
            .is_empty());

        let all_paths = local_markdown_files(&root);
        let broad_query = SearchText::new("common liquidity");
        assert!(
            search_paths_with_workers(&root, &all_paths, &broad_query, None, 7, false, 1)
                .unwrap()
                .len()
                <= 7
        );
        assert!(
            search_paths_with_workers(&root, &all_paths, &broad_query, None, 7, false, 8)
                .unwrap()
                .len()
                <= 8 * 7
        );

        let repeated_kinds = vec!["wiki".to_owned(), "lens".to_owned(), "wiki".to_owned()];
        let unique_kinds = vec!["wiki".to_owned(), "lens".to_owned()];
        assert_eq!(
            local_markdown_files_for_kinds(&root, &repeated_kinds),
            local_markdown_files_for_kinds(&root, &unique_kinds)
        );
        let sequential =
            search_with_workers_for_test(&root, "union signal", &repeated_kinds, None, 50, true, 1);
        let parallel =
            search_with_workers_for_test(&root, "union signal", &repeated_kinds, None, 50, true, 8);
        assert_eq!(parallel, sequential);
        assert!(parallel.iter().all(|item| item.kind != "evidence"));
        assert_eq!(
            search_candidates_with_kinds(&root, "union signal", &repeated_kinds, None, 50).unwrap(),
            candidate_results(&sequential)
        );

        let sequential = search_with_workers_for_test(
            &root,
            "common liquidity",
            &no_kinds,
            Some("TICKER:FILT"),
            50,
            true,
            1,
        );
        let parallel = search_with_workers_for_test(
            &root,
            "common liquidity",
            &no_kinds,
            Some("TICKER:FILT"),
            50,
            true,
            8,
        );
        assert_eq!(parallel, sequential);
        assert_eq!(parallel.len(), 1);
        assert_eq!(parallel[0].id, "wiki:record-257");
        assert!(search_with_kinds(
            &root,
            "common liquidity",
            &no_kinds,
            Some("ticker:OTHER"),
            50,
        )
        .unwrap()
        .is_empty());

        let sequential = read_internal(&root, "wiki:record-257", Some("Detail"), Some(1)).unwrap();
        let parallel = read_internal(&root, "wiki:record-257", Some("Detail"), Some(8)).unwrap();
        assert_eq!(parallel, sequential);
        assert_eq!(parallel.document.see_also, vec!["wiki:record-001"]);
        assert!(parallel.section.unwrap().text.contains("+5.0%"));
        assert_eq!(
            read(&root, "wiki:record-257", Some("Detail")).unwrap(),
            sequential
        );

        let sequential = read_internal(&root, "custom:legacy", None, Some(1)).unwrap();
        let parallel = read_internal(&root, "custom:legacy", None, Some(8)).unwrap();
        assert_eq!(parallel, sequential);
        assert_eq!(parallel.document.kind, "lens");

        let sequential = read_internal(&root, "wiki:misplaced", None, Some(1)).unwrap();
        let parallel = read_internal(&root, "wiki:misplaced", None, Some(8)).unwrap();
        assert_eq!(parallel, sequential);
        assert_eq!(parallel.document.kind, "lens");

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
        let wiki = root.join("gibo/wiki");
        fs::create_dir_all(&wiki).unwrap();
        symlink(&outside, wiki.join("escaped-directory")).unwrap();
        symlink(outside.join("escaped.md"), wiki.join("escaped-file.md")).unwrap();
        symlink(&wiki, wiki.join("loop")).unwrap();
        fs::create_dir_all(root.join("gibo")).unwrap();
        symlink(&outside, root.join("gibo/lens")).unwrap();

        assert!(catalog_local(&root).is_empty());
        assert_eq!(check(&root).errors.len(), 4);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn check_enforces_the_minimal_relationship_matrix() {
        let root = root();
        write(
            &root,
            "gibo/wiki/w.md",
            "---\nid: wiki:w\ntitle: W\nsummary: W\nas_of: 2026-01-01T00:00:00Z\nuses:\n  - method:m\n---",
        );
        write(
            &root,
            "gibo/method/m.md",
            "---\nid: method:m\ntitle: M\nsummary: M\nas_of: 2026-01-01T00:00:00Z\n---",
        );
        write(
            &root,
            "gibo/evidence/e.md",
            "---\nid: evidence:e\ntitle: E\nsummary: E\nas_of: 2026-01-01T00:00:00Z\nsource: https://example.test/e\nupdates:\n  - method:m\n---",
        );
        write(
            &root,
            "gibo/decision/d.md",
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
