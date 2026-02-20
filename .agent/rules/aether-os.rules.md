# AETHER-OS — Agent Rules

> Project-specific rules layered on top of STRATOS master rules (`.agent/rules/claude.rules.md`).

---

## 1. Project Identity

- **Name**: AETHER-OS
- **Type**: SaaS multi-tenant AI agent operating system
- **Languages**: Rust (core) + Python (agent runtime) + TypeScript (frontend)
- **PRD**: [`docs/AETHER_OS_PRD.md`](../../docs/AETHER_OS_PRD.md) (44 sections)

---

## 2. Reference Implementation Mapping

**CRITICAL**: Before implementing any module, check if PicoClaw or OpenClaw has already
implemented it. If yes, **study their architecture first** and adapt patterns for AETHER OS.

Reference projects are in `ref/` (gitignored — local study only).

### PicoClaw (Go) → Adapt to Rust/Python

| AETHER OS Module | PicoClaw Source | What to Adapt |
|---|---|---|
| **Agent Loop** (§3.2) | `pkg/agent/loop.go` | Iteration pattern: LLM call → tool exec → feed back |
| **Context Builder** | `pkg/agent/context.go` | System prompt assembly: identity + skills + memory + tools |
| **Session Manager** (§24) | `pkg/session/manager.go` | Session keys, history, summarization, truncation |
| **Tool Registry** (§5) | `pkg/tools/registry.go` | Register, retrieve, execute with context/async |
| **Tool Interface** | `pkg/tools/base.go` | `Tool`, `ContextualTool`, `AsyncTool` interfaces |
| **Tool Types** | `pkg/tools/types.go` | `Message`, `ToolCall`, `LLMResponse`, `ToolDefinition` |
| **Memory System** | `pkg/agent/memory.go` | `MemoryStore`, `GetMemoryContext` pattern |
| **Agent Instance** | `pkg/agent/instance.go` | Agent config: ID, model, workspace, tools, session |
| **Channel System** (§23) | `pkg/channels/` | 12 adapters: Discord, Telegram, Slack, WhatsApp, etc. |
| **Message Bus** (§26) | `pkg/bus/bus.go` | Message routing between agents |
| **Session Keys** (§24) | `pkg/routing/session_key.go` | Tenant:agent:channel:peer key format |
| **Cron** (§27) | `cmd/picoclaw/cmd_cron.go` | Cron job management |
| **Agent Registry** | `pkg/agent/registry.go` | Multi-agent registration and lookup |
| **Config** | `pkg/config/config.go` | Config structure, defaults, migration |
| **Auth** | `pkg/auth/` | OAuth, PKCE, token store |

### OpenClaw (TypeScript) → Adapt to Python

| AETHER OS Module | OpenClaw Source | What to Adapt |
|---|---|---|
| **System Prompt** | `src/agents/system-prompt.ts` | Prompt modes (full/minimal/none), section construction |
| **Tool System** (§5) | `src/agents/pi-tools.ts` | Tool creation factory, policy pipeline |
| **Plugin SDK** (§25) | `src/plugin-sdk/` | Lifecycle hooks, command handlers, tool injection |
| **Cron Service** (§27) | `src/cron/` | Isolated agent execution, scheduling |
| **Browser Tools** (§36) | `src/browser/` | Browser routes, automation |
| **Sandbox** (§5.3) | `src/agents/sandbox/` | Sandbox context, security policies |
| **Auto-Reply** (§37) | `src/auto-reply/` | Autonomous agent mode |
| **Gateway** | `src/gateway-cli/` | Gateway pairing, multi-device |
| **Discord** (§23) | `src/discord/` | Discord channel adapter + monitoring |
| **Channel System** | `src/channels/` | Channel plugins, Telegram, web |
| **Agent Skills** | `src/agents/skills/` | Skill loading and configuration |
| **Sessions** (§24) | `src/config/sessions/` | Session persistence and config |
| **CLI** | `src/cli/` | CLI tool patterns |

### Implementation Priority Rule

```
BEFORE writing any module:
1. Check the mapping table above
2. If reference exists → read the source file FIRST
3. Understand their design decisions
4. Adapt for AETHER OS (Rust performance, multi-tenancy, type safety)
5. Document what you adapted in the file header comment
```

### Key Architecture Adaptations

| Pattern | PicoClaw/OpenClaw | AETHER OS Adaptation |
|---|---|---|
| **Language** | Go / TypeScript | Rust (core) + Python (runtime) |
| **Tenant** | Single-tenant | Multi-tenant (tenant_id everywhere) |
| **Memory** | File-based / SQLite | Mem0 (vector + graph + multi-modal) |
| **Transport** | Direct channel coupling | Unified `Transport` trait |
| **Tool safety** | Minimal validation | 3-tier + policy engine + sandbox |
| **Audit** | Logs only | Cryptographic ledger with replay |
| **Scheduling** | Sequential | DAG + priority queue + backpressure |
| **Config** | YAML/JSON files | Config service + hot-reload |

---

## 3. Architecture Rules

### Language Boundaries
- **Rust** (`crates/`): All 16 crate modules — core, orchestrator, tool-gateway, ledger, memory, policy, workflow, model-router, context-graph, budget, transport, session, scheduler, compliance, diagnostics, server
- **Python** (`aether_runtime/`): Agent loop, LLM providers, tool execution, cognitive, debate, learning, world model, multimodal, copilot, economics, tool factory, persistent agents, knowledge, plugins, bus, cron, streaming
- **TypeScript** (`frontend/`): Next.js web application
- **gRPC** (`proto/`): Inter-service communication Rust ↔ Python
- **Never** mix languages within a single crate/package

