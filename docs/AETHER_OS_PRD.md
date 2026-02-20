# AETHER-Ω — Cognitive Cloud OS

> **Version**: 2.0 · **Status**: Active · **Last Updated**: 2026-02-20

An AI-native cloud OS that provisions compute, runs intelligent agents, executes tools in sandboxed VMs, and manages intelligence as infrastructure. Built in **Rust** (kernel) + **Python** (runtime) + **TypeScript** (frontend).

---

## Part 0 — Foundation

### §01 Executive Summary

AETHER-Ω is a **4-layer cognitive cloud platform**: host → hypervisor → kernel → agent runtime. It combines compute isolation (VMs), intelligent orchestration (DAGs), and adaptive agents (LLM-powered) into a single modular monolith.

**Design Principles**:
- **SOLID** — each module owns one bounded context
- **DRY** — shared types in `aether-core`; no duplication across modules
- **KISS** — simple agent loop; complexity in tooling, not orchestration
- **Fail-fast** — validate at boundaries; structured errors with codes
- **Tenant isolation** — data, compute, permissions scoped per tenant
- **Observability-first** — traces, metrics, structured logs from day one

### §02 System Architecture

```
AETHER-Ω COGNITIVE CLOUD OS

┌─────────────────────────────────────────────────────────┐
│                 INTERFACE (§37–§38)                     │
│  CLI · Web Dashboard · Agent Console · Workflow Builder │
├─────────────────────────────────────────────────────────┤
│         LAYER 3: AGENT RUNTIME — Python (§22–§36)      │
│  AgentLoop · Cognitive · Debate · Learning · WorldModel │
│  Teams · Persistent · CoPilot · ToolFactory · Plugins   │
├─────────────────────────────────────────────────────────┤
│         LAYER 2: AETHER KERNEL — Rust (§08–§21)        │
│  Orchestrator · Scheduler · Policy · Budget · Ledger    │
│  Workflow · ModelRouter · Memory(Mem0) · Session · Bus  │
│  Transport · Compliance · Diagnostics · Economics       │
├─────────────────────────────────────────────────────────┤
│         LAYER 1: HYPERVISOR — Rust (§05–§07)           │
│  MicroVM · DevVM · AgentVM · SecureVM · Compiler · Net  │
├─────────────────────────────────────────────────────────┤
│         LAYER 0: HOST (§04)                            │
│  Linux/Mac · KVM/Firecracker · Physical Hardware        │
└─────────────────────────────────────────────────────────┘
```

#### Execution Flow

```
1. Tenant → API Gateway (auth + RBAC)
2. Gateway → Policy Engine (evaluate permissions)
3. Policy → Orchestration Kernel (parse intent, build DAG)
4. Kernel → Scheduler (priority queue, tenant fairness)
5. Scheduler → Agent Runtime (spawn agent in VM)
6. Agent Loop:
   a. Context Builder: system prompt + Mem0 memory + skills
   b. LLM Provider call (routed by Model Router)
   c. Tool calls → Policy → Tool Gateway → Sandbox VM → Ledger
   d. Result → agent iterates or spawns subagent
   e. No tool calls → Verification → commit → return
7. Output → Transport Layer → user channel (Discord/Slack/Web/API)
```

#### State Machine

```
PENDING → SCHEDULED → EXECUTING → VERIFYING → COMMITTED
                          ↓
                       FAILED → RETRYING → ESCALATED → HUMAN_REVIEW
                                                ↓
                                           COMPENSATING → ROLLED_BACK
```

### §03 Multi-Tenancy

| Layer | Isolation |
|---|---|
| **API** | JWT with `tenant_id` claim; all requests scoped |
| **Compute** | Per-tenant VM pools; resource quotas |
| **Data** | PostgreSQL RLS; Redis key-prefix; Qdrant collection-per-tenant |
| **Memory** | Mem0 `user_id=tenant:{id}` scoping |
| **Budget** | Per-tenant cost tracking on every call |

#### RBAC

```
Tenant: Owner → Admin → Developer → Viewer
Agent:  Boss(T1) → Specialist(T2) → Worker(T3) → Sensor(T4)
```

