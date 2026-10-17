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

## 4. Gibo Pack

Wiki, Lens, Method 기록 중 하나를 GitHub 저장소로 묶어 다른 사람과 나눌 수
있습니다. 묶음 맨 위에는 `gibo-pack.json` 설명 파일이 있습니다. Guru Maker로
만든 묶음(`guru-pack.json`)도 설치할 수 있습니다. 관리는 `/gibo:packs` 스킬이나
`gibo pack` 명령으로 합니다. 자세한 규칙은
[Gibo Packs](docs/community-plugins.md)를 보세요.

## 5. 실행 파일

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
