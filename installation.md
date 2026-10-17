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

폴더 안에 두 가지가 생깁니다.

- `guru-maker/`: 기록 폴더 (wiki, lens, method, evidence, decision)
- `.guru-maker/`: 설정 폴더 (Guru Mode, 설치한 Guru Pack 원본)

이름이 `guru-maker`인 이유는 Gibo가 Guru Maker의 기록 형식과 실행 파일을 그대로
쓰기 때문입니다. 기존 Guru Maker 기록도 그대로 열 수 있습니다.

## 3. Guru Mode

| 값 | 동작 |
| --- | --- |
| `auto` (기본값) | 다음에 쓸모 있는 결과를 알아서 작게 기록 |
| `off` | "기록해줘"라고 요청할 때만 기록 |

처음에는 `off`로 시작해 어떤 내용이 어떻게 기록되는지 익힌 뒤 `auto`로 바꾸는
것을 권합니다.

```text
/gibo:memory Guru Mode를 off로 바꿔줘.
```

## 4. 실행 파일

처음 쓸 때 `gurumaker` 실행 파일을 Guru Maker GitHub 릴리스에서 받아
`~/.local/share/guru-maker/runtimes/`에 설치합니다. 원본 릴리스를 받을 수 없으면
다른 주소를 지정할 수 있습니다.

```bash
export GURUMAKER_RELEASE_BASE_URL=https://github.com/byulsi/guru-maker/releases/download
```

## Guru Maker에서 옮겨 올 때

같은 프로젝트에 Guru Maker와 Gibo를 함께 설치하면 같은 역할의 스킬이 두 벌
생깁니다. Guru Maker 플러그인을 먼저 제거하세요. 기록 폴더는 그대로 둡니다.

```bash
claude plugin uninstall guru-maker@monarchjuno
```
