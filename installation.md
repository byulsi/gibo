# Gibo 설치

Gibo는 Claude Code 플러그인입니다.

## 1. 플러그인 설치

Claude Code에서:

```text
/plugin marketplace add byulsi/gibo
/plugin install gibo@byulsi
/reload-plugins
```

터미널에서 이 프로젝트 폴더에만 설치하려면:

```bash
claude plugin marketplace add byulsi/gibo
claude plugin install gibo@byulsi --scope project
```

## 2. 기록을 쌓을 프로젝트 초기화

기록을 쌓을 폴더에서 새 세션을 열고:

```text
/gibo:memory 이 프로젝트를 초기화해줘.
```

폴더 안에 다음이 생깁니다.

| 경로 | 내용 |
| --- | --- |
| `gibo/wiki/` | Wiki 기록 |
| `gibo/lens/` | Lens 기록 |
| `gibo/method/` | Method 기록 |
| `gibo/evidence/` | Evidence 기록 |
| `gibo/decision/` | Decision 기록 |
| `.gibo/mode` | Record Mode 설정 (`auto` 또는 `off`) |
| `.gibo/packs/` | 설치한 Gibo Pack 원본 |

기록 종류마다 폴더는 하나입니다. 기록이 많아지면 그 안에 주제별 하위 폴더를
만들어 정리할 수 있습니다(nested subject folders).

## 3. Record Mode

Record Mode는 요청 없이도 기록을 남길지 정하는 스위치입니다.

| 값 | 동작 |
| --- | --- |
| `auto` (기본값) | 다음에 쓸모 있는 결과를 알아서 작게 기록 |
| `off` | "기록해줘"처럼 explicit 요청이 있을 때만 기록 |

처음에는 `off`로 시작해 어떤 내용이 어떻게 기록되는지 익힌 뒤 `auto`로 바꾸는
것을 권합니다. 스킬에 요청하거나 터미널에서 `gibo mode off`로 바꿉니다.

```text
/gibo:memory Record Mode를 off로 바꿔줘.
```

## 4. 데이터 연결

플러그인을 켜면 `openbb`, `dart`, `krx` 세 연결이 Claude Code에 등록됩니다.
연결마다 서버 프로그램 설치와, 필요하면 키나 계정 입력이 한 번씩 필요합니다.

### 서버 프로그램 설치 (한 번)

