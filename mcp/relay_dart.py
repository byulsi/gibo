"""Gibo additions to the DART server: save_report and usage notes."""
import json
import os
import re
import subprocess
import time

import relay

INSTRUCTIONS = """[Gibo] 사용 안내
- 사업보고서·반기보고서 등 공시 원문은 download_document 대신 save_report로 파일에 저장하고, 돌려받은 목차의 줄 번호로 필요한 장만 Read(offset/limit)로 읽는다. download_document의 마크다운 변환은 <LIBRARY>로 묶인 'II. 사업의 내용', 재무제표 주석, 미상환 전환사채 현황 등을 빠뜨리고, 기본값으로 10만 자에서 경고 없이 자른다.
- get_attachments는 쓰지 않는다. DART 웹사이트(dart.fss.or.kr)가 프로그램 접속을 막아 실패하고, 첨부는 대부분 본문의 PDF 사본이라 save_report로 같은 내용을 볼 수 있다.
- 표의 금액 단위는 회사마다 다르다(백만원, 천원 등). 표 위의 단위 표시를 함께 읽는다.
- 국내 시세·수급·PER/PBR은 KRX 도구를 기본으로 쓴다."""

TOOLS = [{
    "name": "save_report",
    "description": (
        "공시 원문(사업보고서, 반기·분기보고서, 주요사항보고서 등)을 빠짐없이 마크다운 파일로 "
        "저장하고, 파일 경로와 목차(줄 번호)만 돌려줍니다. 원문은 대화에 싣지 않습니다. "
        "공시 원문이 필요하면 download_document 대신 이 도구를 쓰고, 목차의 줄 번호로 "
        "필요한 장만 Read(offset/limit)로 읽으세요. 저장 위치: 프로젝트의 자료/사업보고서/."),
    "inputSchema": {
        "type": "object",
        "properties": {
            "rcept_no": {"type": "string", "pattern": r"^\d{14}$",
                         "description": "접수번호 14자리 (search_disclosures 결과의 rcept_no)"},
        },
        "required": ["rcept_no"],
    },
}]


def split_documents(content, files):
    """download_document(raw) joins several XML files with <!-- name --> headers."""
    parts = re.split(r"<!-- ([^>]+?\.xml) -->\n", content)
    if len(parts) == 1:
        return [{"name": files[0] if files else "document.xml", "xml": content}]
    return [{"name": parts[i], "xml": parts[i + 1]} for i in range(1, len(parts) - 1, 2)]


def tag_text(xml, tag):
    match = re.search(rf"<{tag}\b[^>]*>([^<]*)</{tag}>", xml)
    return match.group(1).strip() if match else ""


def report_name(main_xml, rcept_no):
    company = re.sub(r"주식회사|\(주\)|㈜|\s", "", tag_text(main_xml, "COMPANY-NAME")) or "회사"
    document = re.sub(r"\s", "", tag_text(main_xml, "DOCUMENT-NAME")) or "공시"
    period_end = re.search(r'AUNIT="PERIODTO"[^>]*AUNITVALUE="(\d{8})"', main_xml)
    if period_end:
        value = period_end.group(1)
        period = value[:4] if value[4:6] == "12" else value[:6]
    else:
        period = rcept_no[:8]
    return relay.safe_name(f"{company}_{period}_{document}")


def save_report(server, rcept_no):
    converter = os.environ.get("GIBO_DART_CONVERTER")
    if not converter or not os.path.exists(converter):
        raise RuntimeError("korean-dart-mcp converter not found (GIBO_DART_CONVERTER)")
    text = server.call_tool("download_document",
                            {"rcept_no": rcept_no, "format": "raw", "truncate_at": 1_000_000_000})
    payload = json.loads(text)
    docs = split_documents(payload["content"], payload.get("files") or [])
    # The main document is <rcept_no>.xml; attached audit reports follow it.
    docs.sort(key=lambda d: d["name"] != f"{rcept_no}.xml")
    converted = subprocess.run(["node", os.path.join(os.path.dirname(__file__), "dart-markdown.mjs"), converter],
                               input=json.dumps({"docs": docs}, ensure_ascii=False).encode(),
                               capture_output=True, timeout=300)
    if converted.returncode != 0:
        raise RuntimeError(converted.stderr.decode(errors="replace")[-500:])
    markdown = converted.stdout.decode()

    folder = relay.files_dir("사업보고서")
    name = report_name(docs[0]["xml"], rcept_no)
    path = os.path.join(folder, name + ".md")
    marker = f"rcept_no={rcept_no}"
    if os.path.exists(path):
        with open(path, encoding="utf-8") as existing:
            if marker not in existing.readline():
                path = os.path.join(folder, f"{name}_{rcept_no}.md")
    header = (f"<!-- Gibo save_report {marker} 원문=https://dart.fss.or.kr/dsaf001/main.do?rcpNo={rcept_no} "
              f"저장={time.strftime('%Y-%m-%d')} -->\n")
    with open(path, "w", encoding="utf-8") as out:
        out.write(header + markdown)

    body = header + markdown
    lines = body.count("\n") + 1
    return "\n".join([
        f"저장했습니다: {path}",
        f"크기: 약 {relay.estimate_tokens(body):,}토큰, {lines:,}줄 (문서 {len(docs)}개: "
        f"{', '.join(d['name'] for d in docs)})",
        "아래 목차의 줄 번호로 필요한 장만 Read(offset/limit)로 읽으세요. 표 단위는 표 위의 표시를 확인하세요.",
        "",
        "목차(줄 번호: 제목):",
        *relay.headings(body),
    ])


def handle(server, tool, arguments):
    if tool == "save_report":
        return save_report(server, str(arguments.get("rcept_no", "")))
    raise RuntimeError(f"unknown tool {tool}")
