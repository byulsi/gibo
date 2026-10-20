# 데이터 연결 9단계 점검

`scripts/check-connectors.py`는 Gibo의 세 데이터 연결(`openbb`, `dart`, `krx`)이
실제 투자 분석 흐름에서 제대로 답하는지 확인한다. `mcp/`의 실행 스크립트를 Claude
Code와 같은 방식으로 띄우고(중계 파일 `mcp/relay.py` 포함), 단계마다 대표 질문을
보낸다. 중계 파일이 저장하는 파일은 임시 폴더에 쓰고 끝나면 지운다.

## 언제 돌리나

- `mcp/`의 실행 스크립트, 중계 파일, `setup.sh`를 고친 뒤
- OpenBB, korean-dart-mcp, pykrx-mcp의 고정 버전을 올린 뒤
- 새 컴퓨터에 설치한 뒤
- Gibo 새 버전을 릴리스하기 전

## 실행

```bash
python3 scripts/check-connectors.py
python3 scripts/check-connectors.py --only dart,krx          # OpenBB 제외
python3 scripts/check-connectors.py --corp 대주전자재료 --ticker 078600
```

서버가 설치돼 있고 키와 계정이 설정돼 있어야 한다(`installation.md` 4절). Claude
Code 샌드박스 안에서는 플러그인 데이터 폴더에 쓸 수 없어 DART가 시작되지 않으므로
일반 터미널에서 돌린다. 날짜는 실행한 날을 기준으로 계산한다.

## 단계와 확인 내용

8단계 투자 분석 워크플로우에 거시경제 점검을 더한 9단계다.

| 단계 | 내용 | 확인하는 요청 |
|---|---|---|
| 1 리스크 | 지뢰 탐지 | DART 공시 이상 징후, 자본 이벤트 타임라인 |
| 2 성장성 | 사업·수주·가동률 | `save_report`로 저장한 사업보고서 목차에 "1. 사업의 개요"가 있는지 |
| 3 이익의 질 | 손익·현금흐름 | DART 재무제표 |
| 4 거버넌스 | 배당·내부자 | DART 배당, 임원·주요주주 매매 |
| 5 밸류에이션 | PER·PBR, 금리 | KRX 투자지표, OpenBB 한국 장기금리 |
| 6 전술 | 시세·수급·지표 | KRX 일별 시세, 투자자별 수급, `technical_indicators`의 RSI |
| 7 요약 | Decision 기록 | Gibo 실행 파일로 임시 폴더에 Decision을 쓰고 읽기 |
| 8 복기 | 판단 이후 흐름 | KRX 시세 |
| 9 거시 | 지수·수급·금리·환율 | KRX 코스피·시장 투자자별 매매, 외국인 순매수 상위(큰 응답이 파일로 저장되는지), OpenBB 미국 국채 금리·원/달러 |

## 결과 읽기

| 표시 | 뜻 |
|---|---|
| PASS | 정상 |
| WARN | 동작은 했지만 확인 필요. 응답이 Claude Code의 MCP 출력 한도(기본 25,000토큰)를 넘었거나, 기대한 내용이 빠짐 |
| FAIL | 실패하거나 빈 응답. 종료 코드 1 |

## 참고

2026년 10월 측정 결과와, 그에 따라 바꾼 내용(중계 파일, 가벼운 설치)은
`docs/connector-measurement-2026-10.md`에 있다.
