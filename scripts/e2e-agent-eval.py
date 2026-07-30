#!/usr/bin/env python3
"""Run isolated, model-backed Guru Maker E2E evaluations."""

from __future__ import annotations

import argparse
import concurrent.futures
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import time
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CASES = ROOT / "tests" / "e2e" / "cases.json"
PACK_FIXTURE_URL = "https://github.com/guru-maker-e2e/wiki-pack.git"


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content.strip() + "\n", encoding="utf-8")


def run_checked(
    arguments: list[str],
    *,
    cwd: Path | None = None,
    environment: dict[str, str] | None = None,
) -> str:
    completed = subprocess.run(
        arguments,
        cwd=cwd,
        env=environment,
        check=True,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return completed.stdout


def record(
    element_id: str,
    title: str,
    summary: str,
    body: str,
    *,
    source: str | None = None,
    fields: str = "",
) -> str:
    source_line = f"source: {source}\n" if source else ""
    extra = fields.strip()
    if extra:
        extra += "\n"
    return (
        "---\n"
        f"id: {element_id}\n"
        f"title: {title}\n"
        f"summary: {summary}\n"
        "as_of: 2026-07-28T16:00:00-04:00\n"
        f"{source_line}{extra}"
        "---\n\n"
        f"# {title}\n\n{body}\n"
    )


CAPABILITY_SKILLS = {
    "filings": (
        "fixture-filings",
        "Read supplied issuer filing snapshots under sources/ for company research. "
        "Use when current filing evidence is requested in this isolated evaluation.",
        """# Fixture Filings

Read only the relevant supplied filing snapshot under `sources/`. Report its
source locator, period, and cutoff. Own filing retrieval and transcription, but
leave investment synthesis and Guru Maker memory decisions to the coordinating
Skill. Do not treat the fixture as a live external feed.
""",
    ),
    "macro-data": (
        "fixture-macro-data",
        "Read supplied macro, rates, or commodity snapshots under sources/. "
        "Use when the evaluation request needs those current fixture observations.",
        """# Fixture Macro Data

Read the smallest relevant supplied snapshot under `sources/`. Preserve its
date, units, frequency, and stated conventions. Do not extrapolate beyond the
fixture or make memory and investment decisions on behalf of another Skill.
""",
    ),
    "market-data": (
        "fixture-market-data",
        "Read supplied price or return series under sources/ for technical and "
        "quantitative evaluation tasks.",
        """# Fixture Market Data

Use the relevant supplied CSV under `sources/`. Inspect its venue, price
construction, frequency, window, units, and cutoff before calculation. State
material transformations and parameters. Do not invent missing observations.
""",
    ),
    "fund-documents": (
        "fixture-fund-documents",
        "Read supplied fund factsheets under sources/ for ETF and fund research.",
        """# Fixture Fund Documents

Read the relevant supplied factsheet files under `sources/`. Preserve each
document's as-of date, fee basis, benchmark, and other stated conventions.
Normalize only comparable fields and identify unresolved differences.
""",
    ),
}

PUBLIC_SKILLS = {
    "guru-research",
    "guru-memory",
    "guru-decision",
    "guru-train",
    "guru-packs",
}
VALID_WORKSPACES = {"auto", "off", "none"}
CHECK_REQUIRED_KEYS = {
    "event_regex": {"pattern"},
    "event_not_regex": {"pattern"},
    "event_command_count": {"pattern"},
    "final_regex": {"pattern"},
    "final_not_regex": {"pattern"},
    "change_any": {"pattern"},
    "change_none": {"pattern"},
    "changed_exactly_one_matching": {"pattern"},
    "changed_at_most_matching": {"pattern", "count"},
    "changed_only": {"paths"},
    "changed_content_regex": {"pattern"},
    "changed_content_not_regex": {"pattern"},
    "path_exists": {"path"},
    "path_absent": {"path"},
    "glob_min": {"glob", "count"},
    "glob_exact": {"glob", "count"},
    "mode_equals": {"value"},
    "content_regex": {"path", "pattern"},
    "content_not_regex": {"path", "pattern"},
    "frontmatter_list_equals": {"path", "field", "values"},
    "record_frontmatter_list_equals": {"record_id", "field", "values"},
    "wiki_one_way_link": set(),
    "glob_content_regex": {"glob", "pattern"},
    "glob_content_match_count": {"glob", "pattern", "count"},
    "initial_preserved": {"path"},
    "knowledge_valid": set(),
}
REQUIRED_CASE_FIELDS = {
    "id",
    "title",
    "skill",
    "universe",
    "intent",
    "workspace",
    "fixture",
    "prompt",
    "expected",
    "checks",
    "qualitative_focus",
}


def validate_configuration(configuration: Any) -> list[dict[str, Any]]:
    if not isinstance(configuration, dict):
        raise ValueError("E2E configuration must be a JSON object")
    cases = configuration.get("cases")
    if not isinstance(cases, list) or not cases:
        raise ValueError("E2E configuration must contain a non-empty cases list")

    seen_ids: set[str] = set()
    for index, case in enumerate(cases):
        label = f"case at index {index}"
        if not isinstance(case, dict):
            raise ValueError(f"{label} must be an object")
        missing = REQUIRED_CASE_FIELDS - set(case)
        if missing:
            raise ValueError(f"{label} misses fields: {', '.join(sorted(missing))}")

        case_id = case["id"]
        if not isinstance(case_id, str) or not case_id.strip():
            raise ValueError(f"{label} has an invalid id")
        if case_id in seen_ids:
            raise ValueError(f"duplicate case id: {case_id}")
        seen_ids.add(case_id)
        label = case_id

        for field in ("title", "universe", "intent", "fixture", "prompt", "expected"):
            if not isinstance(case[field], str) or not case[field].strip():
                raise ValueError(f"{label} has an empty {field}")
        if case["skill"] not in PUBLIC_SKILLS:
            raise ValueError(f"{label} has unknown public Skill: {case['skill']}")
        if case["workspace"] not in VALID_WORKSPACES:
            raise ValueError(f"{label} has invalid workspace: {case['workspace']}")

        capabilities = case.get("capabilities", [])
        if not isinstance(capabilities, list) or any(
            capability not in CAPABILITY_SKILLS for capability in capabilities
        ):
            raise ValueError(f"{label} has invalid capabilities")

        focus = case["qualitative_focus"]
        if (
            not isinstance(focus, list)
            or not focus
            or any(not isinstance(item, str) or not item.strip() for item in focus)
        ):
            raise ValueError(f"{label} needs non-empty qualitative focus")

        checks = case["checks"]
        if not isinstance(checks, list) or not checks:
            raise ValueError(f"{label} needs at least one deterministic assertion")
        if all(
            isinstance(check, dict) and check.get("type") == "knowledge_valid"
            for check in checks
        ):
            raise ValueError(
                f"{label} needs an outcome assertion beyond schema validation"
            )
        for check_index, check in enumerate(checks):
            if not isinstance(check, dict):
                raise ValueError(f"{label} check {check_index} must be an object")
            check_type = check.get("type")
            if check_type not in CHECK_REQUIRED_KEYS:
                raise ValueError(
                    f"{label} check {check_index} has unknown type: {check_type}"
                )
            required = CHECK_REQUIRED_KEYS[check_type]
            missing_check_keys = required - set(check)
            if missing_check_keys:
                raise ValueError(
                    f"{label} check {check_index} misses: "
                    f"{', '.join(sorted(missing_check_keys))}"
                )
            if "pattern" in check:
                try:
                    re.compile(check["pattern"])
                except (re.error, TypeError) as error:
                    raise ValueError(
                        f"{label} check {check_index} has invalid regex: {error}"
                    ) from error
            if "count" in check and (
                not isinstance(check["count"], int) or check["count"] < 0
            ):
                raise ValueError(
                    f"{label} check {check_index} needs a non-negative count"
                )

    return cases


def install_capability_skills(workspace: Path, names: list[str]) -> None:
    for name in names:
        skill_name, description, body = CAPABILITY_SKILLS[name]
        content = (
            "---\n"
            f"name: {skill_name}\n"
            f"description: {description}\n"
            "---\n\n"
            f"{body}"
        )
        write_text(
            workspace / ".agents" / "skills" / skill_name / "SKILL.md",
            content,
        )


def create_pack_source(workspace: Path) -> Path:
    source = workspace / "fixtures" / "wiki-pack"
    write_text(
        source / "guru-pack.json",
        json.dumps(
            {
                "schema_version": 1,
                "kind": "wiki",
                "elements": ["valuation/valuation-checklist.md"],
            },
            indent=2,
        ),
    )
    write_text(
        source / "valuation" / "valuation-checklist.md",
        record(
            "wiki:valuation-checklist",
            "Valuation checklist",
            "Reusable questions for comparing price with normalized economics.",
            "## Checklist\n\nReview normalized cash flow, reinvestment needs, balance-sheet claims, and scenario ranges.\n\n## Limits\n\nRefresh issuer facts and market prices before use.",
        ),
    )
    run_checked(["git", "init", "-q"], cwd=source)
    run_checked(["git", "config", "user.name", "Guru E2E"], cwd=source)
    run_checked(
        ["git", "config", "user.email", "guru-e2e@example.invalid"],
        cwd=source,
    )
    run_checked(["git", "add", "guru-pack.json", "valuation"], cwd=source)
    run_checked(["git", "commit", "-q", "-m", "fixture"], cwd=source)
    return source


def pack_fixture_environment(source: Path) -> dict[str, str]:
    return {
        "GIT_CONFIG_COUNT": "1",
        "GIT_CONFIG_KEY_0": f"url.{source.resolve().as_uri()}.insteadOf",
        "GIT_CONFIG_VALUE_0": PACK_FIXTURE_URL,
    }


def seed_large_corpus(workspace: Path) -> None:
    topics = [
        "accounting",
        "fixed-income",
        "market-structure",
        "quantum",
        "energy",
        "insurance",
        "payments",
        "supply-chain",
        "biotechnology",
        "portfolio-risk",
    ]
    entities = [
        "ticker:ALP",
        "ticker:BRV",
        "ticker:CRN",
        "ticker:DLT",
        "ticker:EVR",
        "ticker:FLX",
        "ticker:GRD",
        "ticker:HLX",
        "ticker:ION",
        "ticker:JDE",
    ]

    def metadata(index: int, topic: str, alias_kind: str) -> str:
        return (
            f"entities:\n  - {entities[index % len(entities)]}\n"
            f"aliases:\n  - {topic}-{alias_kind}-{index:03d}\n"
            f"tags:\n  - {topic}\n  - large-corpus"
        )

    # The generic population is deliberately broad, aliased, and noisy.
    kind_specs = [
        (
            "wiki",
            89,
            "concept",
            "## Concept\n\nA stable reusable domain term.\n\n"
            "## Limits\n\nCurrent observations require separate Evidence.",
        ),
        (
            "lens",
            29,
            "interpretation",
            "## Assumptions and applicability\n\nApply only when the causal "
            "conditions hold.\n\n## Failure indicators\n\nReject when the "
            "indicator and observed outcome diverge.",
        ),
        (
            "method",
            29,
            "workflow",
            "## Inputs\n\nValidated observations and a cutoff.\n\n"
            "## Procedure\n\nNormalize inputs and test a downside case.\n\n"
            "## Outputs and limits\n\nReport assumptions and validation.",
        ),
    ]
    for kind, count, label, body in kind_specs:
        for index in range(count):
            topic = topics[index % len(topics)]
            write_text(
                workspace
                / "guru-maker"
                / kind
                / topic
                / f"{label}-{index:03d}.md",
                record(
                    f"{kind}:large-{label}-{index:03d}",
                    f"{topic.replace('-', ' ').title()} {label} {index:03d}",
                    f"Reusable {topic} {label} number {index:03d}.",
                    body,
                    fields=metadata(index, topic, label),
                ),
            )

    # Wiki 90: exact alias target plus 89 generic Wiki records.
    write_text(
        workspace
        / "guru-maker"
        / "wiki"
        / "accounting"
        / "cross-border-working-capital.md",
        record(
            "wiki:cross-border-working-capital",
            "Cross-border working capital cycle",
            "How currency, settlement, and local payment terms shape cash conversion.",
            (
                "## Concept\n\nCross-border working capital links cash conversion "
                "to local payment terms, settlement timing, and currency "
                "translation.\n\n## Research use\n\nCompare reported cycles "
                "across currencies.\n\n## Limits\n\nThe current note does not "
                "define which movements represent operating cash generation."
            ),
            fields=(
                "aliases:\n"
                "  - CBWC\n"
                "  - cash conversion across currencies\n"
                "tags:\n"
                "  - working-capital\n"
                "  - foreign-exchange"
            ),
        ),
    )

    # Lens 30: one incomplete target plus 29 generic Lenses.
    write_text(
        workspace
        / "guru-maker"
        / "lens"
        / "infrastructure"
        / "contracted-demand-quality.md",
        record(
            "lens:contracted-demand-quality",
            "Contracted demand quality",
            "Interpret contracted backlog before treating it as durable demand.",
            (
                "## Assumptions and applicability\n\nUse reported contracted "
                "backlog as a demand signal.\n\n## Failure indicators\n\n"
                "Reconsider when cancellations rise."
            ),
            fields=(
                "aliases:\n"
                "  - CDQ\n"
                "  - backlog quality lens\n"
                "tags:\n"
                "  - infrastructure\n"
                "  - demand-quality"
            ),
        ),
    )

    # Method 30: one incomplete target plus 29 generic Methods.
    write_text(
        workspace
        / "guru-maker"
        / "method"
        / "fixed-income"
        / "liquidity-adjusted-spread-review.md",
        record(
            "method:liquidity-adjusted-spread-review",
            "Liquidity-adjusted spread review",
            "Recurring comparison of bond spread compensation and liquidity.",
            (
                "## Inputs\n\nOption-adjusted spread and modified duration.\n\n"
                "## Procedure\n\nRank bonds using raw option-adjusted spread alone."
                "\n\n## Limits\n\nQuote depth and transaction cost are omitted."
            ),
            fields=(
                "aliases:\n"
                "  - LASR\n"
                "  - spread after liquidity cost\n"
                "tags:\n"
                "  - fixed-income\n"
                "  - liquidity"
            ),
        ),
    )

    # Evidence 110: three named histories plus 107 generic observations.
    named_evidence = [
        (
            "orion/margin-2026-q1.md",
            "evidence:orion-margin-2026-q1",
            "Orion Systems 2026 Q1 operating margin",
            "Operating margin was 18% and cost absorption was described as stable.",
            "https://example.invalid/orion/2026-q1",
            "2026-Q1",
            "ticker:ORN",
        ),
        (
            "helix/duration-2026-q1.md",
            "evidence:helix-duration-2026-q1",
            "Helix 2026 Q1 duration compensation",
            "Spread compensation was 34 basis points per unit of modified duration with adequate depth.",
            "https://example.invalid/helix/2026-q1",
            "2026-Q1",
            "ticker:HLX",
        ),
        (
            "helix/duration-2026-q2.md",
            "evidence:helix-duration-2026-q2",
            "Helix 2026 Q2 duration compensation",
            "Spread compensation fell to 19 basis points per unit of modified duration and depth fell below threshold.",
            "https://example.invalid/helix/2026-q2",
            "2026-Q2",
            "ticker:HLX",
        ),
    ]
    for path, element_id, title, observation, source, period, entity in named_evidence:
        write_text(
            workspace / "guru-maker" / "evidence" / path,
            record(
                element_id,
                title,
                f"Source-bound {title.lower()} observation.",
                f"## Observation\n\n{observation}",
                source=source,
                fields=f"period: {period}\nentities:\n  - {entity}",
            ),
        )
    for index in range(107):
        topic = topics[index % len(topics)]
        write_text(
            workspace
            / "guru-maker"
            / "evidence"
            / topic
            / f"observation-{index:03d}.md",
            record(
                f"evidence:large-observation-{index:03d}",
                f"{topic.replace('-', ' ').title()} observation {index:03d}",
                f"Source-bound {topic} observation number {index:03d}.",
                (
                    f"## Observation\n\nThe synthetic measure was {100 + index} "
                    "units.\n\n## Limits\n\nNo investment conclusion follows."
                ),
                source=(
                    "https://example.invalid/large-corpus/evidence/"
                    f"{index:03d}"
                ),
                fields=(
                    f"period: 2025-Q{index % 4 + 1}\n"
                    + metadata(index, topic, "observation")
                ),
            ),
        )

    # Decision 45: an original and challenger plus 43 unrelated histories.
    write_text(
        workspace
        / "guru-maker"
        / "decision"
        / "helix"
        / "duration-original.md",
        record(
            "decision:helix-duration-original",
            "Original Helix duration overweight",
            "Earlier judgment to maintain the Helix bond overweight.",
            (
                "## Decision\n\nMaintain the overweight while spread compensation "
                "and quote depth remain adequate.\n\n## Invalidation\n\nReduce "
                "exposure if either falls below the portfolio threshold."
            ),
            fields=(
                "entities:\n"
                "  - ticker:HLX\n"
                "supports:\n"
                "  - evidence:helix-duration-2026-q1"
            ),
        ),
    )
    write_text(
        workspace
        / "guru-maker"
        / "decision"
        / "helix"
        / "duration-challenger.md",
        record(
            "decision:helix-duration-challenger",
            "Helix duration overweight challenger",
            "Later challenge after spread compression and weaker liquidity.",
            (
                "## Decision\n\nPrefer a reduction until compensation or depth "
                "recovers.\n\n## Invalidation\n\nReconsider if both recover."
            ),
            fields=(
                "entities:\n"
                "  - ticker:HLX\n"
                "supports:\n"
                "  - evidence:helix-duration-2026-q2\n"
                "contradicts:\n"
                "  - decision:helix-duration-original"
            ),
        ),
    )
    for index in range(43):
        write_text(
            workspace
            / "guru-maker"
            / "decision"
            / "archive"
            / f"allocation-{index:03d}.md",
            record(
                f"decision:large-allocation-{index:03d}",
                f"Historical allocation judgment {index:03d}",
                f"Point-in-time allocation judgment number {index:03d}.",
                (
                    "## Decision\n\nMaintain a bounded allocation.\n\n"
                    "## Invalidation\n\nReassess when evidence changes."
                ),
                fields=(
                    f"entities:\n  - {entities[index % len(entities)]}\n"
                    "supports:\n"
                    f"  - evidence:large-observation-{index:03d}"
                ),
            ),
        )

    write_text(
        workspace / "sources" / "orion-q2-update.md",
        """# Orion Systems 2026 Q2 reviewed update

- Source: https://example.invalid/orion/2026-q2
- Period: 2026-Q2
- Entity: ticker:ORN
- Observation: operating margin fell to 11% from 18% in Q1.
- Management explanation: transition costs exceeded the earlier absorption plan.
- Scope: this contradicts the earlier margin-stability observation but is not an allocation judgment.
""",
    )


def seed_fixture(name: str, workspace: Path) -> Path | None:
    sources = workspace / "sources"
    pack_source: Path | None = None

    if name in {"empty", "empty_initialized"}:
        return None

    if name == "us_equity_filings":
        write_text(
            workspace / "guru-maker" / "wiki" / "atlas-cloud-model.md",
            record(
                "wiki:atlas-cloud-model",
                "Atlas Cloud business model",
                "Stable context for the fictional U.S.-listed cloud infrastructure issuer.",
                "## Context\n\nThe 2025 review described capital intensity as falling.\n\n## Limits\n\nCompany guidance and current capex must be refreshed.",
                fields="entities:\n  - ticker:ATLS",
            ),
        )
        write_text(
            sources / "sec-q2.md",
            """# Atlas Cloud 2026 Q2 filing snapshot

- Fixture status: fictional SEC filing snapshot for isolated evaluation
- Source locator: https://www.sec.gov/Archives/edgar/data/example/atlas-2026q2.htm
- Filed: 2026-07-28
- Period: 2026-Q2
- Observation: capital expenditures were $2.4 billion, versus $1.6 billion in 2025-Q2.
- Management statement: full-year capex is expected to remain elevated while new data-center capacity is built.
- Limitation: this fixture contains no revenue guidance or valuation data.
""",
        )
    elif name == "treasury_curve":
        write_text(
            workspace / "guru-maker" / "method" / "duration-review.md",
            record(
                "method:duration-review",
                "Duration and curve review",
                "Repeatable method for interpreting Treasury curve changes.",
                "## Procedure\n\nCompare yields on the same basis and cutoff, calculate slope in basis points, and separate carry from parallel and non-parallel shifts.\n\n## Limits\n\nA two-point snapshot does not establish a regime.",
            ),
        )
        write_text(
            sources / "treasury-curve.csv",
            """as_of,instrument,yield_percent,source
2026-07-25,UST2Y,4.10,U.S. Treasury fixture
2026-07-25,UST10Y,4.35,U.S. Treasury fixture
2026-07-28,UST2Y,3.98,U.S. Treasury fixture
2026-07-28,UST10Y,4.31,U.S. Treasury fixture
""",
        )
    elif name == "fx_technical":
        write_text(
            sources / "eurusd-daily.csv",
            """date,venue,price_construction,eurusd_close
2026-07-15,FixtureFX,17:00 New York midpoint,1.1600
2026-07-16,FixtureFX,17:00 New York midpoint,1.1610
2026-07-17,FixtureFX,17:00 New York midpoint,1.1590
2026-07-20,FixtureFX,17:00 New York midpoint,1.1620
2026-07-21,FixtureFX,17:00 New York midpoint,1.1640
2026-07-22,FixtureFX,17:00 New York midpoint,1.1650
2026-07-23,FixtureFX,17:00 New York midpoint,1.1630
2026-07-24,FixtureFX,17:00 New York midpoint,1.1660
2026-07-27,FixtureFX,17:00 New York midpoint,1.1680
2026-07-28,FixtureFX,17:00 New York midpoint,1.1690
""",
        )
        write_text(
            sources / "signal-spec.md",
            """# Signal specification

Calculate a 3-observation simple moving average and a 7-observation simple
moving average from the supplied closes. A bullish crossover requires the
3-observation average to move from at or below the 7-observation average to
above it. No transaction-cost or intraday data is supplied.
""",
        )
    elif name == "commodity_stale":
        stale = record(
            "evidence:north-basin-inventory-2025",
            "North Basin inventory observation 2025",
            "A stale fictional inventory observation retained for historical context.",
            "Commercial crude inventories were reported at 118 million barrels. This observation must not be treated as current.",
            source="https://example.invalid/energy/2025-inventory",
            fields="period: 2025-W30",
        ).replace(
            "as_of: 2026-07-28T16:00:00-04:00",
            "as_of: 2025-07-25T10:30:00-04:00",
        )
        write_text(
            workspace / "guru-maker" / "evidence" / "north-basin-2025.md",
            stale,
        )
        write_text(
            sources / "crude-inventory.md",
            """# North Basin current inventory fixture

- Source: https://example.invalid/energy/2026-w30
- Observation cutoff: 2026-07-28T10:30:00-04:00
- Commercial crude inventories: 126 million barrels
- Week-over-week change: +4 million barrels
- Front/second-month spread: -$0.42 per barrel
- Limitation: no refinery utilization or export series is supplied.
""",
        )
    elif name == "multiasset_quant":
        write_text(
            sources / "equity-daily.csv",
            """date,series,return_percent,currency
2026-07-22,US_EQ_TR,0.40,USD
2026-07-23,US_EQ_TR,-0.20,USD
2026-07-24,US_EQ_TR,0.30,USD
2026-07-27,US_EQ_TR,0.10,USD
2026-07-28,US_EQ_TR,-0.10,USD
""",
        )
        write_text(
            sources / "treasury-weekly.csv",
            """week_end,series,return_basis_points,currency
2026-07-10,UST_7_10_PRICE,18,USD
2026-07-17,UST_7_10_PRICE,-6,USD
2026-07-24,UST_7_10_PRICE,9,USD
""",
        )
    elif name == "fund_comparison":
        write_text(
            sources / "fund-a.md",
            """# Fictional Fund A factsheet

- As of: 2026-06-30
- Net expense ratio: 0.08% after a temporary 0.02% waiver through 2026-12-31
- Benchmark: Atlas U.S. Large Cap Index
- Return convention: NAV total return
""",
        )
        write_text(
            sources / "fund-b.md",
            """# Fictional Fund B factsheet

- As of: 2026-07-25
- Gross expense ratio: 0.07%
- Net expense ratio: 0.07%
- Benchmark: Pioneer U.S. 500 Index
- Return convention: market-price total return
""",
        )
    elif name == "narrow_fact":
        write_text(
            sources / "pioneer-500.md",
            """# Pioneer 500 Fund fictional factsheet

- As of: 2026-06-30
- Gross expense ratio: 0.04%
- Net expense ratio: 0.04%
- This is an isolated evaluation fixture, not a real fund document.
""",
        )
    elif name == "memory_alias":
        write_text(
            workspace / "guru-maker" / "wiki" / "renewal-economics.md",
            record(
                "wiki:renewal-economics",
                "Renewal economics",
                "How net revenue retention connects expansion and churn.",
                "## Interpretation\n\nNet revenue retention (NRR), also called net dollar retention, combines churn, contraction, and expansion from an opening customer cohort.\n\n## Limits\n\nDefinitions vary when acquisitions, currency, or cohort rules differ.",
                fields="aliases:\n  - net dollar retention\n  - NRR",
            ),
        )
    elif name == "duration_correction":
        write_text(
            workspace / "guru-maker" / "method" / "spread-risk.md",
            record(
                "method:spread-risk",
                "Spread-risk price sensitivity",
                "Recurring procedure for first-order bond price sensitivity.",
                "## Procedure\n\nUse Macaulay duration directly as the first-order price sensitivity measure.\n\n## Limits\n\nRevisit the convention when yield and compounding assumptions differ.",
            ),
        )
    elif name == "wiki_neighborhood":
        wiki_records = [
            (
                "logical-qubits.md",
                "wiki:logical-qubits",
                "Logical qubits",
                "A logical unit used in fault-tolerant quantum computation.",
                "## Concept\n\nA logical qubit encodes quantum information.",
            ),
            (
                "surface-code.md",
                "wiki:surface-code",
                "Surface code",
                "A family of topological quantum error-correction schemes.",
                "## Concept\n\nSurface code uses repeated stabilizer measurements.",
            ),
            (
                "physical-qubits.md",
                "wiki:physical-qubits",
                "Physical qubits",
                "Hardware-level quantum information carriers.",
                "## Concept\n\nPhysical qubits are exposed to device noise.",
            ),
            (
                "syndrome-measurement.md",
                "wiki:syndrome-measurement",
                "Syndrome measurement",
                "Indirect detection of quantum errors.",
                "## Concept\n\nSyndromes reveal errors without reading logical state.",
            ),
            (
                "code-distance.md",
                "wiki:code-distance",
                "Code distance",
                "A measure of error-correcting code protection.",
                "## Concept\n\nDistance bounds the number of correctable errors.",
            ),
            (
                "fault-tolerance.md",
                "wiki:fault-tolerance",
                "Fault tolerance",
                "Reliable computation with imperfect components.",
                "## Concept\n\nFault-tolerant operations limit error propagation.",
            ),
            (
                "decoherence.md",
                "wiki:decoherence",
                "Decoherence",
                "Loss of quantum-state coherence through environmental coupling.",
                "## Concept\n\nDecoherence constrains physical operation fidelity.",
            ),
        ]
        for filename, memory_id, title, summary, body in wiki_records:
            write_text(
                workspace / "guru-maker" / "wiki" / "quantum" / filename,
                record(memory_id, title, summary, body),
            )
    elif name == "wiki_linked_neighborhood":
        write_text(
            workspace / "guru-maker" / "wiki" / "quantum" / "logical-qubits.md",
            record(
                "wiki:logical-qubits",
                "Logical qubits",
                "Error-corrected units built from physical qubits.",
                "## Concept\n\nA logical qubit protects encoded quantum information.\n\n## Limits\n\nResource overhead depends on the error-correction scheme and hardware error rates.",
                fields="see_also:\n  - wiki:surface-code",
            ),
        )
        write_text(
            workspace / "guru-maker" / "wiki" / "quantum" / "surface-code.md",
            record(
                "wiki:surface-code",
                "Surface code",
                "A topological quantum error-correction scheme.",
                "## Concept\n\nRepeated stabilizer measurements detect local errors.",
            ),
        )
        write_text(
            workspace / "guru-maker" / "wiki" / "quantum" / "decoherence.md",
            record(
                "wiki:decoherence",
                "Decoherence",
                "Loss of quantum-state coherence through environmental coupling.",
                "## Concept\n\nDecoherence constrains physical operation fidelity.",
            ),
        )
    elif name == "municipal_source":
        write_text(
            sources / "muni-notice.md",
            """# Fictional River City issuer notice

- Source: https://emma.msrb.org/example/river-city-notice
- Published: 2026-07-28
- Observation: the debt-service reserve was drawn by $4 million.
- Scope: issuer notice only; no rating action or market price is supplied.
""",
        )
    elif name == "consumer_lens":
        write_text(
            workspace / "guru-maker" / "lens" / "pricing-power.md",
            record(
                "lens:pricing-power",
                "Pricing power requires retained demand",
                "Interprets price increases together with volume and mix.",
                "## Lens\n\nNominal price gains are weak evidence of pricing power when unit volumes fall and mix shifts are unresolved.\n\n## Applicability and limits\n\nTest elasticity, customer retention, competitor response, and cost inflation. Temporary capacity constraints can make falling volume non-causal.",
            ),
        )
    elif name == "evidence_revision":
        write_text(
            workspace / "guru-maker" / "evidence" / "margin-old.md",
            record(
                "evidence:example-margin-old",
                "Example issuer margin observation",
                "Earlier fictional operating-margin observation.",
                "## Observation\n\nOperating margin was 18% in 2026-Q1.",
                source="https://example.invalid/issuer/q1",
                fields="period: 2026-Q1\nentities:\n  - ticker:EXMPL",
            ),
        )
        write_text(
            sources / "margin-update.md",
            """# Reviewed fictional issuer update

- Source: https://example.invalid/issuer/q2
- Period: 2026-Q2
- Entity: ticker:EXMPL
- Observation: operating margin fell to 9%, contradicting the earlier stability observation.
- Scope: supplied evaluation fixture; no investment judgment is requested.
""",
        )
    elif name == "equity_decision":
        write_text(
            workspace / "guru-maker" / "lens" / "capacity-payback.md",
            record(
                "lens:capacity-payback",
                "Capacity requires a bounded payback case",
                "Frames growth capex through utilization and cash payback.",
                "## Lens\n\nIncrease exposure only when demand visibility and utilization support a bounded payback range.\n\n## Limits\n\nDo not infer demand from capex alone.",
            ),
        )
        write_text(
            workspace / "guru-maker" / "evidence" / "atlas-q2.md",
            record(
                "evidence:atlas-q2-capex",
                "Atlas Cloud Q2 capex",
                "Fictional current capex and utilization observation.",
                "Capex rose to $2.4 billion while utilization was 72%. The portfolio limit permits at most a 50 basis-point increase.",
                source="https://example.invalid/atlas/q2",
                fields="period: 2026-Q2\nentities:\n  - ticker:ATLS",
            ),
        )
    elif name == "bond_decision":
        write_text(
            workspace / "guru-maker" / "method" / "bond-relative-value.md",
            record(
                "method:bond-relative-value",
                "Bond relative-value review",
                "Compares spread compensation with duration, liquidity, and downside.",
                "## Procedure\n\nCompare spread per unit of duration, liquidity, seniority, and stress loss.\n\n## Limits\n\nOne quote is not executable depth.",
            ),
        )
        write_text(
            workspace / "guru-maker" / "evidence" / "seven-year-bond.md",
            record(
                "evidence:seven-year-bond-quote",
                "Fictional seven-year bond quote",
                "Point-in-time spread, duration, and liquidity observation.",
                "Option-adjusted spread was 128 basis points, modified duration 5.9, and the supplied quote depth was $1 million.",
                source="https://example.invalid/bonds/quote",
                fields="period: 2026-07-28",
            ),
        )
    elif name == "fx_challenger":
        write_text(
            workspace / "guru-maker" / "evidence" / "carry-old.md",
            record(
                "evidence:eurusd-carry-old",
                "Earlier EUR/USD carry snapshot",
                "Earlier fictional policy-rate differential.",
                "The supplied differential favored USD by 120 basis points.",
                source="https://example.invalid/fx/old",
                fields="period: 2026-05-01",
            ),
        )
        write_text(
            workspace / "guru-maker" / "evidence" / "carry-new.md",
            record(
                "evidence:eurusd-carry-new",
                "Updated EUR/USD carry snapshot",
                "Later fictional policy-rate differential.",
                "The supplied differential favored USD by only 20 basis points.",
                source="https://example.invalid/fx/new",
                fields="period: 2026-07-28",
            ),
        )
        original = record(
            "decision:eurusd-carry-original",
            "Original EUR/USD carry judgment",
            "Earlier point-in-time judgment favoring USD carry.",
            "## Decision\n\nMaintain the USD carry expression.\n\n## Invalidation\n\nA material narrowing of the policy differential invalidates the rationale.",
            fields="supports:\n  - evidence:eurusd-carry-old",
        ).replace(
            "as_of: 2026-07-28T16:00:00-04:00",
            "as_of: 2026-05-01T16:00:00-04:00",
        )
        write_text(
            workspace / "guru-maker" / "decision" / "eurusd-carry-original.md",
            original,
        )
        write_text(
            workspace / "guru-maker" / "decision" / "eurusd-carry-reversal.md",
            record(
                "decision:eurusd-carry-reversal",
                "EUR/USD carry reversal review",
                "Later judgment contradicting the original carry rationale.",
                "## Decision\n\nDo not rely on the earlier carry thesis because its invalidation condition was met.",
                fields=(
                    "supports:\n"
                    "  - evidence:eurusd-carry-new\n"
                    "contradicts:\n"
                    "  - decision:eurusd-carry-original"
                ),
            ),
        )
    elif name == "portfolio_constraints_uninitialized":
        write_text(
            sources / "portfolio-constraints.md",
            """# Fictional balanced portfolio

- Equity weight: 72%; policy range: 45% to 65%
- Treasury weight: 18%; cash: 10%
- Twelve-month drawdown limit: 12%; current drawdown: 10.5%
- Cash need within 30 days: 8% of portfolio
- Tax, account, and transaction-cost details: not supplied
""",
        )
    elif name == "crypto_action":
        write_text(
            sources / "btc-context.md",
            """# User-provided action context

The user requested a BTC purchase brief but supplied no venue, account, size,
timing, price limit, tax context, or current market observation.
""",
        )
    elif name == "graph_records":
        write_text(
            workspace / "guru-maker" / "wiki" / "atlas-model.md",
            record(
                "wiki:atlas-model",
                "Atlas Cloud model",
                "Stable fictional business-model context.",
                "Atlas rents compute capacity under multi-year contracts.",
            ),
        )
        write_text(
            workspace / "guru-maker" / "evidence" / "atlas-renewal.md",
            record(
                "evidence:atlas-renewal",
                "Atlas renewal observation",
                "Current fictional renewal observation.",
                "Renewal rate was 91% in 2026-Q2.",
                source="https://example.invalid/atlas/renewal",
                fields="period: 2026-Q2",
            ),
        )
        write_text(
            workspace / "guru-maker" / "decision" / "atlas-hold.md",
            record(
                "decision:atlas-hold",
                "Hold Atlas Cloud",
                "Point-in-time hold judgment.",
                "## Decision\n\nHold within the existing limit.",
                fields=(
                    "uses:\n"
                    "  - wiki:atlas-model\n"
                    "supports:\n"
                    "  - evidence:atlas-renewal"
                ),
            ),
        )
    elif name == "equity_postmortem":
        write_text(
            sources / "atlas-retail-postmortem.md",
            """# Reviewed Atlas Retail loss postmortem

The team increased the position after a price increase but did not test whether
unit volumes were falling because of elasticity. The stock later declined.
Review conclusion: the process failed because price and volume were separated,
not because every post-price-increase volume decline predicts a loss.
""",
        )
        write_text(
            sources / "adjacent-counterexample.md",
            """# Adjacent reviewed counterexample

Fictional Harbor Tools raised prices while units fell because a low-margin
product was discontinued. Retention and contribution margin improved. The
counterexample shows that volume decline must be decomposed into elasticity,
mix, capacity, and deliberate pruning.
""",
        )
    elif name == "fx_lucky_trade":
        write_text(
            sources / "eurusd-lucky-trade.md",
            """# Reviewed EUR/USD trade

The entry relied on an unvalidated chart breakout with no event-risk check.
Profit arrived only after an unrelated central-bank surprise. Review agreement:
the outcome was lucky; a reusable process should define the signal, event-risk
screen, invalidation point, and cost assumptions before entry.
""",
        )
    elif name in {"pack_available", "pack_installed", "pack_installed_modified"}:
        pack_source = create_pack_source(workspace)
    elif name == "large_corpus":
        seed_large_corpus(workspace)
    elif name == "mixed_pack_elements":
        write_text(
            workspace / "guru-maker" / "wiki" / "valuation.md",
            record(
                "wiki:valuation-context",
                "Valuation context",
                "Stable valuation terminology.",
                "Enterprise value separates operating claims from financing claims.",
            ),
        )
        write_text(
            workspace / "guru-maker" / "method" / "valuation-review.md",
            record(
                "method:valuation-review",
                "Valuation review",
                "Repeatable valuation comparison procedure.",
                "## Procedure\n\nNormalize cash flow, claims, and share count.\n\n## Limits\n\nRefresh prices and estimates.",
            ),
        )
    elif name == "wiki_pack_open_link":
        write_text(
            workspace / "guru-maker" / "wiki" / "source.md",
            record(
                "wiki:pack-source",
                "Pack source concept",
                "A reusable concept whose navigation target is separate.",
                "## Concept\n\nThe source remains independently intelligible.",
                fields="see_also:\n  - wiki:pack-target",
            ),
        )
        write_text(
            workspace / "guru-maker" / "wiki" / "target.md",
            record(
                "wiki:pack-target",
                "Pack target concept",
                "The local target of authored Wiki navigation.",
                "## Concept\n\nThe target remains independently intelligible.",
            ),
        )
    else:
        raise ValueError(f"unknown fixture: {name}")

    return pack_source


def snapshot(workspace: Path) -> dict[str, str]:
    ignored_roots = {".agents", "fixtures", "sources", ".git"}
    result: dict[str, str] = {}
    for path in sorted(workspace.rglob("*")):
        relative = path.relative_to(workspace)
        if not relative.parts or relative.parts[0] in ignored_roots:
            continue
        if relative.name.startswith(".e2e-") or not path.is_file():
            continue
        result[relative.as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
    return result


def snapshot_agent_assets(workspace: Path) -> dict[str, str]:
    result: dict[str, str] = {}
    roots = [
        workspace / ".agents" / "skills",
        workspace / ".agents" / "shared" / "gurumaker",
        workspace / ".agents" / "runtime",
    ]
    for root in roots:
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*")):
            if path.is_file():
                relative = path.relative_to(workspace).as_posix()
                result[relative] = hashlib.sha256(path.read_bytes()).hexdigest()
    return result


def changed_paths(before: dict[str, str], after: dict[str, str]) -> dict[str, str]:
    changes: dict[str, str] = {}
    for path in sorted(set(before) | set(after)):
        if path not in before:
            changes[path] = "added"
        elif path not in after:
            changes[path] = "removed"
        elif before[path] != after[path]:
            changes[path] = "modified"
    return changes


def extract_commands(events: str) -> list[str]:
    commands: list[str] = []
    for line in events.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        item = event.get("item", {})
        command = item.get("command")
        if item.get("type") == "command_execution" and isinstance(command, str):
            if command not in commands:
                commands.append(command)
    return commands


def extract_successful_commands(events: str) -> list[str]:
    commands: list[str] = []
    for line in events.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        item = event.get("item", {})
        command = item.get("command")
        if (
            event.get("type") == "item.completed"
            and item.get("type") == "command_execution"
            and item.get("exit_code") == 0
            and isinstance(command, str)
            and command not in commands
        ):
            commands.append(command)
    return commands


def extract_non_shell_tools(events: str) -> list[dict[str, Any]]:
    tools: list[dict[str, Any]] = []
    for line in events.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        item = event.get("item", {})
        item_type = item.get("type")
        if item_type in {None, "agent_message", "command_execution"}:
            continue
        if event.get("type") != "item.completed":
            continue
        tools.append(
            {
                "type": item_type,
                "query": item.get("query"),
                "action": item.get("action"),
            }
        )
    return tools


def disabled_user_skill_config(codex_home: Path) -> str | None:
    skills_root = codex_home / "skills"
    if not skills_root.is_dir():
        return None
    paths = sorted(
        child / "SKILL.md"
        for child in skills_root.iterdir()
        if child.name != ".system" and (child / "SKILL.md").is_file()
    )
    if not paths:
        return None
    entries = ",".join(
        f"{{path={json.dumps(str(path.absolute()))},enabled=false}}"
        for path in paths
    )
    return f"skills.config=[{entries}]"


def external_guru_reads(commands: list[str], workspace: Path) -> list[str]:
    allowed_roots = [
        (workspace / ".agents" / "skills").resolve(),
        (workspace / ".agents" / "shared" / "gurumaker").resolve(),
    ]
    found: set[str] = set()
    pattern = re.compile(r"(?<![A-Za-z0-9._-])/[^\s'\";|&<>]+")
    for command in commands:
        for matched in pattern.findall(command):
            if not (
                matched.endswith("/SKILL.md")
                or "/shared/gurumaker/" in matched
                or "/.codex/skills/" in matched
                or "/.codex/plugins/" in matched
            ):
                continue
            candidate = Path(matched)
            if not any(
                candidate.resolve().is_relative_to(allowed_root)
                for allowed_root in allowed_roots
            ):
                found.add(matched)
    return sorted(found)


def expand_glob(pattern: str) -> list[str]:
    match = re.search(r"\{([^{}]+)\}", pattern)
    if not match:
        return [pattern]
    return [
        pattern[: match.start()] + option + pattern[match.end() :]
        for option in match.group(1).split(",")
    ]


def frontmatter_list(path: Path, field: str) -> list[str]:
    if not path.is_file():
        return []
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0].strip() != "---":
        return []
    frontmatter: list[str] = []
    for line in lines[1:]:
        if line.strip() == "---":
            break
        frontmatter.append(line)
    for index, line in enumerate(frontmatter):
        match = re.fullmatch(rf"{re.escape(field)}:\s*(.*)", line)
        if not match:
            continue
        inline = match.group(1).strip()
        if inline.startswith("[") and inline.endswith("]"):
            return [
                value.strip().strip("'\"")
                for value in inline[1:-1].split(",")
                if value.strip()
            ]
        values: list[str] = []
        for child in frontmatter[index + 1 :]:
            item = re.fullmatch(r"\s+-\s+(.+?)\s*", child)
            if item:
                values.append(item.group(1).strip().strip("'\""))
                continue
            if child.startswith((" ", "\t")) or not child.strip():
                continue
            break
        return values
    return []


def frontmatter_scalar(path: Path, field: str) -> str | None:
    if not path.is_file():
        return None
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    for line in lines[1:]:
        if line.strip() == "---":
            break
        match = re.fullmatch(rf"{re.escape(field)}:\s*(.+?)\s*", line)
        if match:
            return match.group(1).strip().strip("'\"")
    return None


def record_path(workspace: Path, record_id: str) -> Path | None:
    matches: list[Path] = []
    id_pattern = re.compile(rf"(?m)^id:\s*{re.escape(record_id)}\s*$")
    for path in workspace.glob("guru-maker/**/*.md"):
        if path.is_file() and id_pattern.search(path.read_text(encoding="utf-8")):
            matches.append(path)
    return matches[0] if len(matches) == 1 else None


def evaluate_check(
    check: dict[str, Any],
    *,
    workspace: Path,
    before: dict[str, str],
    after: dict[str, str],
    changes: dict[str, str],
    events: str,
    final: str,
    runtime: Path,
) -> tuple[bool, str]:
    check_type = check["type"]
    flags = re.IGNORECASE | re.DOTALL
    commands = extract_commands(events)
    successful_commands = extract_successful_commands(events)

    if check_type == "event_regex":
        ok = any(
            re.search(check["pattern"], command, flags)
            for command in successful_commands
        )
    elif check_type == "event_not_regex":
        ok = not any(
            re.search(check["pattern"], command, flags) for command in commands
        )
    elif check_type == "event_command_count":
        matching = [
            command
            for command in successful_commands
            if re.search(check["pattern"], command, flags)
        ]
        minimum = check.get("min", 0)
        maximum = check.get("max", len(successful_commands))
        ok = minimum <= len(matching) <= maximum
    elif check_type == "final_regex":
        ok = re.search(check["pattern"], final, flags) is not None
    elif check_type == "final_not_regex":
        ok = re.search(check["pattern"], final, flags) is None
    elif check_type == "change_any":
        ok = any(re.search(check["pattern"], path, flags) for path in changes)
    elif check_type == "change_none":
        ok = not any(re.search(check["pattern"], path, flags) for path in changes)
    elif check_type == "changed_exactly_one_matching":
        matching = [
            path for path in changes if re.search(check["pattern"], path, flags)
        ]
        ok = len(matching) == 1
    elif check_type == "changed_at_most_matching":
        matching = [
            path for path in changes if re.search(check["pattern"], path, flags)
        ]
        ok = len(matching) <= check["count"]
    elif check_type == "changed_only":
        ok = set(changes) == set(check["paths"])
    elif check_type == "changed_content_regex":
        path_pattern = check.get("path_pattern", ".*")
        matching_files = [
            workspace / path
            for path in changes
            if re.search(path_pattern, path, flags)
            and (workspace / path).is_file()
        ]
        ok = any(
            re.search(
                check["pattern"],
                path.read_text(encoding="utf-8"),
                flags,
            )
            is not None
            for path in matching_files
        )
    elif check_type == "changed_content_not_regex":
        path_pattern = check.get("path_pattern", ".*")
        matching_files = [
            workspace / path
            for path in changes
            if re.search(path_pattern, path, flags)
            and (workspace / path).is_file()
        ]
        ok = bool(matching_files) and all(
            re.search(
                check["pattern"],
                path.read_text(encoding="utf-8"),
                flags,
            )
            is None
            for path in matching_files
        )
    elif check_type == "path_exists":
        ok = (workspace / check["path"]).exists()
    elif check_type == "path_absent":
        ok = not (workspace / check["path"]).exists()
    elif check_type == "glob_min":
        matched = [
            path
            for pattern in expand_glob(check["glob"])
            for path in workspace.glob(pattern)
            if path.exists()
        ]
        ok = len(matched) >= check["count"]
    elif check_type == "glob_exact":
        matched = {
            path
            for pattern in expand_glob(check["glob"])
            for path in workspace.glob(pattern)
            if path.exists()
        }
        ok = len(matched) == check["count"]
    elif check_type == "mode_equals":
        mode_path = workspace / ".guru-maker" / "mode"
        ok = mode_path.is_file() and mode_path.read_text(encoding="utf-8").strip() == check["value"]
    elif check_type == "content_regex":
        path = workspace / check["path"]
        ok = path.is_file() and re.search(
            check["pattern"], path.read_text(encoding="utf-8"), flags
        ) is not None
    elif check_type == "content_not_regex":
        path = workspace / check["path"]
        ok = path.is_file() and re.search(
            check["pattern"], path.read_text(encoding="utf-8"), flags
        ) is None
    elif check_type == "frontmatter_list_equals":
        actual = frontmatter_list(workspace / check["path"], check["field"])
        ok = actual == check["values"]
    elif check_type == "record_frontmatter_list_equals":
        path = record_path(workspace, check["record_id"])
        ok = path is not None and frontmatter_list(path, check["field"]) == check["values"]
    elif check_type == "wiki_one_way_link":
        wiki_paths = sorted(
            path
            for path in workspace.glob("guru-maker/wiki/**/*.md")
            if path.is_file()
        )
        records = [
            (
                frontmatter_scalar(path, "id"),
                frontmatter_list(path, "see_also"),
            )
            for path in wiki_paths
        ]
        ok = (
            len(records) == 2
            and all(record_id is not None for record_id, _ in records)
            and sum(
                links == [other_id]
                for index, (_, links) in enumerate(records)
                for other_index, (other_id, _) in enumerate(records)
                if index != other_index
            )
            == 1
            and sum(not links for _, links in records) == 1
        )
    elif check_type == "glob_content_regex":
        matched_files: list[Path] = []
        for pattern in expand_glob(check["glob"]):
            matched_files.extend(path for path in workspace.glob(pattern) if path.is_file())
        ok = any(
            re.search(check["pattern"], path.read_text(encoding="utf-8"), flags)
            for path in matched_files
        )
    elif check_type == "glob_content_match_count":
        matched_files = {
            path
            for pattern in expand_glob(check["glob"])
            for path in workspace.glob(pattern)
            if path.is_file()
        }
        content_matches = [
            path
            for path in matched_files
            if re.search(
                check["pattern"],
                path.read_text(encoding="utf-8"),
                flags,
            )
        ]
        ok = len(content_matches) == check["count"]
    elif check_type == "initial_preserved":
        path = check["path"]
        ok = path in before and path in after and before[path] == after[path]
    elif check_type == "knowledge_valid":
        if not (workspace / ".guru-maker").is_dir():
            ok = False
        else:
            completed = subprocess.run(
                [
                    str(runtime),
                    "knowledge",
                    "check",
                    "--workspace",
                    str(workspace),
                ],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            ok = completed.returncode == 0
    else:
        raise ValueError(f"unknown check type: {check_type}")

    detail = json.dumps(check, sort_keys=True)
    return ok, detail


def prepare_case(
    case: dict[str, Any],
    case_dir: Path,
    runtime: Path,
) -> tuple[Path, dict[str, str], str, dict[str, str]]:
    workspace = case_dir / "workspace"
    workspace.mkdir(parents=True)
    (workspace / ".agents").mkdir()
    shutil.copytree(ROOT / "skills", workspace / ".agents" / "skills")
    shutil.copytree(ROOT / "shared", workspace / ".agents" / "shared")
    agent_runtime = workspace / ".agents" / "runtime" / "gurumaker"
    agent_runtime.parent.mkdir()
    shutil.copy2(runtime, agent_runtime)
    agent_runtime.chmod(0o755)
    install_capability_skills(workspace, case.get("capabilities", []))

    if case["workspace"] != "none":
        run_checked([str(runtime), "init", str(workspace)])
        if case["workspace"] == "off":
            run_checked(
                [str(runtime), "mode", "off", "--workspace", str(workspace)]
            )

    pack_source = seed_fixture(case["fixture"], workspace)
    fixture_environment: dict[str, str] = {}
    if pack_source:
        fixture_environment = pack_fixture_environment(pack_source)
    if case["fixture"] in {"pack_installed", "pack_installed_modified"}:
        if pack_source is None:
            raise AssertionError("pack fixture missing source")
        install_environment = os.environ.copy()
        install_environment.update(fixture_environment)
        run_checked(
            [
                str(runtime),
                "pack",
                "install",
                PACK_FIXTURE_URL,
                "--workspace",
                str(workspace),
            ],
            environment=install_environment,
        )
        if case["fixture"] == "pack_installed_modified":
            local = (
                workspace
                / "guru-maker"
                / "wiki"
                / "valuation"
                / "valuation-checklist.md"
            )
            local.write_text(
                local.read_text(encoding="utf-8")
                + "\n## Local project amendment\n\nRequire a downside liquidity scenario.\n",
                encoding="utf-8",
            )

    before = snapshot(workspace)
    prompt = case["prompt"].format(
        pack_source=PACK_FIXTURE_URL if pack_source else ""
    )
    fixture_environment["GURUMAKER_RUNTIME_BIN"] = str(agent_runtime)
    return workspace, before, prompt, fixture_environment


def run_case_in_directory(
    case: dict[str, Any],
    *,
    case_dir: Path,
    runtime: Path,
    model: str,
    reasoning_effort: str,
    timeout_seconds: int,
) -> dict[str, Any]:
    started = time.monotonic()
    workspace, before, prompt, fixture_environment = prepare_case(
        case, case_dir, runtime
    )
    agent_assets_before = snapshot_agent_assets(workspace)
    final_path = workspace / ".e2e-final.txt"
    user_home = Path(os.environ.get("HOME", str(Path.home())))
    source_codex_home = Path(
        os.environ.get("CODEX_HOME", str(user_home / ".codex"))
    )
    isolated_codex_home = case_dir / "codex-home"
    isolated_codex_home.mkdir()
    source_auth = source_codex_home / "auth.json"
    if source_auth.is_file():
        isolated_auth = isolated_codex_home / "auth.json"
        shutil.copy2(source_auth, isolated_auth)
        isolated_auth.chmod(0o600)
    command = [
        "codex",
        "exec",
        "--ephemeral",
        "--ignore-user-config",
        "--ignore-rules",
        "--skip-git-repo-check",
        "-C",
        str(workspace),
        "-s",
        "workspace-write",
        "-m",
        model,
        "-c",
        f'model_reasoning_effort="{reasoning_effort}"',
        "-c",
        'approval_policy="never"',
        "--json",
        "-o",
        str(final_path),
        prompt,
    ]
    if skill_config := disabled_user_skill_config(source_codex_home):
        command[1:1] = ["-c", skill_config]
    environment = os.environ.copy()
    environment.update(fixture_environment)
    isolated_agent_home = case_dir / "agent-home"
    isolated_agent_home.mkdir()
    environment.update(
        {
            "HOME": str(isolated_agent_home),
            "CODEX_HOME": str(isolated_codex_home),
            "XDG_CACHE_HOME": str(isolated_agent_home / ".cache"),
            "XDG_CONFIG_HOME": str(isolated_agent_home / ".config"),
            "XDG_DATA_HOME": str(isolated_agent_home / ".local" / "share"),
        }
    )

    return_code = 124
    events = ""
    stderr = ""
    timed_out = False
    try:
        completed = subprocess.run(
            command,
            cwd=workspace,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=timeout_seconds,
        )
        return_code = completed.returncode
        events = completed.stdout
        stderr = completed.stderr
    except subprocess.TimeoutExpired as error:
        timed_out = True
        events = error.stdout or ""
        stderr = error.stderr or ""
        if isinstance(events, bytes):
            events = events.decode("utf-8", errors="replace")
        if isinstance(stderr, bytes):
            stderr = stderr.decode("utf-8", errors="replace")

    final = (
        final_path.read_text(encoding="utf-8")
        if final_path.is_file()
        else ""
    )
    after = snapshot(workspace)
    agent_assets_after = snapshot_agent_assets(workspace)
    changes = changed_paths(before, after)
    commands = extract_commands(events)
    check_results: list[dict[str, Any]] = []
    for check in case["checks"]:
        ok, detail = evaluate_check(
            check,
            workspace=workspace,
            before=before,
            after=after,
            changes=changes,
            events=events,
            final=final,
            runtime=runtime,
        )
        check_results.append({"passed": ok, "check": check, "detail": detail})
    external_reads = external_guru_reads(commands, workspace)
    check_results.append(
        {
            "passed": not external_reads,
            "check": {"type": "project_local_skill_isolation"},
            "detail": (
                "all accessed Skills and Guru references were project-local"
                if not external_reads
                else f"external Guru reads: {', '.join(external_reads)}"
            ),
        }
    )
    agent_asset_changes = changed_paths(agent_assets_before, agent_assets_after)
    check_results.append(
        {
            "passed": not agent_asset_changes,
            "check": {"type": "project_local_contract_integrity"},
            "detail": (
                "project-local Skills, shared contracts, and Runtime were preserved"
                if not agent_asset_changes
                else "modified project-local agent assets: "
                + ", ".join(
                    f"{path} ({change})"
                    for path, change in agent_asset_changes.items()
                )
            ),
        }
    )

    deterministic_pass = (
        return_code == 0
        and not timed_out
        and all(item["passed"] for item in check_results)
    )
    artifact_contents: dict[str, str] = {}
    for path in changes:
        artifact = workspace / path
        if (
            artifact.is_file()
            and ".git" not in artifact.parts
            and artifact.stat().st_size <= 200_000
        ):
            try:
                artifact_contents[path] = artifact.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
    result = {
        "id": case["id"],
        "title": case["title"],
        "skill": case["skill"],
        "universe": case["universe"],
        "intent": case["intent"],
        "persona": case.get("persona", "experienced-or-neutral"),
        "model": model,
        "reasoning_effort": reasoning_effort,
        "prompt": prompt,
        "expected": case["expected"],
        "qualitative_focus": case["qualitative_focus"],
        "return_code": return_code,
        "timed_out": timed_out,
        "duration_seconds": round(time.monotonic() - started, 3),
        "deterministic_pass": deterministic_pass,
        "checks": check_results,
        "changed_paths": changes,
        "artifact_contents": artifact_contents,
        "commands": commands,
        "non_shell_tools": extract_non_shell_tools(events),
        "final": final,
        "_events": events,
        "_stderr": stderr,
    }
    return result


def run_case(
    case: dict[str, Any],
    *,
    runtime: Path,
    model: str,
    reasoning_effort: str,
    timeout_seconds: int,
) -> dict[str, Any]:
    case_dir = Path(tempfile.mkdtemp(prefix=f"guru-maker-e2e-{case['id']}-"))
    try:
        return run_case_in_directory(
            case,
            case_dir=case_dir,
            runtime=runtime,
            model=model,
            reasoning_effort=reasoning_effort,
            timeout_seconds=timeout_seconds,
        )
    finally:
        shutil.rmtree(case_dir)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run isolated Guru Maker agent E2E cases."
    )
    parser.add_argument("--cases-file", type=Path, default=DEFAULT_CASES)
    parser.add_argument("--case", action="append", dest="case_ids")
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument(
        "--model",
        help="Codex model ID (or set GURU_MAKER_E2E_MODEL)",
    )
    parser.add_argument(
        "--reasoning-effort",
        help=(
            "Codex reasoning effort "
            "(default: GURU_MAKER_E2E_REASONING_EFFORT or high)"
        ),
    )
    parser.add_argument("--timeout-seconds", type=int, default=900)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--prepare-only", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    configuration = json.loads(args.cases_file.read_text(encoding="utf-8"))
    cases = validate_configuration(configuration)
    if args.case_ids:
        selected = set(args.case_ids)
        cases = [case for case in cases if case["id"] in selected]
        missing = selected - {case["id"] for case in cases}
        if missing:
            raise SystemExit(f"unknown case IDs: {', '.join(sorted(missing))}")

    model = args.model or os.environ.get("GURU_MAKER_E2E_MODEL")
    reasoning_effort = args.reasoning_effort or os.environ.get(
        "GURU_MAKER_E2E_REASONING_EFFORT", "high"
    )
    if not model and not (args.dry_run or args.prepare_only):
        raise SystemExit(
            "agent execution requires --model or GURU_MAKER_E2E_MODEL"
        )
    if args.dry_run:
        print(
            json.dumps(
                {
                    "model": model,
                    "reasoning_effort": reasoning_effort,
                    "count": len(cases),
                    "ids": [case["id"] for case in cases],
                },
                indent=2,
            )
        )
        return 0

    timestamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    if args.output:
        output_dir = args.output.resolve()
        if output_dir == ROOT or ROOT in output_dir.parents:
            raise SystemExit(
                "E2E output must be outside the repository so parent AGENTS, "
                "skills, tests, and expected results cannot leak into cases"
            )
    else:
        output_dir = Path(
            tempfile.mkdtemp(prefix=f"guru-maker-e2e-{timestamp}-")
        ).resolve()

    output_dir.mkdir(parents=True, exist_ok=True)
    run_checked(["cargo", "build", "--locked"], cwd=ROOT)
    runtime = (ROOT / "target" / "debug" / "gurumaker").resolve()

    if args.prepare_only:
        failures: list[str] = []
        for case in cases:
            try:
                workspace, _, _, _ = prepare_case(
                    case, output_dir / "cases" / case["id"], runtime
                )
                if (workspace / ".guru-maker").is_dir():
                    run_checked(
                        [
                            str(runtime),
                            "knowledge",
                            "check",
                            "--workspace",
                            str(workspace),
                        ]
                    )
                print(f"{case['id']}: PREPARED", flush=True)
            except Exception as error:
                failures.append(case["id"])
                print(f"{case['id']}: PREPARE FAIL {error!r}", flush=True)
        print(f"prepared fixtures: {output_dir}")
        return 1 if failures else 0

    if args.jobs != 1:
        raise SystemExit(
            "strict E2E isolation requires --jobs 1 so no case workspace exists "
            "while another agent runs"
        )

    results: list[dict[str, Any]] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as executor:
        futures = {
            executor.submit(
                run_case,
                case,
                runtime=runtime,
                model=model,
                reasoning_effort=reasoning_effort,
                timeout_seconds=args.timeout_seconds,
            ): case["id"]
            for case in cases
        }
        for future in concurrent.futures.as_completed(futures):
            case_id = futures[future]
            try:
                result = future.result()
            except Exception as error:
                result = {
                    "id": case_id,
                    "deterministic_pass": False,
                    "harness_error": repr(error),
                }
            results.append(result)
            status = "PASS" if result.get("deterministic_pass") else "FAIL"
            print(f"{case_id}: {status}", flush=True)

    results.sort(key=lambda item: item["id"])
    persisted_results: list[dict[str, Any]] = []
    for result in results:
        case_output = output_dir / "cases" / result["id"]
        case_output.mkdir(parents=True)
        events = result.pop("_events", "")
        stderr = result.pop("_stderr", "")
        write_text(case_output / "events.jsonl", events)
        write_text(case_output / "stderr.txt", stderr)
        write_text(case_output / "result.json", json.dumps(result, indent=2))
        persisted_results.append(result)
    summary = {
        "schema_version": 1,
        "created_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "model": model,
        "reasoning_effort": reasoning_effort,
        "case_count": len(results),
        "deterministic_pass_count": sum(
            bool(result.get("deterministic_pass")) for result in results
        ),
        "qualitative_review_status": "pending",
        "results": persisted_results,
    }
    write_text(output_dir / "summary.json", json.dumps(summary, indent=2))
    print(f"results: {output_dir}")
    return 0 if summary["deterministic_pass_count"] == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
