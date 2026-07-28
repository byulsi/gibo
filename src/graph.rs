//! Explicit-only, offline Decision Graph rendering.
use crate::knowledge::{self, Document, RELATIONSHIP_TYPES};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct GraphNode {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub path: String,
    pub as_of: String,
    pub source: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Ord, PartialOrd)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relationship: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct RenderStats {
    pub output: String,
    pub nodes: usize,
    pub edges: usize,
    pub skipped_files: usize,
    pub unresolved_relationships: usize,
}

pub fn scan(root: &Path) -> (Vec<GraphNode>, Vec<GraphEdge>, RenderStats) {
    let mut nodes = Vec::new();
    let mut raw = Vec::new();
    let mut skipped = 0;
    for document in knowledge::catalog_local(root) {
        if !knowledge::document_is_valid(&document) {
            skipped += 1;
            continue;
        }
        add_document(&mut nodes, &mut raw, &document);
    }
    let mut unique = BTreeMap::new();
    for node in nodes {
        unique.entry(node.id.clone()).or_insert(node);
    }
    let edges = resolve(&raw, &unique);
    let unresolved = raw_count_unresolved(&unique, &edges, &raw);
    let nodes = unique.into_values().collect::<Vec<_>>();
    let stats = RenderStats {
        output: display(&root.join("guru-maker/decision-graph.html"), root),
        nodes: nodes.len(),
        edges: edges.len(),
        skipped_files: skipped,
        unresolved_relationships: unresolved,
    };
    (nodes, edges, stats)
}

pub fn render(root: &Path, output: Option<&Path>) -> std::io::Result<RenderStats> {
    let (nodes, edges, mut stats) = scan(root);
    let path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("guru-maker/decision-graph.html"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?
    }
    let json = serde_json::to_string(&serde_json::json!({"nodes":nodes,"edges":edges}))
        .unwrap()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    fs::write(&path, html(&json))?;
    stats.output = path.to_string_lossy().into();
    Ok(stats)
}

fn add_document(
    nodes: &mut Vec<GraphNode>,
    raw: &mut Vec<(String, String, String)>,
    document: &Document,
) {
    let id = document.id.clone();
    nodes.push(GraphNode {
        id: id.clone(),
        kind: document.kind.clone(),
        title: document.title.clone(),
        summary: document.summary.clone(),
        path: document.path.clone(),
        as_of: document.as_of.clone(),
        source: document.source.clone(),
    });
    raw.extend(document.relationships.iter().map(|relationship| {
        (
            id.clone(),
            relationship.kind.clone(),
            relationship.target.clone(),
        )
    }));
    if document.kind == "evidence" {
        if let Some(source) = document
            .source
            .as_ref()
            .filter(|value| value.starts_with("https://") || value.starts_with("http://"))
        {
            let source_id = format!("source:{source}");
            nodes.push(GraphNode {
                id: source_id.clone(),
                kind: "source".into(),
                title: source.clone(),
                summary: "Evidence source".into(),
                path: source.clone(),
                as_of: String::new(),
                source: Some(source.clone()),
            });
            raw.push((id, "source".into(), source_id));
        }
    }
}

fn resolve(
    raw: &[(String, String, String)],
    nodes: &BTreeMap<String, GraphNode>,
) -> Vec<GraphEdge> {
    let mut edges = Vec::new();
    for (source, relationship, target) in raw {
        let Some(source_node) = nodes.get(source) else {
            continue;
        };
        let Some(target_node) = nodes.get(target) else {
            continue;
        };
        let allowed = relationship == "source"
            || (RELATIONSHIP_TYPES.contains(&relationship.as_str())
                && knowledge::relationship_target_allowed(
                    &source_node.kind,
                    relationship,
                    &target_node.kind,
                ));
        if allowed {
            edges.push(GraphEdge {
                source: source.clone(),
                target: target.clone(),
                relationship: relationship.clone(),
            });
        }
    }
    edges.sort();
    edges.dedup();
    edges
}