---

## Part I — Layer 0: Host

### §04 Host Requirements

| Component | Requirement |
|---|---|
| **OS** | Linux (Arch/NixOS recommended) or macOS (dev) |
| **Hypervisor** | KVM + Firecracker (production) / Docker (dev) |
| **CPU** | 8+ cores recommended |
| **RAM** | 32GB+ recommended |
| **Storage** | SSD, 500GB+ |
| **Network** | Host NIC, virtual bridge for VM networking |

---

## Part II — Layer 1: Hypervisor

### §05 VM System (`aether-hypervisor`)

Files: `manager.rs`, `microvm.rs`, `devvm.rs`, `image.rs`, `allocator.rs`

| Type | Purpose | Resources | Lifetime |
|---|---|---|---|
| **MicroVM** | Single tool execution | 1 vCPU, 256MB | Seconds |
| **DevVM** | Coding, build, test | 2-4 vCPU, 2-8GB | Minutes-hours |
| **AgentVM** | Long-running agents | 2 vCPU, 2GB | Hours-days |
| **SecureVM** | Sensitive ops | 1 vCPU, 1GB, **no network** | Seconds |

#### AETHER-LINUX Base Image

| Category | Tools |
|---|---|
| System | bash, zsh, python3, node, gcc, rustc |
| Dev | git, docker, build-essential |
| AI | llama.cpp, embeddings engine |
| Security | nmap, wireshark (RESTRICTED only) |
| Media | ffmpeg, imagemagick |
| Data | jq, sqlite3, pandas |
| Web | curl, httpie, playwright |

### §06 Virtual Network (`aether-network`)

Files: `mesh.rs`, `routing.rs`, `firewall.rs`, `dns.rs`

| Feature | Description |
|---|---|
| Service mesh | Internal VM-to-VM communication |
| Zero trust | Every call authenticated |
| Rate limiting | Per-tenant, per-agent, per-tool |
| Firewall | Per-VM-type network policies |
| DNS | `agent.tenant.aether.local` discovery |

| VM Type | Internet | Mesh | Host |
|---|---|---|---|
| MicroVM | Restricted | Yes | No |
| DevVM | Filtered | Yes | Limited |
| AgentVM | Yes | Yes | No |
| SecureVM | **No** | **No** | **No** |

### §07 Compiler & Execution (`aether-compiler`)

Files: `compile.rs`, `execute.rs`, `languages.rs`, `sandbox.rs`

```
Code → Parse → AST → Compile → Run in MicroVM → Test → Validate → Return
```

| Language | Runtime | Use |
|---|---|---|
| Python | CPython 3.12+ | Scripts, tools |
| JS/TS | Node 20+ / Bun | Web tools, plugins |
| Rust | rustc stable | Performance tools |
| C/C++ | gcc/clang | System tools |
| Shell | bash/zsh | Automation |

Each compilation sandboxed: resource limits, seccomp, ephemeral filesystem.

---

## Part III — Layer 2: Kernel (Rust)

### §08 Orchestration Kernel (`aether-orchestrator`)

| Property | Spec |
|---|---|
| Responsibility | Intent parsing, DAG construction, agent scheduling |
| APIs | `POST /intents`, `GET /tasks/{id}`, `POST /tasks/{id}/cancel` |
| Storage | Redis (state), PostgreSQL (metadata), Kafka (events) |
| SLA | P99 intent-to-first-action < 800ms |

#### Meta-Orchestrator

System-level controller ("CEO agent") sits above the kernel:
- Global resource allocation across tenants
- Workflow path optimization
- Conflict resolution
- Consumes diagnostics (§21) + learning (§28) data

### §09 Global Scheduler (`aether-scheduler`)

Files: `priority.rs`, `queue.rs`, `fairness.rs`, `backpressure.rs`

| Priority | SLA | Use |
|---|---|---|
| P0 | < 1s | Production incidents |
| P1 | < 10s | High-priority workflows |
| P2 | < 60s | Normal throughput |
| P3 | Best-effort | Background / batch |

- Weighted round-robin per tenant
- Queue starvation prevention
- Load shedding under pressure
- Auto-escalation after TTL

