# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records for workspace-level tooling and configuration decisions that affect multiple projects in ~/Projects.

## What is an ADR?

An ADR captures:

- **Context** — Why this decision is needed
- **Options considered** — Alternatives evaluated
- **Decision** — What we chose and why
- **Consequences** — Good, neutral, and bad outcomes

## Index

| ADR                                       | Title                             | Status   | Date       |
| ----------------------------------------- | --------------------------------- | -------- | ---------- |
| [ADR-001](ADR-001-trunk-configuration.md) | Trunk Configuration Consolidation | Accepted | 2026-01-25 |
| [ADR-002](ADR-002-markdown-formatting.md) | Markdown Formatting Strategy      | Proposed | 2026-01-25 |

## ADR Template

```markdown
# ADR-NNN: Decision Title

**Status:** Proposed | Accepted | Deprecated | Superseded by ADR-XXX
**Date:** YYYY-MM-DD
**Deciders:** who participated
**Affects:** which projects/components

## Context

Why this decision is needed.

## Options Considered

### Option 1: Name

Description, pros, cons.

### Option 2: Name

Description, pros, cons.

## Decision

What we chose and why.

## Consequences

### Positive

- Good outcome

### Neutral

- Neither good nor bad

### Negative

- Trade-off accepted
```

## Status Definitions

- **Proposed** — Under discussion, not yet accepted
- **Accepted** — Decision made and in effect
- **Deprecated** — No longer recommended but not replaced
- **Superseded** — Replaced by another ADR (reference it)
