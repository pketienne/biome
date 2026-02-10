use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_yaml_syntax::{YamlRoot, YamlSyntaxKind, YamlSyntaxNode};
use rustc_hash::{FxHashMap, FxHashSet};

declare_lint_rule! {
    /// Disallow circular anchor/alias references in YAML documents.
    ///
    /// Circular alias chains (where anchor A references anchor B which
    /// references back to anchor A) cause infinite loops in YAML processors
    /// and should be avoided.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// a: &x
    ///   b: *y
    /// c: &y
    ///   d: *x
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// defaults: &defaults
    ///   timeout: 30
    /// production:
    ///   <<: *defaults
    ///   host: prod.example.com
    /// ```
    pub NoCircularAliases {
        version: "next",
        name: "noCircularAliases",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct CircularAliasState {
    alias_name: String,
    alias_range: TextRange,
    anchor_name: String,
    anchor_range: TextRange,
}

impl Rule for NoCircularAliases {
    type Query = Ast<YamlRoot>;
    type State = CircularAliasState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let syntax = root.syntax();

        // Phase 1: Collect all anchors with their name, range, and value scope
        let mut anchor_ranges: FxHashMap<String, TextRange> = FxHashMap::default();
        let mut anchor_scopes: FxHashMap<String, YamlSyntaxNode> = FxHashMap::default();

        for node in syntax.descendants() {
            if node.kind() == YamlSyntaxKind::YAML_ANCHOR_PROPERTY {
                if let Some(token) = node
                    .children_with_tokens()
                    .filter_map(|c| c.into_token())
                    .find(|t| t.kind() == YamlSyntaxKind::ANCHOR_PROPERTY_LITERAL)
                {
                    let name = token
                        .text_trimmed()
                        .strip_prefix('&')
                        .unwrap_or(token.text_trimmed())
                        .to_string();
                    let range = token.text_trimmed_range();
                    let value_scope = node.parent().and_then(|p| p.parent());

                    anchor_ranges.insert(name.clone(), range);
                    if let Some(scope) = value_scope {
                        anchor_scopes.insert(name, scope);
                    }
                }
            }
        }

        // Phase 2: Build dependency graph: anchor_name → aliases in its subtree
        let mut deps: FxHashMap<String, Vec<(String, TextRange)>> = FxHashMap::default();

        for (anchor_name, scope) in &anchor_scopes {
            let mut alias_deps = Vec::new();
            for desc in scope.descendants() {
                if desc.kind() == YamlSyntaxKind::YAML_ALIAS_NODE {
                    if let Some(token) = desc
                        .children_with_tokens()
                        .filter_map(|c| c.into_token())
                        .find(|t| t.kind() == YamlSyntaxKind::ALIAS_LITERAL)
                    {
                        let alias_name = token
                            .text_trimmed()
                            .strip_prefix('*')
                            .unwrap_or(token.text_trimmed())
                            .to_string();
                        let alias_range = token.text_trimmed_range();
                        alias_deps.push((alias_name, alias_range));
                    }
                }
            }
            if !alias_deps.is_empty() {
                deps.insert(anchor_name.clone(), alias_deps);
            }
        }

        // Phase 3: DFS cycle detection
        let mut results = Vec::new();
        let mut global_visited: FxHashSet<String> = FxHashSet::default();
        let anchor_names: Vec<String> = anchor_ranges.keys().cloned().collect();

        for start_anchor in &anchor_names {
            if global_visited.contains(start_anchor) {
                continue;
            }
            detect_cycles(
                start_anchor,
                &deps,
                &anchor_ranges,
                &mut global_visited,
                &mut results,
            );
        }

        results.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.alias_range,
                markup! {
                    "The alias "<Emphasis>{"*"}{&state.alias_name}</Emphasis>" creates a circular reference."
                },
            )
            .detail(
                state.anchor_range,
                markup! {
                    "This anchor "<Emphasis>{"&"}{&state.anchor_name}</Emphasis>" is part of the cycle."
                },
            )
            .note(markup! {
                "Circular alias chains cause infinite loops in YAML processors."
            }),
        )
    }
}

fn detect_cycles(
    start: &str,
    deps: &FxHashMap<String, Vec<(String, TextRange)>>,
    anchor_ranges: &FxHashMap<String, TextRange>,
    global_visited: &mut FxHashSet<String>,
    results: &mut Vec<CircularAliasState>,
) {
    // Iterative DFS with explicit stack
    let mut stack: Vec<(String, usize)> = vec![(start.to_string(), 0)];
    let mut path: Vec<String> = vec![start.to_string()];
    let mut path_set: FxHashSet<String> = FxHashSet::default();
    path_set.insert(start.to_string());

    loop {
        let depth = stack.len();
        if depth == 0 {
            break;
        }

        let (ref current_name, ref mut dep_idx) = stack[depth - 1];
        let current_name = current_name.clone();

        let Some(alias_deps) = deps.get(&current_name) else {
            // No outgoing edges — backtrack
            stack.pop();
            if let Some(done) = path.pop() {
                path_set.remove(&done);
                global_visited.insert(done);
            }
            continue;
        };

        if *dep_idx >= alias_deps.len() {
            // All deps explored — backtrack
            stack.pop();
            if let Some(done) = path.pop() {
                path_set.remove(&done);
                global_visited.insert(done);
            }
            continue;
        }

        let (ref alias_name, alias_range) = alias_deps[*dep_idx];
        let alias_name = alias_name.clone();
        let alias_range = alias_range;
        *dep_idx += 1;

        if path_set.contains(&alias_name) {
            // Cycle found
            if let Some(anchor_range) = anchor_ranges.get(&alias_name) {
                results.push(CircularAliasState {
                    alias_name: alias_name.clone(),
                    alias_range,
                    anchor_name: alias_name,
                    anchor_range: *anchor_range,
                });
            }
        } else if anchor_ranges.contains_key(&alias_name)
            && !global_visited.contains(&alias_name)
        {
            path_set.insert(alias_name.clone());
            path.push(alias_name.clone());
            stack.push((alias_name, 0));
        }
    }
}