### §10 Tool System (`aether-tool-gateway`)

Files: `registry.rs`, `validation.rs`, `execution.rs`, `sandbox.rs`, `health.rs`

#### 4-Tier Access Model

| Level | Who | Examples | Requires |
|---|---|---|---|
| **PUBLIC** | All agents | web_search, read_file | Nothing |
| **PROTECTED** | Tier ≥ 2 | code_compiler, test_runner | Tier check |
| **RESTRICTED** | Approved only | nmap, deploy | Policy + sandbox + approval |
| **CRITICAL** | Human only | prod_deploy, delete_data | Human approval + audit |

#### Universal Tool Schema

```json
{
  "tool_id": "UUID", "name": "snake_case", "version": "semver",
  "access": "PUBLIC|PROTECTED|RESTRICTED|CRITICAL",
  "input_schema": {}, "output_schema": {},
  "idempotent": true, "reversible": false,
  "timeout_ms": 30000, "sandbox_required": true,
  "retry_policy": { "max_attempts": 3, "backoff_ms": [100, 500, 2000] },
  "resource_limits": { "max_memory_mb": 256 }
}
```

#### Enforcement

```
Agent → Policy Engine → access check →
  PUBLIC:     execute immediately
  PROTECTED:  verify tier → execute in VM
  RESTRICTED: verify approval → execute in SecureVM
  CRITICAL:   queue for human → execute after approval
```

#### Sandbox (gVisor)

Network denied (allowlist), ephemeral tmpfs, max 4 cores / 8GB / 300s wall time.

### §11 Policy Engine (`aether-policy`)

Files: `engine.rs`, `rules.rs`, `evaluation.rs`

```
POST /policy/evaluate
{ "subject", "action", "resource", "context" }
→ { "decision": "ALLOW|DENY", "reason", "constraints" }
```

Policy types: RBAC (role), ABAC (attributes), temporal (time-window), budget-gated (cost threshold).

### §12 Budget Engine (`aether-budget`)

Files: `tracker.rs`, `limiter.rs`, `alerts.rs`

| Trigger | Action |
|---|---|
| 75% consumed | Alert tenant admin |
| 90% consumed | Auto-downgrade model |
| 100% consumed | Kill all active tasks |
| Per-task limit | Kill task if exceeded |

#### Economic System

Internal tool pricing, cost-value optimization, resource auctions under contention, ROI tracking per tenant. (Python: `aether_runtime/economics/`)

### §13 Workflow Engine (`aether-workflow`)

Files: `definition.rs`, `executor.rs`, `nodes.rs`, `store.rs`

Node types: `Agent`, `Tool`, `Condition`, `Loop`, `HumanApproval`, `Retry`, `Compensation`.

```
POST /workflows           — Create
POST /workflows/{id}/run  — Execute
POST /workflows/{id}/resume — Resume from checkpoint
```

#### Saga Pattern (§30 merged)

```
Step 1: migrate → Compensation: rollback
Step 2: deploy  → Compensation: revert
Step 3: DNS     → Compensation: restore
Failure at Step 3 → compensations run in reverse
```

### §14 AETHER-LEDGER (`aether-ledger`)

Files: `block.rs`, `chain.rs`, `storage.rs`, `verify.rs`

Append-only cryptographic audit. Ed25519 signatures, SHA-256 hash chaining, Merkle roots.

| Capability | API |
|---|---|
| Append | `POST /ledger/entries` |
| Verify | `POST /ledger/verify` (chain integrity) |
| Replay | `POST /ledger/replay` (reconstruct execution) |
| Simulate | `POST /ledger/simulate` (hypothetical, no side effects) |
| Debug | `POST /ledger/debug` (step-by-step with breakpoints) |
| Diff | `GET /ledger/diff/{a}/{b}` (compare traces) |

SLA: write P99 < 50ms, zero data loss.

### §15 Memory System — Mem0 (`aether-memory`)

Files: `client.rs`, `tiers.rs`, `search.rs`

Powered by **Mem0** (mem0ai). Replaces custom memory with production-proven layer: +26% accuracy, 91% faster, 90% fewer tokens.