[uv](https://docs.astral.sh/uv/)(OpenBB, KRX)와 Node.js 20.19 이상(DART)이
필요합니다. 버전은 고정되어 있습니다.

```bash
sh ~/.claude/plugins/cache/byulsi/gibo/*/mcp/setup.sh
```

설치는 플러그인 데이터 폴더(`~/.claude/plugins/data/gibo-byulsi/`)에 됩니다.
실제로 쓰는 부품만 설치해 약 0.75GB입니다(OpenBB의 개발 도구·차트·기술적 분석
확장, DART 첨부 파일 변환용 선택 부품 제외). 다시 실행하면 OpenBB와 DART는 기존 설치를
지우고 새로 설치합니다.
플러그인을 업데이트해도 유지되고, 플러그인을 제거하면 함께 지워집니다. 필요한
것만 설치하려면 뒤에 `openbb`, `dart`, `krx` 중 원하는 이름을 붙입니다.

### 키와 계정

| 연결 | 값 | 받는 곳 |
| --- | --- | --- |
| `dart` | OpenDART 인증키 (40자) | [opendart.fss.or.kr](https://opendart.fss.or.kr/) 인증키 신청 |
| `krx` | KRX 아이디와 비밀번호 | [data.krx.co.kr](https://data.krx.co.kr/contents/MMC/COMS/client/MMCCOMS002.cmd)에서 **아이디/비밀번호 방식**으로 가입. 카카오·네이버 간편 가입 계정은 쓸 수 없음 |

플러그인을 켤 때 Claude Code가 이 값을 물어봅니다. 나중에 `/plugin`에서 Gibo를
골라 바꿀 수도 있습니다. 키와 비밀번호는 운영체제의 보안 저장소에 저장되고
설정 파일에는 적히지 않습니다.

macOS에서는 키체인에 직접 넣어 두는 방법도 됩니다. 플러그인 설정이 비어 있으면
키체인에서 찾습니다.

```bash
security add-generic-password -a "$USER" -s opendart-api-key -w
security add-generic-password -a 여기에_KRX_아이디 -s krx-data-login -w
```

### 확인

```bash
claude mcp list
```

세 연결이 `Connected`로 나오면 됩니다. 실패하면 `/mcp`에서 해당 연결을 골라
오류 메시지를 보세요. 설치나 키가 빠졌으면 무엇을 해야 하는지 알려 줍니다.
쓰지 않는 연결은 `/mcp`에서 끌 수 있습니다.

### 알아 둘 점

- 플러그인을 **제거하면** 데이터 폴더도 함께 지워져 서버를 다시 설치해야 합니다.
  새 버전으로 바꿀 때는 제거하지 말고 `claude plugin update gibo@byulsi`를 쓰거나,
  제거할 때 `--keep-data`를 붙입니다.
- OpenBB는 시작에 약 20초와 메모리 1GB 이상이 듭니다. 그래서 OpenBB 도구를 처음
  부를 때 시작합니다(아래 중계 파일 참고). 그 첫 요청만 20초쯤 늦습니다.
- pykrx는 KRX에 로그인합니다. 같은 계정으로 다른 곳에 로그인돼 있으면 그 접속이
  끊길 수 있습니다.
- 공매도 잔고는 2거래일 늦게 공개됩니다.

### 중계 파일 (`mcp/relay.py`)

세 연결은 모두 Gibo의 중계 파일을 거쳐 실행됩니다. 중계 파일이 하는 일은 다음과
같습니다.

- **큰 응답을 파일로:** 응답이 약 2만 토큰을 넘으면 대화에 싣지 않고 프로젝트의
  `자료/응답/`에 저장한 뒤, 구조 요약과 파일 경로만 돌려줍니다. Claude는 파일에서
  필요한 부분만 읽습니다. 기준은 `GIBO_SPILL_TOKENS`로 바꿀 수 있고, 7일이 지난
  파일은 다음 저장 때 지웁니다.
- **사업보고서 저장 (`save_report`, dart):** 공시 원문을 빠짐없이 마크다운으로
  바꿔 `자료/사업보고서/`에 저장하고 목차와 줄 번호만 돌려줍니다. korean-dart-mcp의
  `download_document`는 `<LIBRARY>`로 묶인 "II. 사업의 내용", 재무제표 주석 등을
  빠뜨리고 기본 10만 자에서 잘라서, 그 대신 씁니다.
- **기술적 지표 (`technical_indicators`, krx):** KRX 일별 시세로 이동평균, RSI,
  MACD, ATR, 볼린저 밴드, 변동성, 기간 수익률, 52주 고저, 최대 낙폭을 계산해
  최신값만 돌려줍니다. 시세 원본은 대화를 거치지 않습니다.
- **사용 안내:** 각 연결의 안내문에 Gibo의 사용 안내(국내 데이터는 KRX와 DART 우선,
  `get_attachments`는 쓰지 않음 등)를 덧붙입니다.
- **OpenBB 늦게 시작:** 첫 시작 때 저장해 둔 도구 목록으로 응답하다가, OpenBB
  도구를 처음 부를 때 서버를 시작합니다. 국내 분석만 하는 세션에서는 OpenBB가
  메모리를 쓰지 않습니다.
- **연결 첫 단계 충돌 회피:** Claude Code 2.1.274가 보내는 새 규격 확인 요청
  (`server/discover`)에 직접 답합니다. OpenBB의 MCP 라이브러리(fastmcp 4)가 이 요청
  뒤에 오는 일반 연결 요청을 거절하기 때문입니다.

`자료/` 폴더 위치는 `GIBO_FILES_DIR`로 바꿀 수 있습니다. 공개 저장소로 관리하는
프로젝트라면 `자료/응답/`은 Git에서 빼 두는 것이 좋습니다.

## 5. Gibo Pack

Wiki, Lens, Method 기록 중 하나를 GitHub 저장소로 묶어 다른 사람과 나눌 수
있습니다. 묶음 맨 위에는 `gibo-pack.json` 설명 파일이 있습니다. Guru Maker로
만든 묶음(`guru-pack.json`)도 설치할 수 있습니다. 관리는 `/gibo:packs` 스킬이나
`gibo pack` 명령으로 합니다. 자세한 규칙은
[Gibo Packs](docs/community-plugins.md)를 보세요.

## 6. 실행 파일

처음 쓸 때 `gibo` 실행 파일을 Gibo GitHub 릴리스에서 받아
`~/.local/share/gibo/runtimes/`에 설치합니다. 다른 주소에서 받으려면:

```bash
export GIBO_RELEASE_BASE_URL=https://example.com/gibo/releases/download
```

## Guru Maker에서 옮겨 올 때

1. 같은 프로젝트에 Guru Maker와 Gibo를 함께 두면 같은 역할의 스킬이 두 벌
   생깁니다. Guru Maker 플러그인을 먼저 제거하세요.

   ```bash
   claude plugin uninstall guru-maker@monarchjuno
   ```

2. 기록 폴더 이름을 바꿉니다. 기록 파일 형식은 같습니다.

   ```bash
   mv guru-maker gibo
   mv .guru-maker .gibo
   ```