### Multi-Tenancy
- Every data structure must carry `tenant_id`
- Every query must be tenant-scoped (no cross-tenant access)
- Tests must verify tenant isolation

### Workspace Layout
```
AETHER-OS/
├── crates/                        # Rust workspace (16 crates)
│   ├── aether-core/               # Shared types, traits, errors
│   ├── aether-orchestrator/       # Intent parsing, DAG, scheduling
│   ├── aether-tool-gateway/       # Tool registry, validation, sandbox
│   ├── aether-ledger/             # Append-only audit ledger
│   ├── aether-memory/             # Tiered memory management
│   ├── aether-policy/             # Centralized RBAC/ABAC policy engine
│   ├── aether-workflow/           # Persistent workflow engine + saga
│   ├── aether-model-router/       # LLM selection by complexity/cost
│   ├── aether-context-graph/      # AST + call graph + deps + embeddings
│   ├── aether-budget/             # Cost governance + kill switch
│   ├── aether-transport/          # Multi-channel transport (Discord, etc.)
│   ├── aether-session/            # Conversation session management
│   ├── aether-scheduler/          # Global priority scheduling
│   ├── aether-compliance/         # GDPR, PII, data governance
│   ├── aether-diagnostics/        # Self-diagnostics, health scoring
│   └── aether-server/             # Binary — modular monolith entry point
├── aether_runtime/                # Python agent runtime (31 modules)
│   ├── agent/                     # Loop, context, session, planner, spawn
│   ├── providers/                 # LLM abstraction (Anthropic, OpenAI, etc.)
│   ├── tools/                     # Tool definitions & execution
│   ├── memory/                    # Mem0 wrapper
│   ├── plugins/                   # Plugin SDK (OpenClaw pattern)
│   ├── bus/                       # Agent-to-agent messaging
│   ├── cognitive/                 # Beliefs, hypotheses, confidence
│   ├── debate/                    # Multi-agent negotiation
│   ├── learning/                  # Self-improvement loop
│   ├── world_model/               # Simulation & counterfactual
│   ├── multimodal/                # Modality routing
│   ├── copilot/                   # Human-AI co-creation
│   ├── economics/                 # Internal pricing
│   ├── tool_factory/              # Autonomous tool creation
│   ├── persistent/                # Always-on daemon agents
│   ├── knowledge/                 # Knowledge synthesis
│   └── server.py                  # FastAPI server
├── frontend/                      # Next.js web application
├── proto/                         # gRPC service definitions
├── docs/                          # Architecture docs, ADRs, PRD
├── tests/                         # Integration & e2e tests
├── deploy/                        # Docker, K8s, Terraform
├── scripts/                       # Dev scripts, CI helpers
└── ref/                           # Reference projects (gitignored)
```

---

## 4. Rust Crate Rules

- Follow `rust-standards` skill strictly
- All public types derive `Debug, Clone, Serialize, Deserialize`
- Use `thiserror` for library errors, `anyhow` only in binaries
- Use `async-trait` for async trait definitions
- All IDs are `uuid::Uuid` wrapped in newtypes (see `aether-core/src/ids.rs`)
- Max function length: 40 lines
- Max file length: 400 lines
- No `unwrap()` in production code
- All crates re-export public API from `lib.rs`

### Naming
```
aether-core     → aether_core
aether-ledger   → aether_ledger
ToolGateway     → ToolGateway (struct)
execute_tool    → execute_tool (fn)
MAX_RETRIES     → MAX_RETRIES (const)
```

---

## 5. Python Rules

- Follow `python-standards` skill strictly
- Python 3.12+, strict mypy, ruff linting
- Google-style docstrings on all public APIs
- `Protocol` for interfaces (dependency inversion)
- `pydantic` for all data models and validation
- `structlog` for structured JSON logging
- All async code must handle `CancelledError`
- Max function length: 40 lines
- Max file length: 400 lines

---

## 6. Tool Development Rules

- Every tool must implement the universal schema (see PRD §5.2)
- Tools classified as FUNDAMENTAL / COMPOUND / PERMISSIVE
- All tool inputs/outputs validated via JSON Schema
- All tool executions logged to AETHER-LEDGER
- Permissive tools require workflow contract approval
- Tools must be idempotent or declare otherwise
- Auto-created tools (§39) must pass sandbox validation before registration

---

## 7. Testing Rules

- Follow `testing-standards` skill
- Coverage: 80% min, 95% for critical paths
- Tenant isolation verified in every integration test
- Ledger integrity checked in every e2e test
- Use `#[tokio::test]` for async Rust tests
- Use `pytest-asyncio` for async Python tests

---

## 8. Git Rules

- Follow `git-workflow` skill
- Conventional commits: `feat|fix|docs|refactor|test|ci|chore`
- Scope by crate/package: `feat(aether-core): add TenantId type`
- One logical change per commit
- PR title = conventional commit format

---

## 9. Documentation Rules

- Follow `documentation-standards` skill
- ADRs in `docs/architecture/decisions/`
- API docs auto-generated (FastAPI OpenAPI + Rust doc comments)
- Every public Rust type has `///` doc comments
- Every public Python function has Google-style docstrings
- Architecture diagrams in Mermaid format
- Every adapted module header must cite the reference source

---

## 10. Memory Rules (Mem0)

- Use Mem0 as the memory backend (§22)
- Tenant isolation via `user_id=f"tenant:{tenant_id}"`
- Agent memories via `agent_id`
- Session memories via `run_id`
- Graph memory enabled by default (Neo4j)
- Never build custom memory infra — extend Mem0 instead