fn raw_count_unresolved(
    nodes: &BTreeMap<String, GraphNode>,
    edges: &[GraphEdge],
    raw: &[(String, String, String)],
) -> usize {
    raw.iter()
        .filter(|(source, relationship, target)| {
            let recognized =
                relationship == "source" || RELATIONSHIP_TYPES.contains(&relationship.as_str());
            recognized
                && nodes.contains_key(source)
                && !edges.iter().any(|edge| {
                    edge.source == *source
                        && edge.target == *target
                        && edge.relationship == *relationship
                })
        })
        .count()
}

fn display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn html(payload: &str) -> String {
    format!(
        r#"<!doctype html><meta charset="utf-8"><title>Guru Maker Decision Graph</title><style>body{{font:14px system-ui;margin:0;background:#f6f7f9;color:#172033}}header{{padding:20px 24px;background:#172033;color:#fff}}main{{padding:20px 24px}}svg{{width:100%;min-height:360px;background:#fff;border:1px solid #d9deea}}article{{background:#fff;border:1px solid #d9deea;padding:12px;margin-top:12px}}</style><header><h1>Decision Graph</h1><p id=s></p></header><main><svg id=g viewBox="0 0 1000 600"></svg><section id=d></section></main><script>const graph={payload},svg=document.querySelector('#g'),details=document.querySelector('#d'),cols=Math.max(1,Math.ceil(Math.sqrt(graph.nodes.length||1))),pos=new Map(graph.nodes.map((n,i)=>[n.id,{{x:90+i%cols*220,y:70+Math.floor(i/cols)*150}}]));s.textContent=`${{graph.nodes.length}} nodes · ${{graph.edges.length}} explicit relationships`;for(const e of graph.edges){{let a=pos.get(e.source),b=pos.get(e.target);if(!a||!b)continue;let l=document.createElementNS('http://www.w3.org/2000/svg','line');for(const[k,v]of Object.entries({{x1:a.x,y1:a.y,x2:b.x,y2:b.y,stroke:'#9da9be'}}))l.setAttribute(k,v);svg.append(l)}}for(const n of graph.nodes){{let p=pos.get(n.id),t=document.createElementNS('http://www.w3.org/2000/svg','text');t.setAttribute('x',p.x);t.setAttribute('y',p.y);t.textContent=n.title;svg.append(t);let a=document.createElement('article');a.textContent=`${{n.kind}} · ${{n.id}} — ${{n.summary}}`;details.append(a)}}</script>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn graph_has_only_valid_explicit_edges_and_safe_offline_html() {
        let root = std::env::temp_dir().join(format!(
            "guru-maker-graph-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let evidence = root.join("guru-maker/evidence/e.md");
        fs::create_dir_all(evidence.parent().unwrap()).unwrap();
        fs::write(&evidence,"---\nid: evidence:e\ntitle: </script><img>\nsummary: evidence\nas_of: 2026-01-01T00:00:00Z\nsource: https://example.test/source\n---").unwrap();
        let decision = root.join("guru-maker/decision/d.md");
        fs::create_dir_all(decision.parent().unwrap()).unwrap();
        fs::write(
            decision,
            "---\nid: decision:d\ntitle: D\nsummary: decision\nas_of: 2026-01-01T00:00:00Z\nsupports:\n  - evidence:e\n---",
        )
        .unwrap();
        let (nodes, edges, _) = scan(&root);
        assert!(nodes
            .iter()
            .any(|node| node.id == "source:https://example.test/source"));
        assert!(edges.iter().any(|edge| edge.relationship == "supports"));
        assert!(edges.iter().any(|edge| edge.relationship == "source"));
        let output = root.join("graph.html");
        render(&root, Some(&output)).unwrap();
        let page = fs::read_to_string(output).unwrap();
        assert!(page.contains("\\u003c/script\\u003e"));
        let _ = fs::remove_dir_all(root);
    }
}
