# Gibo (기보)

**판단을 한 수씩 기록하고, 나중에 복기하는 투자 메모리.**

바둑의 기보(棋譜)는 한 판의 모든 수를 적은 기록입니다. 기보가 있어야 복기를
할 수 있고, 복기를 해야 실력이 늡니다. Gibo는 AI 에이전트와 함께 한 투자
리서치와 판단을 프로젝트 안의 Markdown 기록으로 남기고, 다음 작업에서 다시
꺼내 쓰게 합니다.

Gibo는 [Guru Maker](https://github.com/monarchjuno/guru-maker)를 바탕으로
Claude Code용으로 고친 프로젝트입니다. Guru Maker 프로젝트와는 관계가 없으며
공식 버전이 아닙니다.

## Guru Maker와 다른 점

| | Guru Maker | Gibo |
| --- | --- | --- |
| 지원 환경 | Codex, Claude Code | Claude Code 전용 |
| 스킬 호출 | `/guru-maker:guru-research` | `/gibo:research` |
| 데이터 연결 | 없음 (호스트에 맡김) | 공신력 있는 출처(공시, 거래소, 거시 데이터) 연결 예정 |
| 기록 폴더 | `guru-maker/` | `gibo/`. 기록 파일 형식은 같음 |
| 실행 파일 | `gurumaker` | `gibo` |
| 기록 스위치 | Guru Mode | Record Mode |
| 기록 공유 묶음 | Guru Pack | Gibo Pack. Guru Pack도 설치 가능 |

## 기록 다섯 가지

| 종류 | 담는 것 |
| --- | --- |
| Wiki | 잘 바뀌지 않는 구조 지식. 예: 반도체 밸류체인 |
| Lens | 검증된 해석 틀. 적용 범위와 반증 조건 포함 |
| Method | 반복해서 쓰는 분석 절차 |
| Evidence | 출처와 시점이 있는 관찰 |
| Decision | 특정 시점의 판단과 근거, 위험, 무효화 조건 |

Decision은 나중에 고쳐 쓰지 않습니다. 회고는 새 기록으로 남겨서, 그때 무엇을
알고 무엇을 몰랐는지 그대로 복기할 수 있게 합니다.

## 스킬

| 스킬 | 쓰임 |
| --- | --- |
| `/gibo:research` | 최신 자료와 프로젝트 기록으로 리서치 |
| `/gibo:memory` | 초기화, 기록 찾기·저장·정리, Record Mode 변경 |
| `/gibo:decision` | 투자 판단 기록과 결과 회고 |
| `/gibo:train` | 회고에서 교훈을 뽑아 반례로 검증 |
| `/gibo:packs` | GitHub로 공유하는 기록 묶음(Gibo Pack) 관리 |

초기화된 프로젝트에서는 스킬 이름을 부르지 않고 평소처럼 질문해도 알맞은
스킬이 쓰입니다.

## 설치

[설치 안내](installation.md)를 보세요.

## 문서

- [설치 안내](installation.md)
- [원본 Guru Maker README](docs/guru-maker-readme.md)
- [기록 모델과 검색](docs/knowledge-model.md)
- [판단 기록과 그래프](docs/decision-memory.md)
- [구조와 제품 범위](docs/architecture.md)

상세 문서는 영어로 되어 있습니다.

## 면책

Gibo는 리서치와 판단 기록을 정리하는 소프트웨어입니다. 투자, 금융, 법률, 세무
조언을 제공하지 않고, 결과를 보장하지 않으며, 매매를 실행하지 않습니다. 출력은
불완전하거나 틀릴 수 있으니 중요한 사실과 출처는 직접 확인하세요. 투자 판단과
그 결과에 대한 책임은 사용자에게 있습니다.

## 라이선스

Apache-2.0. 출처와 저작권 고지는 [NOTICE](NOTICE)를 보세요.