| Tier | Mem0 Scope | TTL | Use |
|---|---|---|---|
| Working | `run_id=session` | Session | Current conversation |
| Project | `agent_id=agent` | Permanent | Agent knowledge |
| Knowledge | `user_id=tenant:X` | Permanent | Tenant patterns |
| Graph | Neo4j graph store | Permanent | Entity relationships |

Config: Qdrant (vectors) + Neo4j (graph) + OpenAI embeddings.

```
POST /v1/memory/add      — Store (auto-extracts facts)
POST /v1/memory/search   — Semantic search (tenant-scoped)
GET /v1/memory/{id}      — Get specific memory
DELETE /v1/memory/{id}   — Delete memory
```

### §16 Session Management (`aether-session`)

Files: `manager.rs`, `history.rs`, `summary.rs`, `keys.rs`

Adapted from PicoClaw's `SessionManager`.

```rust
impl SessionManager {
    fn get_or_create(&self, key: SessionKey) -> Session;
    fn add_message(&self, key: &SessionKey, msg: Message);
    fn get_history(&self, key: &SessionKey) -> Vec<Message>;
    fn truncate(&self, key: &SessionKey, keep_last: usize);
    async fn summarize(&self, key: &SessionKey) -> String;
    async fn save(&self, key: &SessionKey) -> Result<()>;
}
```

Key format: `tenant:{id}:agent:{id}:channel:{type}:peer:{id}`

Sessions auto-persist to Mem0: short-term via `run_id`, facts via `agent_id`, knowledge via `user_id`.

### §17 Transport Layer (`aether-transport`)

Files: `channel.rs`, `discord.rs`, `telegram.rs`, `slack.rs`, `websocket.rs`, `webhook.rs`

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    fn channel_type(&self) -> ChannelType;
    async fn listen(&self) -> Result<Box<dyn Stream<Item = InboundMessage>>>;
    async fn send(&self, msg: OutboundMessage) -> Result<()>;
    async fn send_streaming(&self, stream: ResponseStream) -> Result<()>;
}
```

Channels: Discord, Telegram, Slack, WhatsApp, WebSocket, Webhook, REST.

Routing: inbound → resolve (channel, tenant, peer) → create/resume session → agent loop → respond via same channel.

### §18 Context Graph (`aether-context-graph`)

Files: `ast.rs`, `call_graph.rs`, `dependency.rs`, `embeddings.rs`, `summary.rs`

| Layer | Tech | Provides |
|---|---|---|
| AST | Tree-sitter | Syntax, symbols |
| Call Graph | Static analysis | Function relationships |
| Dependencies | Manifest parsing | Module/crate deps |
| Embeddings | Qdrant | Similarity search |
| Summaries | LLM | Hierarchical summaries |

### §19 Model Router (`aether-model-router`)

Files: `router.rs`, `strategy.rs`, `complexity.rs`

Routes by task complexity, budget, latency, domain:

| Signal | Low | Medium | High |
|---|---|---|---|
| Complexity | Haiku | Sonnet | Opus |
| Budget tight | Small always | Small→mid | Deny |
| Latency critical | Cached/small | Mid+stream | Async queue |

### §20 Compliance (`aether-compliance`)

Files: `retention.rs`, `pii.rs`, `gdpr.rs`, `export.rs`

Retention policies, PII detection/masking, GDPR delete (`DELETE /v1/compliance/gdpr/{user_id}`), audit export, data classification (PUBLIC/INTERNAL/CONFIDENTIAL/RESTRICTED).

### §21 Self-Diagnostics (`aether-diagnostics`)

Files: `health.rs`, `prediction.rs`, `bottleneck.rs`

Health scoring (0–100 per module), failure prediction (anomaly detection), bottleneck detection (critical path), capacity planning, auto-remediation.

---

## Part IV — Layer 3: Agent Runtime (Python)

### §22 Agent System (`aether_runtime/agent/`)

Files: `loop.py`, `context.py`, `session.py`, `planner.py`, `spawn.py`

| Tier | Role | Tools | Limit |
|---|---|---|---|
| T1 Boss | Strategy, delegation | All (via delegation) | 1 per tenant |
| T2 Specialist | Domain work | PUBLIC + PROTECTED | Configurable |
| T3 Worker | Execution | PUBLIC only | Cannot spawn |
| T4 Sensor | Read-only | PUBLIC read-only | No side effects |

#### Agent Loop (adapted from PicoClaw `loop.go`)

```
Context Build → LLM Call → Tool/Spawn/Output →
  Tool: Policy → Gateway → Sandbox → Ledger → feed back
  Spawn: create child agent (max depth 3) → wait → feed back
  Output: Verification → commit → return
