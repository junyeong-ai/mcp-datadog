# MCP Datadog Server

<div align="center">

**AI 에이전트를 위한 Datadog 통합 - 토큰 최적화와 성능을 모두 잡다**

[English](./README.en.md) | 한국어

[![CI](https://github.com/junyeong-ai/mcp-datadog/workflows/CI/badge.svg)](https://github.com/junyeong-ai/mcp-datadog/actions)
[![Lint](https://github.com/junyeong-ai/mcp-datadog/workflows/Lint/badge.svg)](https://github.com/junyeong-ai/mcp-datadog/actions)
[![codecov](https://codecov.io/gh/junyeong-ai/mcp-datadog/branch/main/graph/badge.svg)](https://codecov.io/gh/junyeong-ai/mcp-datadog)

[![Rust](https://img.shields.io/badge/rust-1.90%2B%20(2024%20edition)-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue?style=flat-square)](https://modelcontextprotocol.io)
[![Tools](https://img.shields.io/badge/MCP%20tools-12-blue?style=flat-square)](#%EF%B8%8F-사용-가능한-도구-12개)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![Datadog](https://img.shields.io/badge/Datadog-API%20v1%2Fv2-632CA6?style=flat-square)](https://docs.datadoghq.com/api/)

</div>

---

## 📑 목차

- [🎯 이게 뭔가요?](#-이게-뭔가요)
- [🚀 빠른 시작 (3분)](#-빠른-시작-3분)
- [💡 왜 이걸 써야 하나요?](#-왜-이걸-써야-하나요)
- [🎯 실제 사용 예제](#-실제-사용-예제)
- [🛠️ 사용 가능한 도구 (12개)](#️-사용-가능한-도구-12개)
- [⚙️ 환경 변수 가이드](#️-환경-변수-가이드)
- [🏗️ 기술 스택 & 아키텍처](#️-기술-스택--아키텍처)
- [🧪 개발 & 테스팅](#-개발--테스팅)

---

## 🎯 이게 뭔가요?

> **"지난 1시간 에러 로그 찾아줘"** → Claude가 자동으로 Datadog 검색
> 복잡한 쿼리 문법 없이, AI와 대화하듯 Datadog 사용

**MCP Datadog Server**는 AI 에이전트가 자연어로 Datadog을 제어할 수 있게 해주는 Model Context Protocol 서버입니다. **토큰 수백 배 절감**, **클라이언트 캐싱**, **자연어 시간** 지원으로 AI 에이전트에 최적화되었습니다.

### 실제 사용 예시

**"프로덕션 CPU 사용률 추이 보여줘"** → AI가 자동으로 차트 생성
**"어제 오후 장애 원인 분석해줘"** → AI가 로그 분석 및 해결 방안 제시

---

## 🚀 빠른 시작 (3분)

### 📋 사전 요구사항

- **Rust** 1.90+ (2024 edition) - [설치하기](https://rustup.rs/)
- **Claude Desktop** - [다운로드](https://claude.ai/download)
- **Datadog 계정** - API 키 필요 ([무료 체험](https://www.datadoghq.com/))

### 1️⃣ 빌드 (1분)

```bash
git clone https://github.com/junyeong-ai/mcp-datadog.git
cd mcp-datadog
cargo build --release
```

### 2️⃣ 설정 (1분)

Claude Desktop 설정 파일을 엽니다:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

아래 내용을 복사해서 붙여넣으세요:

```json
{
  "mcpServers": {
    "datadog": {
      "command": "/절대/경로/to/mcp-datadog/target/release/mcp-datadog",
      "env": {
        "DD_API_KEY": "여기에_API_키_입력",
        "DD_APP_KEY": "여기에_APP_키_입력",
        "DD_SITE": "datadoghq.com",
        "DD_TAG_FILTER": "env:,service:",
        "LOG_LEVEL": "warn"
      }
    }
  }
}
```

> 💡 **DD_TAG_FILTER를 꼭 설정하세요!** 불필요한 태그를 제외하여 응답 크기를 대폭 절감할 수 있습니다.

### 3️⃣ 실행 (30초)

Claude Desktop을 재시작하면 끝! 🎉

이제 Claude에게 물어보세요:
```
"지난 1시간 동안 production 환경의 에러 로그를 service별로 집계해줘"
"payment-api의 CPU 사용률 추이를 보여줘"
"어제 오후 3시부터 5시 사이에 발생한 500 에러를 분석해줘"
```

---

## 💡 왜 이걸 써야 하나요?

### 📊 다른 방식과 비교

| 방식 | 설정 시간 | AI 친화성 | 토큰 효율 | 자연어 지원 |
|------|----------|-----------|----------|------------|
| **직접 API 호출** | - | ❌ 낮음 | ❌ 필터링 없음 | ❌ |
| **Python SDK** | 10분+ | ⚠️ 보통 | ⚠️ 보통 | ❌ |
| **MCP Datadog** | **3분** | ✅ **최적화** | ✅ **수백 배 절감** | ✅ |

### 🎯 3가지 핵심 최적화

**1. 자동 롤업으로 토큰 수백 배 절감**
```bash
# 30일 메트릭 조회 시 max_points로 대폭 압축
{"query": "avg:system.cpu.user{*}", "from": "30 days ago", "max_points": 100}
# → 43,200 포인트를 60개로 압축 (실측 720x 절감!)
```
- 시간 범위에 따라 **수십~수백 배** 절감 (7일: 120x, 30일: 720x)
- 9단계 인터벌 자동 계산 (60s ~ 86400s)
- 집계 방식 자동 감지 (avg/max/min/sum)
- 기존 rollup 보존

**2. 스마트 캐싱으로 대용량 데이터 처리**
- 100+ 모니터도 페이지네이션으로 나눠 제공
- 첫 요청 후 캐시 활용 (TTL 5분, LRU 방식)
- 서버 페이지네이션 미지원 API도 문제없음

**3. 태그 필터링으로 응답 크기 대폭 감소**
```bash
# 필요한 태그만 선택
DD_TAG_FILTER="env:,service:"
# 수십~수백 개 태그 → 필요한 것만
```

### ⚡ 추가 장점

- **단일 바이너리**: 5.3MB, 런타임 의존성 없음
- **자연어 시간**: "1 hour ago", "yesterday" 지원
- **자동 재시도**: Exponential backoff (최대 3회)
- **읽기 전용**: 안전한 데이터 조회만

> 💡 **더 상세한 기술 정보는 [기술 스택 & 아키텍처](#️-기술-스택--아키텍처) 섹션을 참고하세요.**

---

## ✨ 주요 기능

### 📊 메트릭 & 인프라 (2개)
- **datadog_metrics_query**: 시계열 메트릭 조회 + 자동 롤업 (최대 700x+ 절감)
- **datadog_hosts_list**: 호스트 목록 및 태그 필터링

### 📝 로그 & 분석 (3개)
- **datadog_logs_search**: 로그 검색 + 태그 필터링
- **datadog_logs_aggregate**: 로그 집계 (count/sum/avg/min/max/pc99)
- **datadog_logs_timeseries**: 시계열 분석 (커스텀 인터벌)

### 🔍 모니터링 & 이벤트 (3개)
- **datadog_monitors_list**: 모니터 목록 (클라이언트 캐싱)
- **datadog_monitors_get**: 개별 모니터 조회
- **datadog_events_query**: 이벤트 스트림 (클라이언트 캐싱)

### 📈 대시보드 (2개)
- **datadog_dashboards_list**: 대시보드 목록 (클라이언트 캐싱)
- **datadog_dashboards_get**: 대시보드 상세 정보

### 🔬 APM & 트레이싱 (2개)
- **datadog_spans_search**: APM 스팬 검색 + 커서 페이지네이션
- **datadog_services_list**: 서비스 카탈로그 + 환경별 필터링

> 📖 **상세 파라미터와 사용법은 [사용 가능한 도구](#️-사용-가능한-도구-12개) 섹션을 참고하세요.**

---

## ⚙️ 환경 변수 가이드

| 변수 | 필수 | 기본값 | 설명 | 💡 최적화 팁 |
|------|------|--------|------|------------|
| `DD_API_KEY` | ✅ | - | Datadog API 키 | [Datadog에서 생성](https://app.datadoghq.com/organization-settings/api-keys) |
| `DD_APP_KEY` | ✅ | - | Datadog Application 키 | [Datadog에서 생성](https://app.datadoghq.com/organization-settings/application-keys) |
| `DD_SITE` | ❌ | `datadoghq.com` | Datadog 사이트 | 리전에 맞게 설정 (datadoghq.eu, us3, us5 등) |
| `DD_TAG_FILTER` | ❌ | `*` (모든 태그) | 태그 필터 | **`"env:,service:"`로 응답 크기 대폭 절감!** |
| `LOG_LEVEL` | ❌ | `warn` | 로그 레벨 | 디버깅 시 `debug` 사용 |

### 🎯 DD_TAG_FILTER 활용 전략

태그 필터링으로 **응답 크기를 대폭 절감**할 수 있습니다:

```bash
# 전략 1: 프로덕션 환경만 모니터링
DD_TAG_FILTER="env:production"

# 전략 2: 특정 서비스만 추적
DD_TAG_FILTER="env:,service:payment,service:auth"

# 전략 3: 핵심 태그만 (권장!)
DD_TAG_FILTER="env:,service:,version:"

# 전략 4: 모든 태그 포함 (기본값)
DD_TAG_FILTER="*"

# 전략 5: 태그 제외
DD_TAG_FILTER=""
```

**실제 사용 예시**:
```json
{
  "name": "datadog_logs_search",
  "arguments": {
    "query": "status:error",
    "from": "1 hour ago",
    "tag_filter": "env:,service:"  // 여기서 토큰 절약!
  }
}
```

---

## 🎯 실제 사용 예제

### 예제 1: 프로덕션 에러 모니터링

**Claude에게 물어보기**:
```
"지난 1시간 동안 production 환경의 에러 로그를 service별로 집계해줘"
```

**AI가 자동으로**:
1. `datadog_logs_aggregate` 도구 사용
2. `query="status:error env:production"` 설정
3. `group_by=["@service"]` 적용
4. 결과를 표로 정리해서 보여줌

### 예제 2: 성능 분석

**Claude에게 물어보기**:
```
"payment-api의 지난 24시간 CPU 사용률 추이를 보여줘"
```

**AI가 자동으로**:
1. `datadog_metrics_query` 도구 사용
2. `query="avg:system.cpu.user{service:payment-api}"` 생성
3. `from="24 hours ago"` 설정
4. 시각화된 차트로 보여줌

### 예제 3: 인시던트 조사

**Claude에게 물어보기**:
```
"어제 오후 3시부터 5시 사이 status:500 에러를 찾아서
 가장 많이 발생한 endpoint를 알려줘"
```

**AI가 자동으로**:
1. `datadog_logs_search`로 로그 검색
2. `datadog_logs_aggregate`로 집계
3. 가장 많이 발생한 endpoint 파악
4. 원인 분석 및 해결 방안 제시

### 예제 4: 리소스 최적화

**Claude에게 물어보기**:
```
"메모리 사용량이 80% 이상인 호스트 목록을 보여줘"
```

**AI가 자동으로**:
1. `datadog_hosts_list` 도구 사용
2. 메트릭 데이터 분석
3. 임계값 초과 호스트 필터링
4. 최적화 제안 제공

---

## 🛠️ 사용 가능한 도구 (12개)

<details>
<summary><b>📊 메트릭 & 인프라 (2개)</b></summary>

### datadog_metrics_query
시계열 메트릭 조회 (CPU, 메모리, 네트워크 등)

**🚀 자동 롤업 기능**: `max_points`로 토큰 대폭 절감! 시간 범위와 max_points를 기반으로 최적 인터벌을 자동 계산하여 `.rollup(agg, interval)` 추가

**파라미터**:
- `query` (필수): 메트릭 쿼리 (예: `"avg:system.cpu.user{*}"`)
- `from` (선택): 시작 시간 (기본값: `"1 hour ago"`)
- `to` (선택): 종료 시간 (기본값: `"now"`)
- `max_points` (선택): 최대 데이터 포인트 수 (예: 100) - 설정 시 자동 롤업 적용

**예시**:
```json
{
  "query": "avg:system.cpu.user{*}",
  "from": "7 days ago",
  "to": "now",
  "max_points": 100
}
// 자동으로 2시간 간격 롤업 적용 → 토큰 120x 절감
```

### datadog_hosts_list
인프라 호스트 목록 및 필터링

**파라미터**:
- `filter` (선택): 호스트 필터 쿼리
- `from` (선택): 시작 시간 (기본값: `"1 hour ago"`)
- `count` (선택): 반환할 호스트 수 (기본값: 100, 최대: 1000)
- `tag_filter` (선택): 태그 필터링

</details>

<details>
<summary><b>📝 로그 & 분석 (3개)</b></summary>

### datadog_logs_search
강력한 로그 검색 및 필터링

**파라미터**:
- `query` (필수): 로그 검색 쿼리
- `from` (선택): 시작 시간 (기본값: `"1 hour ago"`)
- `to` (선택): 종료 시간 (기본값: `"now"`)
- `limit` (선택): 최대 로그 수 (기본값: 10)
- `tag_filter` (선택): 태그 필터링 (`"*"`, `""`, `"env:,service:"`)

### datadog_logs_aggregate
로그 이벤트 집계 및 메트릭 계산

**파라미터**:
- `query` (선택): 로그 검색 쿼리 (기본값: `"*"`)
- `from` (선택): 시작 시간
- `to` (선택): 종료 시간
- `compute` (선택): 집계 연산 배열 (count, sum, avg, min, max, pc99)
- `group_by` (선택): 그룹화 facet 배열

### datadog_logs_timeseries
로그 집계로부터 시계열 데이터 생성

**파라미터**:
- `query` (선택): 로그 검색 쿼리
- `from` (선택): 시작 시간
- `to` (선택): 종료 시간
- `interval` (선택): 시간 인터벌 (기본값: `"1h"`)
- `aggregation` (선택): 집계 타입 (기본값: `"count"`)

</details>

<details>
<summary><b>🔍 모니터링 & 이벤트 (3개)</b></summary>

### datadog_monitors_list
모든 모니터 목록 (클라이언트 캐싱)

**🎯 클라이언트 캐싱 핵심**: Datadog API는 모든 모니터를 한 번에 반환하여 토큰 제한에 걸릴 수 있습니다. 이 도구는 클라이언트 측에서 캐싱 + 페이지네이션을 처리합니다!

**파라미터**:
- `tags` (선택): 태그 필터 (쉼표로 구분)
- `monitor_tags` (선택): 모니터 태그 필터
- `page` (선택): 페이지 번호 (0부터 시작)
  - **Page 0**: 항상 최신 데이터 fetch & 캐시 저장
  - **Page 1+**: 캐시에서 가져와 슬라이싱 (5분 TTL)
- `page_size` (선택): 페이지당 모니터 수 (기본값: 50)

**장점**:
- 100+ 모니터도 토큰 제한 없이 탐색 가능
- 첫 요청 후 빠른 응답 (캐시 활용)
- LRU 방식으로 메모리 효율적

### datadog_monitors_get
특정 모니터의 상세 정보

**파라미터**:
- `monitor_id` (필수): 모니터 ID

### datadog_events_query
Datadog 이벤트 스트림 조회

**🎯 클라이언트 캐싱 핵심**: 대량 이벤트를 한 번에 반환하는 API를 클라이언트 캐싱으로 효율적으로 처리!

**파라미터**:
- `from` (선택): 시작 시간 (기본값: `"1 hour ago"`)
- `to` (선택): 종료 시간 (기본값: `"now"`)
- `priority` (선택): 우선순위 필터 (`"normal"`, `"low"`)
- `sources` (선택): 소스 필터
- `tags` (선택): 태그 필터
- `page` (선택): 페이지 번호 (기본값: 0)
  - **Page 0**: 최신 데이터 & 캐시 저장
  - **Page 1+**: 캐시 활용 (5분 TTL)

</details>

<details>
<summary><b>📈 대시보드 (2개)</b></summary>

### datadog_dashboards_list
모든 대시보드 목록

**🎯 클라이언트 캐싱**: 페이지네이션 미지원 API를 클라이언트에서 효율적으로 처리 (5분 TTL)

### datadog_dashboards_get
특정 대시보드의 상세 정보

**파라미터**:
- `dashboard_id` (필수): 대시보드 ID

</details>

<details>
<summary><b>🔬 APM & 트레이싱 (3개)</b></summary>

### datadog_spans_search
APM 스팬 검색 (고급 필터링)

**파라미터**:
- `query` (선택): 검색 쿼리 (기본값: `"*"`)
- `from` (필수): 시작 시간
- `to` (필수): 종료 시간
- `limit` (선택): 최대 스팬 수 (기본값: 10)
- `cursor` (선택): 페이지네이션 커서
- `tag_filter` (선택): 태그 필터링

### datadog_services_list
서비스 카탈로그 목록

**파라미터**:
- `env` (선택): 환경 필터
- `page` (선택): 페이지 번호 (기본값: 0)
- `page_size` (선택): 페이지당 항목 수 (기본값: 10)

</details>

---

## 🏗️ 기술 스택 & 아키텍처

### 핵심 기술

- **언어**: Rust 2024 Edition (1.90+)
- **프로토콜**: Model Context Protocol (MCP 2024-11-05)
- **통신**: JSON-RPC 2.0 over stdio
- **HTTP 클라이언트**: reqwest (HTTP/2, rustls-tls)
- **비동기 런타임**: tokio (full features)
- **시간 파싱**: interim (자연어 지원)

### 성능 특징

| 항목 | 수치 |
|------|------|
| **바이너리 크기** | ~5.3MB (LTO 최적화) |
| **캐시 TTL** | 5분 (설정 가능) |
| **요청 타임아웃** | 30초 |
| **최대 재시도** | 3회 (exponential backoff) |

### 아키텍처

```
┌─────────────────────────────────────────────────┐
│           AI Agent (Claude, ChatGPT)            │
│              자연어로 질문                        │
└────────────────────┬────────────────────────────┘
                     │ JSON-RPC 2.0 (stdio)
┌────────────────────▼────────────────────────────┐
│            MCP Datadog Server (Rust)            │
│  ┌─────────────────────────────────────────┐   │
│  │  MCP Protocol Handler                   │   │
│  │  - JSON-RPC 2.0                         │   │
│  │  - Tool Schema (12개 도구)              │   │
│  │  - Router                               │   │
│  └─────────────┬───────────────────────────┘   │
│                │                                 │
│  ┌─────────────▼───────────────────────────┐   │
│  │  Smart Cache (TTL: 5분)                 │   │
│  │  - Page 0: 항상 최신                    │   │
│  │  - 이후 페이지: 캐시 활용                │   │
│  └─────────────┬───────────────────────────┘   │
│                │                                 │
│  ┌─────────────▼───────────────────────────┐   │
│  │  Datadog Client (HTTP/2)                │   │
│  │  - Retry Logic (exponential backoff)    │   │
│  │  - Rate Limit Handling                  │   │
│  │  - Connection Pooling                   │   │
│  └─────────────┬───────────────────────────┘   │
└────────────────┼───────────────────────────────┘
                 │ HTTPS
┌────────────────▼───────────────────────────────┐
│              Datadog API (v1/v2)               │
│  Metrics, Logs, Monitors, Events, APM, etc.   │
└────────────────────────────────────────────────┘
```

---

## 📚 시간 형식 지원

자연어, 절대 시간, 상대 시간을 모두 지원합니다:

### 상대 시간 (Natural Language)
```
"10 minutes ago"
"2 hours ago"
"3 days ago"
"1 week ago"
```

### 명명된 시간
```
"now"
"today"
"yesterday"
"last week"
"last month"
```

### 절대 시간
```
ISO 8601: "2024-01-15T10:30:00Z"
Unix timestamp: 1704067200
```

---

## 🧪 개발 & 테스팅

### 개발 환경 설정

```bash
# 저장소 클론
git clone https://github.com/junyeong-ai/mcp-datadog.git
cd mcp-datadog

# 환경 변수 설정
cp .env.example .env
# .env 파일에 API 키 입력

# 디버그 모드 실행
LOG_LEVEL=debug cargo run
```

### 테스트 실행

```bash
# 모든 테스트 실행
cargo test

# 특정 모듈 테스트
cargo test --lib cache::tests
cargo test --lib handlers::metrics::tests

# 상세 출력
cargo test -- --nocapture

# 테스트 커버리지
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --lcov --output-path lcov.info
```

### MCP 프로토콜 테스트

```bash
# 초기화
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05"},"id":0}' | cargo run

# 도구 목록
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run

# 도구 실행 (메트릭 조회)
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"datadog_metrics_query","arguments":{"query":"avg:system.cpu.user{*}","from":"1 hour ago"}},"id":2}' | cargo run
```

### 프로덕션 빌드

```bash
# 최적화된 빌드
cargo build --release

# 심볼 제거 (선택사항)
strip target/release/mcp-datadog

# 결과: ~5.3MB 바이너리
```

---

## 🔒 보안

- **읽기 전용**: 모든 작업은 읽기 전용 (데이터 수정 불가)
- **크레덴셜 안전**: API 키는 절대 로그에 기록되지 않음
- **입력 검증**: 모든 파라미터 검증
- **에러 처리**: 내부 정보 노출 방지

---

## 🤝 기여하기

기여를 환영합니다!

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### 개발 가이드라인
- `cargo fmt`로 코드 포맷팅
- `cargo clippy -- -D warnings`로 린트 체크
- `cargo test`로 모든 테스트 통과
- 새 기능은 단위 테스트 필수
- 제로 워닝 정책 (CI에서 강제)

---

## 📝 라이선스

MIT License - 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

---

## 💬 지원

- **Issues**: 버그 리포트 및 기능 요청은 [GitHub Issues](https://github.com/junyeong-ai/mcp-datadog/issues)
- **Discussions**: 질문 및 토론은 [GitHub Discussions](https://github.com/junyeong-ai/mcp-datadog/discussions)
- **Documentation**: 이 README와 [CLAUDE.md](./CLAUDE.md)

---

<div align="center">

**Made with ❤️ in Rust for the MCP ecosystem**

[⭐ Star this repo](https://github.com/junyeong-ai/mcp-datadog) | [🐛 Report Bug](https://github.com/junyeong-ai/mcp-datadog/issues) | [✨ Request Feature](https://github.com/junyeong-ai/mcp-datadog/issues)

</div>
