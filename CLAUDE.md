# mBot RuVector - Development Guide

## 🚨 RULE 1: No Ticket = No Code

**ALL work MUST have a GitHub issue before ANY code is written.**

```
NO TICKET → NO CODE
```

### Before Starting ANY Task

1. Check issues: `gh issue list -R Hulupeep/mbot_ruvector`
2. Find or create ticket with acceptance criteria
3. Reference issue in commits: `feat(scope): description (#123)`
4. Link PRs to the issue they close

### Issue Format (Specflow-Compliant)

Every issue MUST have these sections:

```markdown
## Description
Brief description of the work

## DOD Criticality
- [ ] **Critical** - Blocks release if failing
- [ ] **Important** - Should pass before release
- [ ] **Future** - Can release without

## Contract References
- **Feature Contracts:** [ARCH-001, ART-001]
- **Journey Contract:** [J-ART-FIRST-DRAWING]

## Acceptance Criteria (Gherkin)
Scenario: [Scenario name]
  Given [precondition]
  When [action]
  Then [expected result]

## data-testid Requirements
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Start button | `start-drawing` | Begin drawing session |

## E2E Test File
`tests/journeys/[feature].journey.spec.ts`
```

**Quick Checklist** before submitting issues:
- [ ] Has Gherkin acceptance criteria
- [ ] Lists all data-testid selectors
- [ ] References applicable contracts
- [ ] Names the E2E test file

---

## 🚨 RULE 2: Contracts Are Non-Negotiable

This project uses **Specflow contracts** (`docs/contracts/*.yml`) enforced by tests.

**The rule:** Violate a contract → build fails → PR blocked.

### Before Modifying Code

1. Check if file is protected (see table below)
2. Read the contract in `docs/contracts/`
3. Run `cargo test` or `npm test -- contracts`
4. Fix any `CONTRACT VIOLATION` errors

### Protected Files

| Files | Contract | Key Rules |
|-------|----------|-----------|
| `src/mbot-core/**` | `feature_architecture.yml` | ARCH-001: no_std compatible |
| `src/**/*.rs` | `feature_architecture.yml` | ARCH-002: Deterministic nervous system |
| `src/**/*.rs` | `feature_architecture.yml` | SAFE-001: Kitchen Table Test |
| `src/personality/**` | `feature_personality.yml` | PERS-001: Bounded parameters |

### Contract Violation Example

```
❌ CONTRACT VIOLATION: ARCH-001
File: src/mbot-core/homeostasis.rs
Pattern: Found std:: usage in no_std module
See: docs/contracts/feature_architecture.yml
```

### Override Protocol

Only humans can override. User must say:
```
override_contract: [contract_id]
```

Then Claude MUST:
1. Explain what rule is broken and why
2. Warn about consequences
3. Ask if contract should be updated

---

## 🚨 RULE 3: Tests Must Pass Before Closing

Work is NOT complete until tests pass.

### After ANY Code Changes

```bash
# 1. Rust tests
cargo test

# 2. Contract tests (pattern enforcement)
npm test -- contracts

# 3. E2E tests (user journeys)
npm run test:journeys
```

**Do NOT mark work complete if tests fail.**

---

## Active Contracts

### Feature Contracts

| ID | Contract | Description |
|----|----------|-------------|
| ARCH-001 | `feature_architecture.yml` | Core must be no_std compatible |
| ARCH-002 | `feature_architecture.yml` | Nervous system must be deterministic |
| ARCH-003 | `feature_architecture.yml` | Kitchen Table Test - no harmful behaviors |
| ARCH-004 | `feature_architecture.yml` | Personality parameters must be bounded |
| ARCH-005 | `feature_architecture.yml` | Transport layer must be abstracted |

### Journey Contracts (Definition of Done)

A feature is DONE when its journeys pass.

#### Critical (MUST pass for release)

| Journey | Description | Test File |
|---------|-------------|-----------|
| `J-ART-FIRST-DRAWING` | User creates first artwork | `tests/journeys/first-drawing.journey.spec.ts` |
| `J-PERS-MEET-PERSONALITY` | User experiences personality switch | `tests/journeys/meet-personality.journey.spec.ts` |