```

#### Agent Identity (`aether_runtime/agent/identity/`)

```yaml
agent:
  name: "CodeReviewer"
  version: "1.2.0"
  tier: 2
  identity: { role: "Senior code reviewer", personality: "Thorough, direct" }
  rules: ["Run tests before approving", "Flag security as CRITICAL"]
  tools_allowed: [code_compiler, test_runner, security_scanner]
  memory: { auto_recall: true, recall_limit: 20 }
```

### §23 LLM Providers (`aether_runtime/providers/`)

Files: `base.py`, `anthropic.py`, `openai.py`, `google.py`, `ollama.py`, `factory.py`

```python
class LLMProvider(Protocol):
    async def chat(self, messages, tools, **kwargs) -> LLMResponse: ...
    async def chat_stream(self, messages, tools, **kwargs) -> AsyncIterator[StreamChunk]: ...
```

Stream chunks: `text_delta`, `tool_call_start`, `tool_call_delta`, `tool_result`, `thinking`, `done`.

Delivery: WebSocket (bidirectional), SSE (simpler), transport-specific (Discord edits, Telegram chunks, Slack blocks).

### §24 Cognitive State (`aether_runtime/cognitive/`)

Files: `state.py`, `beliefs.py`, `hypotheses.py`, `confidence.py`

| State | Tracks | Storage |
|---|---|---|
| Beliefs | What agent thinks is true | Mem0 + Redis |
| Hypotheses | What agent is testing | Working memory |
| Confidence Graph | Probability over time | PostgreSQL |
| Reasoning Trace | Structured chain | Ledger |

### §25 Debate Engine (`aether_runtime/debate/`)

Files: `engine.py`, `protocol.py`, `evaluator.py`

```
Propose → Critique → Verify → Synthesize best answer
```

Protocols: adversarial, cooperative, consensus.

### §26 World Model (`aether_runtime/world_model/`)

Files: `simulator.py`, `counterfactual.py`, `predictor.py`

Outcome simulation, counterfactual analysis ("what if strategy B?"), risk prediction, system modeling (traffic/load/failure).

### §27 Knowledge Synthesis (`aether_runtime/knowledge/`)

Files: `synthesizer.py`, `graph.py`

Combine sources → novel insights. Build knowledge graphs via Mem0 graph store. Generate documentation from analysis.

### §28 Learning Engine (`aether_runtime/learning/`)

Files: `tracker.py`, `optimizer.py`, `metrics.py`

Tracks: tool success rates, agent performance, model effectiveness, user corrections, workflow outcomes. Feeds into Model Router (§19) and prompt optimization.

### §29 Auto Teams (`aether_runtime/teams/`)

Files: `generator.py`, `roles.py`, `protocol.py`

Spawn when `task_complexity > 0.7`:

```
Boss(T1) → Planner(T2) + Researcher(T2) + Engineer(T2) + Tester(T2) + Reviewer(T2)
```

Communication via Agent Bus. Boss delegates via DAG.

### §30 Tool Factory (`aether_runtime/tool_factory/`)

Files: `generator.py`, `composer.py`

Problem → no tool → LLM generates → validate schema → sandbox test → register → use. All generated tools sandboxed + ledger-logged + human-reviewable.

### §31 Human Interaction (`aether_runtime/human_review/`, `aether_runtime/copilot/`)

**Review** (`human_review/service.py`, `types.py`):
```
Output → confidence < threshold? → queue for human
  Approve → commit | Reject → retry with feedback | Edit → commit + learn
