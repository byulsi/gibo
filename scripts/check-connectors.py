#!/usr/bin/env python3
"""Nine-step scenario check for Gibo's data connections.

Runs one or two representative requests per step of the investment workflow
(risk, growth, earnings quality, governance, valuation, tactics, decision record,
review, macro) against the installed launchers in mcp/, the same way Claude Code
starts them, including the Gibo relay (oversized responses saved to files,
save_report, technical_indicators). Files the relay saves go to a temporary
folder. Use it after changing mcp/, mcp/setup.sh, or a pinned server version.

Usage:
  python3 scripts/check-connectors.py [--only openbb,dart,krx] [--corp 삼성전자 --ticker 005930]

Exit status is 1 when any step fails. A WARN means the step worked but needs
attention, such as a response larger than Claude Code's default MCP output limit.
"""
import argparse
import datetime as dt
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.environ.get("CLAUDE_PLUGIN_DATA", os.path.expanduser("~/.claude/plugins/data/gibo-byulsi"))
FILES = tempfile.mkdtemp(prefix="gibo-check-")
OUTPUT_LIMIT_TOKENS = 25_000  # Claude Code's default MAX_MCP_OUTPUT_TOKENS


def estimate_tokens(text):
    ascii_chars = sum(1 for ch in text if ord(ch) < 128)
    return round(ascii_chars / 4 + (len(text) - ascii_chars))


class Server:
    def __init__(self, name):
        self.name = name
        env = {**os.environ, "CLAUDE_PLUGIN_DATA": DATA, "GIBO_FILES_DIR": FILES}
        self.proc = subprocess.Popen(["sh", os.path.join(ROOT, "mcp", f"{name}.sh")],
                                     stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                     stderr=subprocess.PIPE, text=True, env=env)
        self.next_id = 10
        started = time.time()
        # Same opening as Claude Code: a discovery probe, then the handshake.
        self._send({"jsonrpc": "2.0", "id": "probe", "method": "server/discover", "params": {}})
        self._send({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-11-25", "capabilities": {},
            "clientInfo": {"name": "gibo-check", "version": "1"}}})
        reply = self._wait(1)
        if not reply or "result" not in reply:
            error = self.proc.stderr.read() if self.proc.poll() is not None else ""
            raise RuntimeError(f"{name} did not start: {(error or reply)!s:.300}")
        self.startup = time.time() - started
        self._send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def _send(self, message):
        self.proc.stdin.write(json.dumps(message) + "\n")
        self.proc.stdin.flush()

    def _wait(self, want_id):
        for line in self.proc.stdout:
            try:
                data = json.loads(line)
            except ValueError:
                continue
            if data.get("id") == want_id:
                return data
        return None

    def call(self, tool, arguments):
        self.next_id += 1
        self._send({"jsonrpc": "2.0", "id": self.next_id, "method": "tools/call",
                    "params": {"name": tool, "arguments": arguments}})
        reply = self._wait(self.next_id)
        if reply is None:
            return False, "server closed the connection"
        if "error" in reply:
            return False, json.dumps(reply["error"], ensure_ascii=False)
        result = reply["result"]
        text = "\n".join(c.get("text", "") for c in result.get("content", []))
        return not result.get("isError", False), text

    def close(self):
        self.proc.kill()
        self.proc.wait()


