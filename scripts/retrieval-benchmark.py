#!/usr/bin/env python3
"""Compare deterministic Guru Maker retrieval against the v0.0.3 Runtime."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import os
import platform
from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import tarfile
import tempfile
import time
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[1]
BASELINE_REF = "v0.0.3"
PARALLEL_SCAN_MIN_FILES = 256
MAX_SCAN_WORKERS = 8
KINDS = ("wiki", "lens", "method", "evidence", "decision")


def run(
    arguments: list[str],
    *,
    cwd: Path | None = None,
    capture: bool = True,
) -> str:
    completed = subprocess.run(
        arguments,
        cwd=cwd,
        check=True,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
        text=True,
    )
    return completed.stdout if capture else ""


def source_revision() -> str | None:
    try:
        revision = run(["git", "rev-parse", "--short=12", "HEAD"], cwd=ROOT).strip()
        dirty = bool(run(["git", "status", "--porcelain"], cwd=ROOT).strip())
        return revision + ("-dirty" if dirty else "")
    except (OSError, subprocess.CalledProcessError):
        return None


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def runtime_source_sha256(root: Path) -> str:
    paths = [root / "Cargo.toml", root / "Cargo.lock"]
    build_script = root / "build.rs"
    if build_script.is_file():
        paths.append(build_script)
    paths.extend(sorted(path for path in (root / "src").rglob("*") if path.is_file()))
    if any(not path.is_file() for path in paths):
        raise SystemExit(f"invalid Runtime source root: {root}")

    digest = hashlib.sha256()
    for path in paths:
        digest.update(path.relative_to(root).as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def available_cpu_count() -> int:
    if hasattr(os, "sched_getaffinity"):
        try:
            return max(1, len(os.sched_getaffinity(0)))
        except OSError:
            pass
    return max(1, os.cpu_count() or 1)


def scan_worker_count(files: int, available_cpus: int) -> int:
    if files < PARALLEL_SCAN_MIN_FILES:
        return 1
    return min(available_cpus, MAX_SCAN_WORKERS, files)


def kind_document_count(documents: int, kinds: tuple[str, ...]) -> int:
    wanted = set(kinds)
    return sum(1 for index in range(documents) if KINDS[index % len(KINDS)] in wanted)


def percentile(samples: list[float], fraction: float) -> float:
    ordered = sorted(samples)
    index = max(0, math.ceil(len(ordered) * fraction) - 1)
    return ordered[index]


def timing_summary(samples: list[float], runs: int) -> dict[str, float | None]:
    return {
        "warm_p50": round(statistics.median(samples), 3),
        "warm_p95": round(percentile(samples, 0.95), 3) if runs >= 20 else None,
    }


def timed_pair(
    baseline: Callable[[], Any], candidate: Callable[[], Any], runs: int
) -> tuple[dict[str, float | None], dict[str, float | None]]:
    baseline()
    candidate()
    samples = {"v0.0.3": [], "candidate": []}
    operations = {"v0.0.3": baseline, "candidate": candidate}
    for run_index in range(runs):
        order = (
            ("v0.0.3", "candidate")
            if run_index % 2 == 0
            else ("candidate", "v0.0.3")
        )
        for label in order:
            started = time.perf_counter()
            operations[label]()
            samples[label].append((time.perf_counter() - started) * 1_000)
    return (
        timing_summary(samples["v0.0.3"], runs),
        timing_summary(samples["candidate"], runs),
    )


def safe_extract_tar(payload: bytes, destination: Path) -> None:
    destination = destination.resolve()
    with tarfile.open(fileobj=io.BytesIO(payload), mode="r:") as archive:
        for member in archive.getmembers():
            target = (destination / member.name).resolve()
            if target != destination and destination not in target.parents:
                raise RuntimeError(f"unsafe path in {BASELINE_REF} archive: {member.name}")
        archive.extractall(destination)


def build_baseline(temporary: Path) -> tuple[Path, Path]:
    completed = subprocess.run(
        ["git", "-C", str(ROOT), "archive", "--format=tar", BASELINE_REF],
        check=True,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    source_root = temporary / "baseline-source"
    source_root.mkdir()
    safe_extract_tar(completed.stdout, source_root)
    target_root = temporary / "baseline-target"
    run(
        [
            "cargo",
            "build",
            "--release",
            "--locked",
            "--manifest-path",
            str(source_root / "Cargo.toml"),
            "--target-dir",
            str(target_root),
        ],
        capture=False,
    )
    suffix = ".exe" if os.name == "nt" else ""
    return target_root / "release" / f"gurumaker{suffix}", source_root


def choose_index(documents: int, residue: int, fraction: float) -> int:
    candidate = min(documents - 1, int(documents * fraction))
    candidate -= (candidate - residue) % len(KINDS)
    return candidate if candidate >= residue else residue


def record(index: int, target: int, union_target: int, excluded_target: int) -> str:
    kind = KINDS[index % len(KINDS)]
    identifier = f"{kind}:scale-record-{index:05}"
    aliases = []
    if index == target:
        aliases.extend(["advanced packaging bottleneck", "첨단 패키징 병목"])
    if index == union_target:
        aliases.append("repeated kind union target")
    alias_line = f"aliases: [{', '.join(aliases)}]\n" if aliases else "aliases: []\n"
    source_line = (
        f"source: https://example.test/scale/{index}\n" if kind == "evidence" else ""
    )
    title = (
        "Repeated kind union target"
        if index in (union_target, excluded_target)
        else f"Scale record {index:05}"
    )
    summary = (
        "HBM packaging capacity is the uniquely labeled retrieval target."
        if index == target
        else "A synthetic liquidity and capital-allocation memory distractor."
    )
    numeric = (
        "Operating margin changed +5.0% in 2026-Q2."
        if index == target
        else f"Scenario {index} preserves exact period and ratio notation."
    )
    body = "\n\n".join(
        [
            "# Business context\n\n"
            + " ".join(["Demand, capacity, pricing, and reinvestment shape returns."] * 18),
            "# Liquidity\n\n"
            + " ".join(["Liquidity remains a common broad-query distractor."] * 18),
            f"# Numeric discipline\n\n{numeric}",
            "# Supply chain\n\n"
            + " ".join(["Suppliers, lead times, and utilization require current evidence."] * 18),
            "# Decision use\n\n"
            + " ".join(["Re-test assumptions before applying historical memory."] * 18),
            "# Limits\n\n"
            + " ".join(["This synthetic record is only a scale and quality fixture."] * 18),
        ]
    )
    return (
        "---\n"
        f"id: {identifier}\n"
        f"title: {title}\n"
        f"summary: {summary}\n"
        "as_of: 2026-07-31T12:00:00+09:00\n"
        f"{alias_line}"
        f"tags: [scale, liquidity, cohort-{index % 17}]\n"
        f"entities: [ticker:S{index:05}]\n"
        f"{source_line}"
        "---\n\n"
        f"{body}\n"
    )


def search(
    binary: Path,
    workspace: Path,
    query: str,
    *,
    candidates: bool = False,
    kinds: tuple[str, ...] = (),
    entity: str | None = None,
    limit: int = 8,
) -> list[dict[str, Any]]:
    arguments = [
        str(binary),
        "knowledge",
        "search",
        query,
        "--workspace",
        str(workspace),
        "--limit",
        str(limit),
    ]
    for kind in kinds:
        arguments.extend(["--kind", kind])
    if entity:
        arguments.extend(["--entity", entity])
    if candidates:
        arguments.append("--candidates")
    arguments.append("--json")
    return json.loads(run(arguments))


def read(binary: Path, workspace: Path, identifier: str) -> dict[str, Any]:
    return json.loads(
        run(
            [
                str(binary),
                "knowledge",
                "read",
                identifier,
                "--workspace",
                str(workspace),
                "--json",
            ]
        )
    )


def prepare(
    binary: Path, parent: Path, documents: int
) -> tuple[Path, str, str, int, int, int, int]:
    workspace = parent / f"corpus-{documents}"
    run([str(binary), "init", str(workspace), "--json"])
    target = choose_index(documents, 0, 0.90)
    union_target = choose_index(documents, 1, 0.75)
    excluded_target = choose_index(documents, 3, 0.70)
    for index in range(documents):
        kind = KINDS[index % len(KINDS)]
        directory = workspace / "guru-maker" / kind
        directory.mkdir(parents=True, exist_ok=True)
        (directory / f"scale-record-{index:05}.md").write_text(
            record(index, target, union_target, excluded_target), encoding="utf-8"
        )
    target_id = f"wiki:scale-record-{target:05}"
    union_target_id = f"lens:scale-record-{union_target:05}"
    markdown_bytes = sum(
        path.stat().st_size
        for kind in KINDS
        for path in (workspace / "guru-maker" / kind).glob("*.md")
    )
    check = json.loads(
        run(
            [
                str(binary),
                "knowledge",
                "check",
                "--workspace",
                str(workspace),
                "--json",
            ]
        )
    )
    if not check["valid"] or check["documents"] != documents:
        raise RuntimeError(f"invalid generated corpus: {check}")
    return (
        workspace,
        target_id,
        union_target_id,
        target,
        union_target,
        excluded_target,
        markdown_bytes,
    )


def quality(
    binary: Path,
    workspace: Path,
    target_id: str,
    union_target_id: str,
    target_index: int,
) -> dict[str, bool]:
    kinds = ("wiki", "lens", "wiki")
    target_entity = f"ticker:S{target_index:05}"
    candidate = search(
        binary, workspace, "advanced packaging bottleneck", candidates=True
    )[0]
    entity_results = search(
        binary, workspace, "liquidity", entity=target_entity
    )
    return {
        "phrase_top1": search(binary, workspace, "HBM packaging")[0]["id"]
        == target_id,
        "alias_top1": candidate["id"] == target_id,
        "candidate_compact": "text" not in candidate and "match_tier" in candidate,
        "hangul_alias_top1": search(binary, workspace, "첨단패키징병목")[0]["id"]
        == target_id,
        "exact_id_top1": search(binary, workspace, target_id)[0]["id"] == target_id,
        "signed_number_top1": search(binary, workspace, "+5.0%")[0]["id"]
        == target_id,
        "wrong_sign_no_match": not search(binary, workspace, "-5.0%"),
        "no_match": not search(binary, workspace, "cocoa basis teleportation"),
        "entity_filter_exact": len(entity_results) == 1
        and entity_results[0]["id"] == target_id,
        "repeated_kind_default_top1": search(
            binary, workspace, "repeated kind union target", kinds=kinds
        )[0]["id"]
        == union_target_id,
        "repeated_kind_candidate_top1": search(
            binary,
            workspace,
            "repeated kind union target",
            candidates=True,
            kinds=kinds,
        )[0]["id"]
        == union_target_id,
        "exact_read": read(binary, workspace, target_id)["document"]["id"]
        == target_id,
    }


def parity(
    baseline: Path,
    candidate: Path,
    workspace: Path,
    target_id: str,
    target_index: int,
) -> dict[str, bool]:
    repeated = ("wiki", "lens", "wiki")
    target_entity = f"ticker:S{target_index:05}"
    return {
        "default_search": search(baseline, workspace, "HBM packaging")
        == search(candidate, workspace, "HBM packaging"),
        "broad_search": search(baseline, workspace, "liquidity")
        == search(candidate, workspace, "liquidity"),
        "candidate_search": search(
            baseline, workspace, "advanced packaging bottleneck", candidates=True
        )
        == search(candidate, workspace, "advanced packaging bottleneck", candidates=True),
        "broad_candidate_search": search(
            baseline, workspace, "liquidity", candidates=True
        )
        == search(candidate, workspace, "liquidity", candidates=True),
        "entity_filter_search": search(
            baseline, workspace, "liquidity", entity=target_entity
        )
        == search(candidate, workspace, "liquidity", entity=target_entity),
        "repeated_kind_default": search(
            baseline, workspace, "repeated kind union target", kinds=repeated
        )
        == search(candidate, workspace, "repeated kind union target", kinds=repeated),
        "repeated_kind_candidate": search(
            baseline,
            workspace,
            "repeated kind union target",
            candidates=True,
            kinds=repeated,
        )
        == search(
            candidate,
            workspace,
            "repeated kind union target",
            candidates=True,
            kinds=repeated,
        ),
        "exact_read": read(baseline, workspace, target_id)
        == read(candidate, workspace, target_id),
    }


def timing_operations(
    binary: Path, workspace: Path, target_id: str
) -> dict[str, Callable[[], Any]]:
    repeated = ("wiki", "lens", "wiki")
    return {
        "narrow_search": lambda: search(binary, workspace, "HBM packaging"),
        "broad_search": lambda: search(binary, workspace, "liquidity"),
        "candidate_search": lambda: search(
            binary, workspace, "advanced packaging bottleneck", candidates=True
        ),
        "repeated_kind_search": lambda: search(
            binary, workspace, "repeated kind union target", kinds=repeated
        ),
        "repeated_kind_candidate_search": lambda: search(
            binary,
            workspace,
            "repeated kind union target",
            candidates=True,
            kinds=repeated,
        ),
        "exact_read": lambda: read(binary, workspace, target_id),
    }


def compare_timings(
    baseline: Path,
    candidate: Path,
    workspace: Path,
    target_id: str,
    runs: int,
) -> dict[str, Any]:
    baseline_operations = timing_operations(baseline, workspace, target_id)
    candidate_operations = timing_operations(candidate, workspace, target_id)
    results = {}
    for name in baseline_operations:
        baseline_timing, candidate_timing = timed_pair(
            baseline_operations[name], candidate_operations[name], runs
        )
        baseline_p50 = baseline_timing["warm_p50"]
        candidate_p50 = candidate_timing["warm_p50"]
        results[name] = {
            "v0.0.3": baseline_timing,
            "candidate": candidate_timing,
            "candidate_over_v0.0.3_p50": (
                round(candidate_p50 / baseline_p50, 3)
                if baseline_p50 and candidate_p50 is not None
                else None
            ),
        }
    return results


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Benchmark lexical retrieval against the v0.0.3 Runtime."
    )
    parser.add_argument("--binary", type=Path, help="candidate Runtime binary")
    parser.add_argument(
        "--candidate-revision", help="revision label for an external --binary"
    )
    parser.add_argument(
        "--baseline-binary",
        type=Path,
        help="prebuilt v0.0.3 Runtime; the default builds the v0.0.3 tag",
    )
    parser.add_argument("--sizes", type=int, nargs="+", default=[100, 1000, 5000])
    parser.add_argument("--runs", type=int, default=20)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--keep", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.runs < 1 or any(size < len(KINDS) * 2 for size in args.sizes):
        raise SystemExit("--runs must be positive and every --sizes value must be at least 10")
    if args.binary and not args.candidate_revision:
        raise SystemExit("external --binary requires --candidate-revision")
    if not args.binary and args.candidate_revision:
        raise SystemExit("--candidate-revision requires --binary")

    temporary = Path(tempfile.mkdtemp(prefix="guru-maker-retrieval-benchmark-"))
    try:
        if args.binary:
            candidate_binary = args.binary.resolve()
            candidate_revision = args.candidate_revision
            candidate_source_sha256 = None
        else:
            run(["cargo", "build", "--release", "--locked"], cwd=ROOT, capture=False)
            suffix = ".exe" if os.name == "nt" else ""
            candidate_binary = ROOT / "target" / "release" / f"gurumaker{suffix}"
            candidate_revision = source_revision()
            candidate_source_sha256 = runtime_source_sha256(ROOT)

        baseline_source = None
        if args.baseline_binary:
            baseline_binary = args.baseline_binary.resolve()
        else:
            baseline_binary, baseline_source = build_baseline(temporary)
        if not candidate_binary.is_file() or not baseline_binary.is_file():
            raise SystemExit("candidate or baseline Guru Maker binary was not found")

        baseline_version = run([str(baseline_binary), "--version"]).strip()
        if baseline_version != "0.0.3":
            raise SystemExit(
                f"--baseline-binary must report exactly 0.0.3, got {baseline_version!r}"
            )
        baseline_commit = run(
            ["git", "rev-parse", f"{BASELINE_REF}^{{commit}}"], cwd=ROOT
        ).strip()
        candidate_version = run([str(candidate_binary), "--version"]).strip()
        available_cpus = available_cpu_count()

        cases = []
        corpus_root = temporary / "corpora"
        corpus_root.mkdir()
        for documents in args.sizes:
            (
                workspace,
                target_id,
                union_target_id,
                target_index,
                union_target_index,
                excluded_target_index,
                markdown_bytes,
            ) = prepare(candidate_binary, corpus_root, documents)
            baseline_quality = quality(
                baseline_binary,
                workspace,
                target_id,
                union_target_id,
                target_index,
            )
            candidate_quality = quality(
                candidate_binary,
                workspace,
                target_id,
                union_target_id,
                target_index,
            )
            parity_results = parity(
                baseline_binary,
                candidate_binary,
                workspace,
                target_id,
                target_index,
            )
            failures = [
                f"v0.0.3:{name}"
                for name, passed in baseline_quality.items()
                if not passed
            ]
            failures.extend(
                f"candidate:{name}"
                for name, passed in candidate_quality.items()
                if not passed
            )
            failures.extend(
                f"parity:{name}" for name, passed in parity_results.items() if not passed
            )
            if failures:
                raise RuntimeError(
                    f"retrieval failures at {documents} documents: {', '.join(failures)}"
                )

            cases.append(
                {
                    "documents": documents,
                    "markdown_bytes": markdown_bytes,
                    "bytes_per_document": round(markdown_bytes / documents, 1),
                    "target_id": target_id,
                    "target_index": target_index,
                    "repeated_kind_target_id": union_target_id,
                    "repeated_kind_target_index": union_target_index,
                    "excluded_evidence_target_index": excluded_target_index,
                    "scan_workers": {
                        "v0.0.3": {
                            "narrow_search": 1,
                            "broad_search": 1,
                            "candidate_search": 1,
                            "repeated_kind_search": 1,
                            "repeated_kind_candidate_search": 1,
                            "exact_read": 1,
                        },
                        "candidate": {
                            "narrow_search": scan_worker_count(
                                documents, available_cpus
                            ),
                            "broad_search": scan_worker_count(
                                documents, available_cpus
                            ),
                            "candidate_search": scan_worker_count(
                                documents, available_cpus
                            ),
                            "repeated_kind_search": scan_worker_count(
                                kind_document_count(documents, ("wiki", "lens")),
                                available_cpus,
                            ),
                            "repeated_kind_candidate_search": scan_worker_count(
                                kind_document_count(documents, ("wiki", "lens")),
                                available_cpus,
                            ),
                            "exact_read": scan_worker_count(
                                kind_document_count(documents, ("wiki",)),
                                available_cpus,
                            ),
                        },
                    },
                    "quality": {
                        "v0.0.3": baseline_quality,
                        "candidate": candidate_quality,
                        "output_parity": parity_results,
                    },
                    "timing_ms": compare_timings(
                        baseline_binary,
                        candidate_binary,
                        workspace,
                        target_id,
                        args.runs,
                    ),
                }
            )

        result = {
            "schema_version": 1,
            "baseline": {
                "ref": BASELINE_REF,
                "commit": baseline_commit,
                "version": baseline_version,
                "binary": str(baseline_binary),
                "binary_sha256": sha256_file(baseline_binary),
                "runtime_source_sha256": (
                    runtime_source_sha256(baseline_source) if baseline_source else None
                ),
            },
            "candidate": {
                "version": candidate_version,
                "source_revision": candidate_revision,
                "binary": str(candidate_binary),
                "binary_sha256": sha256_file(candidate_binary),
                "runtime_source_sha256": candidate_source_sha256,
            },
            "benchmark_sha256": sha256_file(Path(__file__).resolve()),
            "platform": platform.platform(),
            "machine": platform.machine(),
            "logical_cpus": os.cpu_count(),
            "available_cpus": available_cpus,
            "scan_worker_policy": {
                "parallel_min_files": PARALLEL_SCAN_MIN_FILES,
                "maximum_workers": MAX_SCAN_WORKERS,
            },
            "runs": args.runs,
            "timing_order": "alternating AB/BA within each operation",
            "cases": cases,
        }
        serialized = json.dumps(result, indent=2, ensure_ascii=False) + "\n"
        if args.output:
            args.output.write_text(serialized, encoding="utf-8")
        print(serialized, end="")
    finally:
        if args.keep:
            print(f"kept benchmark workspace: {temporary}", file=sys.stderr)
        else:
            shutil.rmtree(temporary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
