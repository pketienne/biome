# Agent Design References

Sources identified in `~/Clones/anthropics/` for designing agents to assist the YAML support effort in Biome.

## Top-tier references

### claude-code/plugins/feature-dev/

The closest match to what we need. Contains three agent definitions with a 7-phase workflow for feature development.

- `agents/code-explorer.md` - Maps codebase, traces execution paths, identifies patterns. Directly applicable to the research/analysis agent.
- `agents/code-architect.md` - Designs implementations, analyzes patterns, creates blueprints. Directly applicable to architecture investigation.
- `agents/code-reviewer.md` - Validates code quality, checks for bugs, enforces conventions.
- Agent frontmatter format: name, description, tools, model, color.
- Tools used: Glob, Grep, LS, Read, NotebookRead, WebFetch, TodoWrite, WebSearch, KillShell, BashOutput.

### skills/skill-creator/

The meta-skill for building skills. Key design principles and patterns.

- `SKILL.md` - Comprehensive guide (keep under 500 lines, progressive disclosure).
- `references/output-patterns.md` - Template patterns for structured output, examples for flexible guidance.
- `references/workflows.md` - Sequential workflows, conditional workflows with decision points.
- Key principle: metadata -> SKILL.md -> references/ (progressive disclosure to manage context).

### anthropic-cookbook/claude_agent_sdk/

Three progressive tutorials covering research, orchestration, and external system integration.

- `00_The_one_liner_research_agent.ipynb` - Minimal research agent pattern (WebSearch + multimodal analysis).
- `01_The_chief_of_staff_agent.ipynb` - Memory management (CLAUDE.md files), output styles, plan mode, hooks, subagent orchestration. Closest to the spec/tracking agent.
- `02_The_observability_agent.ipynb` - Git/GitHub MCP integration, external system interaction.

### anthropic-cookbook/patterns/agents/

Reference implementations from Anthropic's agent design research.

- `basic_workflows.ipynb` - Prompt chaining, routing, parallelization.
- `orchestrator_workers.ipynb` - Orchestrator-workers pattern.
- `evaluator_optimizer.ipynb` - Evaluator-optimizer pattern.

## Supporting references

### claude-code/plugins/plugin-dev/

8-phase plugin development workflow. Shows how to structure a plugin bundling agents + skills + commands.

- `skills/agent-development/` - Patterns for building agents.
- `skills/command-development/` - Patterns for building commands.
- `skills/hook-development/` - Patterns for building hooks.
- `skills/mcp-integration/` - Patterns for MCP server integration.
- `skills/plugin-structure/` - Overall plugin structure guidance.
- Workflow: Discovery -> Component Planning -> Detailed Design -> Structure Creation -> Implementation -> Validation -> Testing -> Documentation.

### claude-code/plugins/code-review/

Multi-agent pattern running 5 parallel Sonnet agents with confidence-based scoring. Relevant for running multiple analysis agents concurrently and filtering false positives.

### anthropic-cookbook/.claude/skills/cookbook-audit/

4-dimension scoring rubric (narrative, code, accuracy, actionability). Could inform how the tracking agent evaluates progress and completeness.

- `SKILL.md` - Audit skill definition.
- `style_guide.md` - Quality rubric.

### skills/mcp-builder/

4-phase process (research, implementation, testing, documentation). Clean workflow template.

### anthropic-cookbook/.claude/agents/code-reviewer.md

Detailed code review checklist pattern. Specific review areas (structure, code quality, pedagogy, security). Could inform implementation guidance agent.

### courses/prompt_engineering_interactive_tutorial/

9-chapter progression on prompt engineering. Informs how to structure agent prompts and spec writing patterns.

### courses/prompt_evaluations/

Evaluation frameworks and scoring rubrics. Code-graded evals, model-graded evals, custom graders. Could inform progress tracking methodology.

## Suggested agent structure

Based on these sources, the agents for the YAML support effort should be organized as:

```
agents/
  research-analyst.md           # based on code-explorer pattern
  architecture-investigator.md  # based on code-architect pattern
  spec-writer.md                # based on chief-of-staff patterns
  progress-tracker.md           # based on chief-of-staff + cookbook-audit rubric
skills/
  yaml-feature-comparison/      # structured analysis output
  biome-integration/            # biome-specific reference knowledge
commands/
  research.md                   # orchestrates research workflow
  track-progress.md             # orchestrates tracking workflow
```

## Key design patterns to apply

1. **Agent definition format** (from feature-dev agents): YAML frontmatter with name, description, tools list, model, color. Detailed mission statement. Clear approach/process steps. Output guidance with examples.

2. **Skill design** (from skill-creator): Concise SKILL.md under 500 lines. Progressive disclosure via references/ directory. Clear trigger phrases in description. Separate references/ files for detailed information.

3. **Agent coordination** (from chief-of-staff, orchestrator-workers): Central coordinator agent managing specialist agents. Each agent has specific focus area. Information flows between agents via structured outputs. Progress tracking via TodoWrite.

4. **Structured output** (from output-patterns.md): Templates for consistent output format. Examples showing desired style and detail level. Markdown formatting for readability. File/line references for code analysis.

## Quick reference: key file locations

| Purpose                 | Primary source                                            | Backup source                                  |
|-------------------------|-----------------------------------------------------------|------------------------------------------------|
| Agent definitions       | `claude-code/plugins/feature-dev/agents/code-*.md`        | `anthropic-cookbook/claude_agent_sdk/`          |
| Agent coordination      | `anthropic-cookbook/patterns/agents/`                      | `claude-code/plugins/plugin-dev/`              |
| Skill structure         | `skills/skill-creator/SKILL.md`                           | `skills/*/SKILL.md`                            |
| Output patterns         | `skills/skill-creator/references/output-patterns.md`      | `anthropic-cookbook/.claude/`                   |
| Workflow design         | `skills/skill-creator/references/workflows.md`            | `claude-code/plugins/feature-dev/README.md`    |
| Progress tracking       | `anthropic-cookbook/claude_agent_sdk/01_*.ipynb`           | `courses/prompt_evaluations/`                  |
| Spec writing            | `skills/doc-coauthoring/SKILL.md`                         | `anthropic-cookbook/.claude/skills/`            |
| Hook patterns           | `claude-code/plugins/hookify/`                            | `claude-code/plugins/plugin-dev/skills/hook-*` |
| MCP integration         | `skills/mcp-builder/SKILL.md`                             | `claude-code/plugins/plugin-dev/skills/mcp-*`  |
| Teaching patterns       | `courses/prompt_engineering_interactive_tutorial/`         | `anthropic-cookbook/.claude/skills/cookbook-*`   |

All paths are relative to `~/Clones/anthropics/`.
