use biome_analyze::{Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::TextRange;
use biome_rule_options::no_undefined_subject_reference::NoUndefinedSubjectReferenceOptions;
use biome_turtle_syntax::TurtleRoot;
use std::collections::HashSet;

use crate::services::semantic::Semantic;

declare_lint_rule! {
    /// Detect references to subjects that are never defined in the current document.
    ///
    /// In RDF, it is common to reference external resources. However, when working
    /// with self-contained Turtle documents, an object that uses a locally-declared
    /// prefix but is never defined as a subject may indicate a typo or missing data.
    ///
    /// This rule only checks prefixed-name objects (e.g., `ex:bob`) — full IRIs,
    /// literals, and blank nodes are always skipped. Common vocabulary prefixes
    /// (rdf:, rdfs:, owl:, xsd:, foaf:, dc:, etc.) are allowed by default.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:knows ex:bob .
    /// ex:alice ex:knows ex:carol .
    /// ex:bob ex:name "Bob" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// ex:alice foaf:name "Alice" .
    /// ex:alice ex:knows ex:bob .
    /// ex:bob foaf:name "Bob" .
    /// ```
    ///
    /// ## Options
    ///
    /// Use the `allowedPrefixes` option to whitelist additional external vocabulary
    /// prefixes beyond the built-in defaults.
    ///
    /// ```json
    /// {
    ///   "linter": { "rules": { "nursery": {
    ///     "noUndefinedSubjectReference": {
    ///       "level": "info",
    ///       "options": {
    ///         "allowedPrefixes": ["org:", "geo:"]
    ///       }
    ///     }
    ///   }}}
    /// }
    /// ```
    ///
    pub NoUndefinedSubjectReference {
        version: "next",
        name: "noUndefinedSubjectReference",
        language: "turtle",
        recommended: false,
        severity: Severity::Information,
    }
}

/// Well-known vocabulary prefixes that are always allowed.
const BUILTIN_ALLOWED: &[&str] = &[
    "rdf:", "rdfs:", "owl:", "xsd:", "dc:", "dcterms:", "skos:", "foaf:", "schema:", "sh:",
    "prov:", "dcat:",
];

pub struct UndefinedSubjectRef {
    range: TextRange,
    object: String,
}

/// Check if a triple object text looks like a literal (not an IRI reference).
fn is_literal(object: &str) -> bool {
    object.starts_with('"')
        || object.starts_with('\'')
        || object == "true"
        || object == "false"
        || object.starts_with(|c: char| c.is_ascii_digit())
        || object.starts_with('+')
        || object.starts_with('-')
}

/// Check if a triple object text is a blank node.
fn is_blank_node(object: &str) -> bool {
    object.starts_with("_:") || object.starts_with('[')
}

/// Extract the namespace prefix from a prefixed name (e.g., "ex:" from "ex:bob").
fn extract_prefix(object: &str) -> Option<&str> {
    object.find(':').map(|pos| &object[..=pos])
}

impl Rule for NoUndefinedSubjectReference {
    type Query = Semantic<TurtleRoot>;
    type State = UndefinedSubjectRef;
    type Signals = Vec<Self::State>;
    type Options = NoUndefinedSubjectReferenceOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let model = ctx.model();
        let options = ctx.options();
        let triples = model.triples();

        // Collect all defined subjects
        let defined_subjects: HashSet<&str> = triples.iter().map(|t| t.subject.as_str()).collect();

        // Build allowed-prefixes set: built-in + user-configured
        let mut allowed: HashSet<&str> = BUILTIN_ALLOWED.iter().copied().collect();
        if let Some(user_allowed) = &options.allowed_prefixes {
            for prefix in user_allowed.iter() {
                allowed.insert(prefix.as_ref());
            }
        }

        // Track which objects we've already reported to avoid duplicates
        let mut reported: HashSet<&str> = HashSet::new();
        let mut signals = Vec::new();

        for triple in triples {
            let object = triple.object.as_str();

            // Skip literals
            if is_literal(object) {
                continue;
            }

            // Skip blank nodes
            if is_blank_node(object) {
                continue;
            }

            // Skip full IRIs in angle brackets
            if object.starts_with('<') {
                continue;
            }

            // Must be a prefixed name — extract namespace prefix
            let Some(prefix) = extract_prefix(object) else {
                continue;
            };

            // Skip if prefix is allowed (built-in or user-configured)
            if allowed.contains(prefix) {
                continue;
            }

            // Skip if defined as a subject
            if defined_subjects.contains(object) {
                continue;
            }

            // Skip if already reported
            if !reported.insert(object) {
                continue;
            }

            signals.push(UndefinedSubjectRef {
                range: triple.statement_range,
                object: object.to_string(),
            });
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "'"{ &state.object }"' is used as an object but never defined as a subject in this document."
                },
            )
            .note(markup! {
                "If this resource is defined externally, add its prefix to the 'allowedPrefixes' option."
            }),
        )
    }
}