```

**Co-Pilot** (`copilot/session.py`, `intervention.py`):
Editable execution, intervention points, suggestion mode, human takeover, feedback injection to Mem0.

### §32 Persistent Agents (`aether_runtime/persistent/`)

Files: `daemon.py`, `monitor.py`

Always-on agents. Triggers: events (git.push, ci.failure), intervals (cron), continuous.

### §33 Plugin SDK (`aether_runtime/plugins/`)

Files: `sdk.py`, `loader.py`, `lifecycle.py`

```python
class AetherPlugin(Protocol):
    def metadata(self) -> PluginMetadata: ...
    def on_load(self, api: AetherPluginAPI) -> None: ...
    def before_agent_start(self, ctx: AgentContext) -> AgentContext: ...
    def after_response(self, resp: AgentOutput) -> AgentOutput: ...
    def register_tools(self) -> list[ToolDefinition]: ...
    def on_cron(self, schedule: str) -> CronHandler | None: ...
    def on_unload(self) -> None: ...
```

Hot-reload via file watcher. Distribution: `plugins/` directory + `plugin.yaml` manifest.

### §34 Agent Bus (`aether_runtime/bus/`)

Files: `broker.py`, `types.py`

Tenant-scoped `publish`, `subscribe`, `broadcast`. Backend: Redis Pub/Sub (real-time) + Kafka (persistent). All messages ledger-logged.

### §35 Multimodal Pipeline (`aether_runtime/multimodal/`)

Files: `router.py`, `pipeline.py`

```
Input → classify → TEXT|CODE|IMAGE|AUDIO|VIDEO|PDF → route to model → unify output
```

### §36 Cron & Scheduling (`aether_runtime/cron/`)

Files: `scheduler.py`, `jobs.py`

Standard 5-field cron. Per-execution agent isolation. Timeout enforcement. Failure notification via transport.

---

## Part V — Interface

### §37 CLI (`aether-cli`)

Files: `main.rs`, `commands.rs`, `output.rs`

```bash
aether run "build a REST API"        # natural language task
aether vm create --type devvm        # VM management
aether agent list | inspect | logs   # agent operations
aether workflow run | list           # workflow management
aether task status | cancel | replay # task management
aether memory search "patterns"      # memory queries
aether status | health | cost        # system info
aether deploy <service> --env prod   # deployment
```

Output: `--format json|table`, `--follow`, `--verbose`.

### §38 Frontend (Next.js)

| View | Purpose |
|---|---|
| Workspace | Tenant home — agents, workflows, tasks |
| Agent Console | Chat + live execution |
| Workflow Builder | Visual DAG editor |
| Ledger Explorer | Audit trail + replay |
| Memory Explorer | Mem0 search + knowledge graph |
| Cost Dashboard | Budget tracking + projections |
| Plugin Manager | Install/configure |
| Settings | Tenant config, API keys, RBAC |

Stack: Next.js 14+, Shadcn/ui, Zustand, React Query, WebSocket, Recharts.

---

## Part VI — Operations

### §39 Security

| Domain | Implementation |
|---|---|
| Auth | JWT (15min access / 7d refresh) + API keys (hash-before-store) |
| Encryption | TLS 1.3 (transit), AES-256-GCM (rest) |
| Secrets | HashiCorp Vault; never in code |
| Sandbox | All code in gVisor; network denied; ephemeral FS |
| Zero trust | All VM-to-VM calls authenticated |

### §40 Observability

| Signal | Tech |
|---|---|
| Metrics | Prometheus + Grafana |
| Logs | Structured JSON (`tenant_id`, `trace_id`, `agent_id`) |
| Traces | OpenTelemetry + Jaeger |
| Alerts | Alertmanager (SLO-based, PagerDuty P0/P1) |
| Cost | Per-tenant token + compute tracking |

### §41 Versioning

All entities versioned (semver). API: URL prefix (`/v1/`). Proto: package path (`aether.v1`). Compatibility: major = breaking (6mo support), minor = additive, patch = bugfix.

### §42 Retry Framework

Exponential backoff + jitter, idempotency keys (Redis, 24h TTL), circuit breaker (5 failures → 60s cooldown → half-open with 3 calls).

### §43 Technology Stack

| Layer | Tech |
|---|---|
| Kernel | Rust (axum, tonic, tokio) |
| Runtime | Python 3.12+ (asyncio, pydantic, structlog) |
| Frontend | Next.js 14+ (TypeScript) |
| Primary DB | PostgreSQL 16 |
| Cache | Redis Cluster 7.x |
| Vector DB | Qdrant |
| Graph DB | Neo4j |
| Memory | Mem0 (mem0ai) |
| Events | Kafka |
| Objects | S3 / MinIO |
| Monitoring | Prometheus + Grafana + Jaeger |
| Container | Docker + Kubernetes |

---

## Module Registry

### Rust Crates (20)

| Crate | Layer | SRP |
|---|---|---|
| `aether-core` | Shared | Types, IDs, errors, traits |
| `aether-hypervisor` | L1 | VM lifecycle (§05) |
| `aether-network` | L1 | Virtual networking (§06) |
| `aether-compiler` | L1 | Code compilation + execution (§07) |
| `aether-orchestrator` | L2 | Intent → DAG → schedule (§08) |
| `aether-scheduler` | L2 | Priority queue + fairness (§09) |
| `aether-tool-gateway` | L2 | Tool registry + sandbox (§10) |
| `aether-policy` | L2 | RBAC/ABAC evaluation (§11) |
| `aether-budget` | L2 | Cost governance (§12) |
| `aether-workflow` | L2 | Persistent DAGs + saga (§13) |
| `aether-ledger` | L2 | Cryptographic audit (§14) |
| `aether-memory` | L2 | Mem0 integration (§15) |
| `aether-session` | L2 | Conversation sessions (§16) |
| `aether-transport` | L2 | Channel adapters (§17) |
| `aether-context-graph` | L2 | Code intelligence (§18) |
| `aether-model-router` | L2 | LLM selection (§19) |
| `aether-compliance` | L2 | Data governance (§20) |
| `aether-diagnostics` | L2 | Self-diagnostics (§21) |
| `aether-cli` | Interface | CLI binary (§37) |
| `aether-server` | Binary | Modular monolith entry point |

### Python Modules (33 directories)

| Module | Layer | SRP |
|---|---|---|
| `agent/` | L3 | Loop, context, session, planner, spawn, identity (§22) |
| `providers/` | L3 | LLM abstraction + streaming (§23) |
| `cognitive/` | L3 | Beliefs, hypotheses, confidence (§24) |
| `debate/` | L3 | Multi-agent negotiation (§25) |
| `world_model/` | L3 | Simulation, counterfactual (§26) |
| `knowledge/` | L3 | Synthesis, graph (§27) |
| `learning/` | L3 | Self-improvement (§28) |
| `teams/` | L3 | Auto-team generation (§29) |
| `tool_factory/` | L3 | Auto tool creation (§30) |
| `human_review/` | L3 | Review queue (§31) |
| `copilot/` | L3 | Human-AI co-creation (§31) |
| `persistent/` | L3 | Daemon agents (§32) |
| `plugins/` | L3 | Plugin SDK + loader (§33) |
| `bus/` | L3 | Agent messaging (§34) |
| `multimodal/` | L3 | Modality routing (§35) |
| `cron/` | L3 | Scheduling (§36) |
| `tools/` | L3 | Tool base + registry + builtins |
| `skills/` | L3 | Skill loader + schema |
| `memory/` | L3 | Mem0 client wrapper |
| `streaming/` | L3 | Stream handler + types |
| `model_router/` | L3 | Python routing client |
| `context_graph/` | L3 | Graph builder + resolver |
| `policy/` | L3 | Policy client |
| `workflow/` | L3 | Workflow client + nodes |
| `retry/` | L3 | Retry policy + circuit breaker |
| `economics/` | L3 | Pricing + allocation |
| `vm/` | L3 | VM client |
| `config/` | Shared | Configuration |
| `errors/` | Shared | Error hierarchy |
| `server.py` | Entry | FastAPI server |