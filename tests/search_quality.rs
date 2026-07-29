use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_gurumaker")
}

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "guru-maker-search-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn command(arguments: &[&str]) -> Output {
    Command::new(bin()).args(arguments).output().unwrap()
}

fn write_record(
    root: &Path,
    kind: &str,
    id: &str,
    title: &str,
    summary: &str,
    extra_frontmatter: &str,
    body: &str,
) {
    let filename = format!("{}.md", id.replace(':', "-"));
    let path = root.join("guru-maker").join(kind).join(filename);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        format!(
            "---\nid: {id}\ntitle: {title}\nsummary: {summary}\n\
             as_of: 2026-07-29T09:00:00+09:00\n\
             {extra_frontmatter}---\n\n{body}\n"
        ),
    )
    .unwrap();
}

fn search(root: &Path, query: &str) -> Vec<serde_json::Value> {
    let output = command(&[
        "knowledge",
        "search",
        query,
        "--workspace",
        root.to_str().unwrap(),
        "--limit",
        "50",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn result<'a>(results: &'a [serde_json::Value], id: &str) -> Option<&'a serde_json::Value> {
    results.iter().find(|item| item["id"] == id)
}

fn assert_top(root: &Path, query: &str, expected: &str) {
    assert_eq!(
        search(root, query)
            .first()
            .and_then(|item| item["id"].as_str()),
        Some(expected),
        "unexpected top result for {query:?}"
    );
}

#[test]
fn ranking_context_and_relationship_recall_regression() {
    let root = temp_dir("ranking");
    assert!(command(&["init", root.to_str().unwrap()]).status.success());

    write_record(
        &root,
        "wiki",
        "wiki:cash-conversion-durability",
        "Cash conversion durability",
        "Tests whether accounting earnings become distributable cash.",
        "",
        "# Evidence quality\n\nFree cash flow should persist after normalized reinvestment.",
    );
    write_record(
        &root,
        "wiki",
        "wiki:liquidity-signals",
        "Free flow and cash checklist",
        "Reviews cash timing, free float, and transaction flow.",
        "tags: [free, flow, cash]\n",
        "# Trading liquidity\n\nMarket depth and turnover conventions.",
    );
    write_record(
        &root,
        "wiki",
        "wiki:net-revenue-retention",
        "Net revenue retention",
        "Separates expansion, contraction, and churn.",
        "aliases: [net dollar retention, NRR]\n",
        "# Interpretation\n\nCohort revenue retained after expansion and churn.",
    );
    write_record(
        &root,
        "wiki",
        "wiki:operating-profit-resilience",
        "영업 이익 회복력",
        "영업 이익이 매출 충격 이후 회복되는 조건.",
        "",
        "# 판단 기준\n\n영업 이익의 회복 속도와 비용 구조를 함께 본다.",
    );
    write_record(
        &root,
        "wiki",
        "wiki:capital-allocation-guardrail",
        "Capital allocation guardrail",
        "Requires returns above the cost of capital.",
        "",
        "",
    );
    write_record(
        &root,
        "wiki",
        "wiki:pricing-power-framework",
        "Pricing power framework",
        "Evaluates durable price realization.",
        "",
        "# Power history\n\nHistorical examples.\n\n# Appendix\n\nFormatting conventions.",
    );
    write_record(
        &root,
        "decision",
        "decision:original",
        "Original margin review",
        "Initial operating margin judgment.",
        "",
        "# Decision\n\nHold while unit economics remain intact.",
    );
    write_record(
        &root,
        "decision",
        "decision:revision",
        "Revised margin review",
        "Later judgment after costs increased.",
        "updates: [decision:original]\n",
        "# Decision\n\nReduce exposure after the margin break.",
    );

    assert_top(&root, "free cash flow", "wiki:cash-conversion-durability");
    assert_top(&root, "net dollar retention", "wiki:net-revenue-retention");
    assert_top(&root, "영업이익", "wiki:operating-profit-resilience");
    assert_top(&root, "decision:original", "decision:original");
    assert!(result(&search(&root, "decision:original"), "decision:revision").is_some());

    for (query, id) in [
        (
            "capital allocation guardrail",
            "wiki:capital-allocation-guardrail",
        ),
        ("pricing power framework", "wiki:pricing-power-framework"),
    ] {
        let results = search(&root, query);
        let item = result(&results, id).unwrap();
        assert_eq!(item["section"], "");
        assert_eq!(item["heading_path"], serde_json::json!([]));
    }
    assert!(!search(&root, "pricing power framework")[0]["text"]
        .as_str()
        .unwrap()
        .contains("Historical examples"));
    assert!(search(&root, "cocoa basis risk").is_empty());
    assert!(search(&root, "가격 전가 능력").is_empty());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn precision_numeric_and_multilingual_regression() {
    let root = temp_dir("precision");
    assert!(command(&["init", root.to_str().unwrap()]).status.success());

    write_record(
        &root,
        "wiki",
        "wiki:model-infrastructure",
        "AI infrastructure capex",
        "Tracks accelerator and data-center investment.",
        "",
        "# Capacity\n\nAI accelerator deployments are expanding.",
    );
    write_record(
        &root,
        "wiki",
        "wiki:subscriber-growth",
        "Paid subscriber growth",
        "Tracks subscription additions and churn.",
        "",
        "# Cohorts\n\nSubscriber retention by cohort.",
    );
    write_record(
        &root,
        "wiki",
        "wiki:earnings-quality",
        "Earnings quality",
        "Compares accounting earnings with cash generation.",
        "",
        "# Accounting\n\nNet income should be reconciled with cash flow.",
    );
    write_record(
        &root,
        "wiki",
        "wiki:internet-business",
        "Internet income trends",
        "Reviews digital subscription economics.",
        "",
        "# Business model\n\nDigital distribution and subscriber acquisition.",
    );
    write_record(
        &root,
        "evidence",
        "evidence:margin-change",
        "Operating margin update",
        "Operating margin changed +5.0%.",
        "source: https://example.com/margin\n",
        "# Observation\n\nThe reported margin improved.",
    );
    write_record(
        &root,
        "evidence",
        "evidence:unsigned-margin",
        "Unsigned margin notation",
        "The table reports a 5.0% margin without a direction sign.",
        "source: https://example.com/unsigned-margin\n",
        "# Observation\n\nThe source does not state a direction.",
    );
    write_record(
        &root,
        "evidence",
        "evidence:forecast-range",
        "Forecast period",
        "Forecast covers 2025-2026.",
        "source: https://example.com/forecast-range\n",
        "# Period\n\nThe forecast spans two calendar years.",
    );
    write_record(
        &root,
        "wiki",
        "wiki:japanese-profit",
        "営業 利益の持続性",
        "利益の質を評価する。",
        "",
        "# 判断\n\n再投資後の利益を確認する。",
    );
    write_record(
        &root,
        "wiki",
        "wiki:chinese-cash-flow",
        "自由 现金流质量",
        "评估现金转换的持续性。",
        "",
        "# 判断\n\n检查再投资后的现金流。",
    );
    write_record(
        &root,
        "wiki",
        "wiki:cyrillic-cash-flow",
        "Денежный Поток",
        "Проверяет устойчивость денежных потоков.",
        "",
        "# Интерпретация\n\nСравнивает прибыль и денежный поток.",
    );

    let ai = search(&root, "ai");
    assert_eq!(ai[0]["id"], "wiki:model-infrastructure");
    assert!(result(&ai, "wiki:subscriber-growth").is_none());
    assert_top(&root, "net income", "wiki:earnings-quality");

    assert_top(&root, "+5.0%", "evidence:margin-change");
    assert!(search(&root, "-5.0%").is_empty());
    assert!(search(&root, "−5.0%").is_empty());
    assert!(search(&root, "+5.1%").is_empty());
    assert_top(&root, "5.0%", "evidence:unsigned-margin");
    assert_top(&root, "2025-2026", "evidence:forecast-range");
    assert!(search(&root, "-2026").is_empty());

    assert_top(&root, "営業利益", "wiki:japanese-profit");
    assert_top(&root, "自由现金流", "wiki:chinese-cash-flow");
    assert_top(&root, "ДЕНЕЖНЫЙ ПОТОК", "wiki:cyrillic-cash-flow");
    assert_eq!(search(&root, "ai"), search(&root, "  ai  "));

    let blank = command(&[
        "knowledge",
        "search",
        " \t ",
        "--workspace",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert!(!blank.status.success());
    assert!(String::from_utf8_lossy(&blank.stderr).contains("query must not be empty"));

    fs::remove_dir_all(root).unwrap();
}