#### Important (SHOULD pass)

| Journey | Description | Test File |
|---------|-------------|-----------|
| `J-GAME-TICTACTOE` | User plays Tic-Tac-Toe | `tests/journeys/tictactoe.journey.spec.ts` |
| `J-LEARN-FIRST-EXPERIMENT` | Student runs first experiment | `tests/journeys/first-experiment.journey.spec.ts` |

---

## Contract Locations

| Type | Location |
|------|----------|
| Feature contracts | `docs/contracts/feature_*.yml` |
| Journey contracts | `docs/contracts/journey_*.yml` |
| Contract index | `docs/contracts/CONTRACT_INDEX.yml` |
| Contract tests | `tests/contracts/*.test.ts` |
| Journey tests | `tests/journeys/*.journey.spec.ts` |

---

## Subagent Library

Reusable agents live in `.claude/agents/*.md`. Claude Code reads and follows these instructions automatically.

### Agent Registry

| Agent | When to Use | Trigger Phrases |
|-------|-------------|-----------------|
| `specflow-writer` | New feature needs Gherkin, contracts, acceptance criteria | "write specflow for...", "spec out...", "create tickets for..." |
| `board-auditor` | Check which issues are specflow-compliant | "audit the board", "check compliance" |
| `specflow-uplifter` | Fill gaps in partially-compliant issues | "uplift issues", "fill spec gaps" |
| `contract-generator` | Generate YAML contracts from specs | "generate contracts", "create YAML contracts" |
| `contract-test-generator` | Create Jest tests from YAML contracts | "generate contract tests" |
| `contract-validator` | Verify implementation matches spec | "validate contracts" |
| `dependency-mapper` | Extract dependencies, build sprint waves | "map dependencies", "show me the waves" |
| `sprint-executor` | Execute parallel build waves | "execute sprint", "build wave 0" |
| `migration-builder` | Feature needs database schema changes | "create migrations" |
| `frontend-builder` | Create React hooks and components | "build the frontend" |
| `edge-function-builder` | Create serverless functions | "create edge functions" |
| `journey-enforcer` | Verify journey coverage, release readiness | "check journeys", "are we release ready?" |
| `journey-tester` | Create E2E journey tests | "create journey tests" |
| `playwright-from-specflow` | Generate tests from Gherkin | "generate playwright tests" |
| `ticket-closer` | Close validated issues with summaries | "close tickets", "close completed issues" |

### Auto-Trigger Rules

**Claude MUST use these agents automatically:**

1. **User asks to make issues "specflow compliant":**
   - Read `.claude/agents/specflow-writer.md`
   - Follow its process for each issue

2. **User asks to "audit" or "check compliance":**
   - Read `.claude/agents/board-auditor.md`
   - Produce compliance matrix

3. **After ANY code changes (MANDATORY):**
   - Run tests
   - Do NOT mark complete if tests fail

4. **User asks to "close tickets" or "mark done":**
   - Read `.claude/agents/ticket-closer.md`
   - Post summaries and close

### Orchestration Pipeline

```
specflow-writer → board-auditor → dependency-mapper
       ↓
sprint-executor → [implementation agents]
       ↓
contract-validator → journey-enforcer → ticket-closer
```

### Execute Backlog in Waves

Tell Claude Code:
```
Make my GitHub issues specflow-compliant, then execute my backlog in waves.
```

---

## Invariant Domains

| Domain | Prefix | Scope |
|--------|--------|-------|
| Architecture | ARCH-XXX | All code |
| Safety | SAFE-XXX | Kitchen Table Test |
| ArtBot | ART-XXX | Drawing features |
| Personality | PERS-XXX | Personality system |
| GameBot | GAME-XXX | Games |
| HelperBot | HELP-XXX | Utility features |
| LearningLab | LEARN-XXX | Education features |

---

## GitHub Repository

- **Repo:** `Hulupeep/mbot_ruvector`
- **Issues:** `gh issue list -R Hulupeep/mbot_ruvector`
- **Labels:** `epic`, `story`, `enhancement`
