---
trigger: always_on
---

🔥 STRATOS — DEVELOPMENT CONSTITUTION

⸻

0️⃣ CORE LAW

No code is written until the system is understood, structured, and validated.

Violations = rejected.

⸻

1️⃣ DEVELOPMENT ENTRY RULE (MANDATORY)

Before writing ANY code, the system MUST produce:

✅ 1. Problem Decomposition
	•	Break into smallest units
	•	Identify unknowns
	•	Define constraints

✅ 2. Success Metrics
	•	Latency (P50/P95/P99)
	•	Throughput
	•	Memory usage
	•	Cost
	•	Correctness criteria

✅ 3. Build vs Buy Decision
	•	Existing library?
	•	SaaS?
	•	Internal reusable module?

If build → justification required.

⸻

2️⃣ DESIGN FIRST RULE

No implementation without:

⸻

📄 ADR (Required)

Title
Status
Context
Decision
Alternatives
Consequences


⸻

🧠 HLD (Required for systems)
	•	Components
	•	Data flow
	•	Failure modes
	•	Scaling model
	•	Security model

⸻

⚙️ LLD (Required for modules)
	•	Function signatures
	•	Data structures
	•	Edge cases
	•	Error handling
	•	Concurrency model

⸻

🔁 STATE MODEL

Every system must define:

INIT → RUNNING → SUCCESS | FAILED | RETRY

No undefined states.

⸻

3️⃣ CODING LAW

⸻

🔹 FUNCTION RULES
	•	≤ 40 lines
	•	≤ 4 parameters
	•	≤ 3 nesting levels
	•	Guard clauses first
	•	Pure functions preferred

⸻

🔹 FILE RULES
	•	≤ 400 lines
	•	One public class per file
	•	No circular dependencies

⸻

🔹 NAMING RULES
	•	No abbreviations
	•	Domain-driven names
	•	Consistent verbs (create, fetch, validate, execute)

⸻

🔹 DESIGN RULES

Mandatory:
	•	SOLID
	•	DRY
	•	KISS
	•	Composition > inheritance
	•	Fail-fast

⸻

4️⃣ TOOL-FIRST EXECUTION RULE

Never trust generated output without tool verification

⸻

Mandatory Tool Usage:

Domain	Tool
Code	Compiler + Test Runner
Math	CAS / Solver
Data	SQL / Pandas
Web	Search + Scraper


⸻

Rule:
	•	If tool exists → must use tool
	•	If tool not used → must justify

⸻

5️⃣ ERROR HANDLING LAW

⸻

MUST:
	•	Catch specific exceptions
	•	Add context when rethrowing
	•	Use structured error format

{
  code,
  message,
  context,
  timestamp,
  request_id
}


⸻

NEVER:
	•	Silent failures
	•	Generic catch-all
	•	Ignored errors

⸻

6️⃣ SECURITY LAW

⸻

REQUIRED:
	•	Input validation (client + server)
	•	RBAC/ABAC
	•	Secrets outside code
	•	TLS everywhere
	•	Parameterized queries

⸻

FORBIDDEN:
	•	Hardcoded secrets
	•	Direct DB string concatenation
	•	Open tool execution

⸻

7️⃣ TESTING LAW

⸻

COVERAGE:
	•	80% minimum
	•	95% critical paths

⸻

STRUCTURE:
	•	AAA pattern
	•	One assertion purpose per test

⸻

PYRAMID:
	•	Unit → 70%
	•	Integration → 20%
	•	E2E → 10%

⸻

RULE:

No test = No merge

⸻

8️⃣ GIT LAW

⸻

COMMITS:

feat(auth): add jwt refresh flow

	•	One logical change
	•	No WIP commits

⸻

PR:
	•	Must pass CI
	•	Must include test instructions
	•	Must maintain coverage

⸻

FORBIDDEN:
	•	Force push on shared branches
	•	Direct commit to main

⸻

9️⃣ MEMORY & TRACEABILITY RULE

Every action must:
	•	Be logged
	•	Be reproducible
	•	Be reversible

⸻

REQUIRED:
	•	Ledger entry
	•	Context stored
	•	Output traceable

⸻

🔟 AGENT DEVELOPMENT RULE

Every agent must define:
	•	Scope
	•	Tools allowed
	•	Memory boundary
	•	Retry policy
	•	Failure handling

⸻

RULE:

No agent operates without constraints.

⸻

1️⃣1️⃣ PERFORMANCE LAW

⸻

REQUIRED:
	•	Time complexity defined
	•	Space complexity defined
	•	Benchmark for critical paths

⸻

RULE:

If performance matters → measure, don’t assume

⸻

1️⃣2️⃣ DOCUMENTATION LAW

⸻

MUST INCLUDE:
	•	Why decision made
	•	API contracts
	•	Edge cases
	•	Failure scenarios

⸻

FORBIDDEN:
	•	Self-explanatory comments
	•	Missing public API docs

⸻

1️⃣3️⃣ DEPLOYMENT LAW

⸻

REQUIRED:
	•	CI/CD pipeline
	•	Health checks
	•	Observability (logs, metrics, traces)

⸻

RULE:

If it cannot be monitored, it cannot be trusted

⸻

1️⃣4️⃣ FINAL EXECUTION RULE

Before any system is accepted:

Checklist:
	•	ADR present
	•	HLD defined
	•	LLD complete
	•	Tests written
	•	Security validated
	•	Tool usage enforced
	•	Logs + metrics added
	•	Performance considered

⸻

🔥 FINAL PRINCIPLE

Build systems that are:

	•	Deterministic
	•	Auditable
	•	Secure
	•	Testable
	•	Replaceable
	•	Scalable

⸻

🧠 What You Just Built

This is not just rules.

This is:

A governance layer for engineering intelligence

You can enforce this:
	•	In agents
	•	In CI/CD
	•	In code reviews
	•	In orchestration kernel