def ymd(day, dashed=False):
    return day.strftime("%Y-%m-%d" if dashed else "%Y%m%d")


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--only", default="openbb,dart,krx")
    parser.add_argument("--corp", default="삼성전자")
    parser.add_argument("--ticker", default="005930")
    args = parser.parse_args()
    enabled = set(args.only.split(","))

    today = dt.date.today()
    end = today - dt.timedelta(days=1)
    start = end - dt.timedelta(days=45)
    year = today.year - 1 if today.month >= 4 else today.year - 2  # latest filed annual report

    corp, ticker = args.corp, args.ticker
    checks = [
        # (step, server, label, tool, arguments, expected substring or None)
        (1, "dart", "리스크: 공시 이상 징후", "disclosure_anomaly", {"corp": corp}, None),
        (1, "dart", "리스크: 자본 이벤트 타임라인", "get_corporate_event",
         {"corp": corp, "mode": "timeline", "start": ymd(today - dt.timedelta(days=365), True)}, None),
        (2, "dart", "성장성: 사업보고서 저장(사업의 개요 포함)", "annual_report", {}, "1. 사업의 개요"),
        (3, "dart", "이익의 질: 재무제표(손익·현금흐름)", "get_financials",
         {"corps": [corp], "year": year, "scope": "full", "sj_div": ["IS", "CF"]}, None),
        (4, "dart", "거버넌스: 배당", "get_periodic_report",
         {"corp": corp, "year": year, "report_type": "dividends"}, None),
        (4, "dart", "거버넌스: 임원·주요주주 매매", "insider_signal",
         {"corp": corp, "start": ymd(today - dt.timedelta(days=365), True)}, None),
        (5, "krx", "밸류에이션: PER·PBR", "get_market_fundamental_by_date",
         {"ticker": ticker, "start_date": ymd(start), "end_date": ymd(end)}, None),
        (5, "openbb", "밸류에이션: 한국 장기금리(OECD)", "call_tool",
         {"name": "oecd_country_interest_rates",
          "arguments": {"country": "korea", "duration": "long", "start_date": ymd(start, True)}}, None),
        (6, "krx", "전술: 일별 시세", "get_stock_ohlcv",
         {"ticker": ticker, "start_date": ymd(start), "end_date": ymd(end)}, None),
        (6, "krx", "전술: 투자자별 수급", "get_market_trading_value_by_date",
         {"ticker": ticker, "start_date": ymd(start), "end_date": ymd(end)}, None),
        (6, "krx", "전술: 기술적 지표", "technical_indicators", {"ticker": ticker, "end_date": ymd(end)}, "RSI_14"),
        (7, "gibo", "요약: Decision 기록 쓰기·읽기", "decision_record", {}, None),
        (8, "krx", "복기: 판단 이후 시세", "get_stock_ohlcv",
         {"ticker": ticker, "start_date": ymd(end - dt.timedelta(days=120)), "end_date": ymd(end)}, None),
        (9, "krx", "거시: 코스피 지수", "get_index_ohlcv",
         {"ticker": "1001", "start_date": ymd(start), "end_date": ymd(end)}, None),
        (9, "krx", "거시: 시장 투자자별 매매", "get_market_trading_value_by_investor",
         {"start_date": ymd(end - dt.timedelta(days=7)), "end_date": ymd(end), "ticker": "KOSPI"}, None),
        (9, "krx", "거시: 외국인 순매수 상위(큰 응답은 파일로)", "get_market_net_purchases_of_equities",
         {"start_date": ymd(end - dt.timedelta(days=7)), "end_date": ymd(end), "market": "KOSPI",
          "investor": "외국인"}, "파일로 저장"),
        (9, "openbb", "거시: 미국 국채 금리", "call_tool",
         {"name": "federal_reserve_treasury_rates", "arguments": {"start_date": ymd(start, True)}}, None),
        (9, "openbb", "거시: 원/달러 환율", "call_tool",
         {"name": "yfinance_currency_price_historical",
          "arguments": {"symbol": "USDKRW", "start_date": ymd(start, True)}}, None),
    ]

    servers, rows, failed = {}, [], False
    for step, server_name, label, tool, arguments, expected in checks:
        if server_name != "gibo" and server_name not in enabled:
            continue
        started = time.time()
        try:
            if tool == "decision_record":
                ok, text = decision_record_check()
            else:
                if server_name not in servers:
                    servers[server_name] = Server(server_name)
                server = servers[server_name]
                if tool == "annual_report":
                    ok, text = annual_report(server, corp)
                else:
                    ok, text = server.call(tool, arguments)
        except Exception as error:  # report and continue with the next step
            ok, text = False, str(error)
        seconds = time.time() - started
        tokens = estimate_tokens(text) if ok else 0
        if not ok or not text.strip():
            status = "FAIL"
            note = text.strip()[:140] or "empty response"
        elif expected and expected not in text:
            status = "WARN"
            note = f"'{expected}' missing from the response"
        elif tokens > OUTPUT_LIMIT_TOKENS:
            status = "WARN"
            note = f"larger than the {OUTPUT_LIMIT_TOKENS:,}-token MCP output limit"
        else:
            status, note = "PASS", ""
        failed |= status == "FAIL"
        rows.append((step, status, label, tokens, seconds, note))
        print(f"[{status}] {step}단계 {label}: {tokens:,} tok, {seconds:.1f}s {note}", flush=True)

    for server in servers.values():
        server.close()
    shutil.rmtree(FILES, ignore_errors=True)
    print()
    for step in range(1, 10):
        statuses = [r[1] for r in rows if r[0] == step]
        if statuses:
            worst = "FAIL" if "FAIL" in statuses else "WARN" if "WARN" in statuses else "PASS"
            print(f"  {step}단계: {worst} ({len(statuses)}건)")
    return 1 if failed else 0


def annual_report(server, corp):
    """Save the latest annual report with save_report; the reply lists its headings."""
    ok, text = server.call("search_disclosures", {"corp": corp, "kind": "periodic", "days": 365, "size": 50})
    if not ok:
        return ok, text
    items = json.loads(text).get("items", [])
    report = next((i for i in items if i["report_nm"].startswith("사업보고서")), None)
    if not report:
        return False, "no annual report filed in the last 365 days"
    return server.call("save_report", {"rcept_no": report["rcept_no"]})


def decision_record_check():
    """Initialize a throwaway workspace, write a Decision, and read it back with the Runtime."""
    runtime = os.path.join(ROOT, "shared", "gibo", "scripts", "gibo.sh")
    with tempfile.TemporaryDirectory() as workspace:
        def run(*command):
            done = subprocess.run(["sh", runtime, *command], cwd=workspace, capture_output=True, text=True)
            if done.returncode != 0:
                raise RuntimeError(done.stderr.strip() or done.stdout.strip())
            return done.stdout
        run("init", ".")
        os.makedirs(os.path.join(workspace, "gibo", "decision"), exist_ok=True)
        with open(os.path.join(workspace, "gibo", "decision", "check.md"), "w") as f:
            f.write("---\nid: decision:connector-check\ntitle: Connector check decision\n"
                    "summary: Throwaway decision written by check-connectors.py.\n"
                    f"as_of: {dt.datetime.now().astimezone().replace(microsecond=0).isoformat()}\n---\n\n"
                    "# Connector check decision\n\nHold. Falsified if the check stops passing.\n")
        run("knowledge", "check")
        return True, run("knowledge", "read", "decision:connector-check")


if __name__ == "__main__":
    sys.exit(main())
