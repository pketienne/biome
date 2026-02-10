use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_source_rule,
};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt};
use biome_yaml_syntax::{
    AnyYamlBlockMapEntry, AnyYamlMappingImplicitKey, YamlBlockMapping, YamlLanguage,
};

declare_source_rule! {
    /// Sort the keys of a YAML mapping in natural alphabetical order.
    ///
    /// Sorting keys makes YAML files more predictable and easier to navigate,
    /// especially in configuration files.
    ///
    /// ## Examples
    ///
    /// ```yaml,expect_diff
    /// zebra: 1
    /// apple: 2
    /// mango: 3
    /// ```
    pub UseSortedKeys {
        version: "next",
        name: "useSortedKeys",
        language: "yaml",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

fn get_key_text(entry: &AnyYamlBlockMapEntry) -> Option<String> {
    match entry {
        AnyYamlBlockMapEntry::YamlBlockMapImplicitEntry(e) => {
            let key = e.key()?;
            Some(implicit_key_text(&key))
        }
        AnyYamlBlockMapEntry::YamlBlockMapExplicitEntry(e) => {
            e.key().map(|k| k.syntax().text_trimmed().to_string().trim().to_string())
        }
        AnyYamlBlockMapEntry::YamlBogusBlockMapEntry(_) => None,
    }
}

fn implicit_key_text(key: &AnyYamlMappingImplicitKey) -> String {
    key.syntax().text_trimmed().to_string().trim().to_string()
}

impl Rule for UseSortedKeys {
    type Query = Ast<YamlBlockMapping>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let mapping = ctx.query();
        let entries: Vec<_> = mapping.entries().iter().collect();

        if entries.len() < 2 {
            return None;
        }

        // Check if keys are already sorted
        let keys: Vec<Option<String>> = entries.iter().map(|e| get_key_text(e)).collect();
        let mut is_sorted = true;

        for i in 1..keys.len() {
            match (&keys[i - 1], &keys[i]) {
                (Some(prev), Some(curr)) => {
                    if prev.to_lowercase() > curr.to_lowercase() {
                        is_sorted = false;
                        break;
                    }
                }
                _ => {}
            }
        }

        if is_sorted {
            None
        } else {
            Some(())
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let mapping = ctx.query();
        Some(
            RuleDiagnostic::new(
                category!("assist/source/useSortedKeys"),
                mapping.syntax().text_trimmed_range(),
                markup! {
                    "The keys of this mapping are not sorted alphabetically."
                },
            )
            .note(markup! {
                "Sorting keys makes the file more predictable and easier to navigate."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let mapping = ctx.query();
        let mut entries: Vec<_> = mapping.entries().iter().collect();

        // Sort entries by key text (case-insensitive)
        entries.sort_by(|a, b| {
            let key_a = get_key_text(a).unwrap_or_default().to_lowercase();
            let key_b = get_key_text(b).unwrap_or_default().to_lowercase();
            key_a.cmp(&key_b)
        });

        let mut mutation = ctx.root().begin();

        // Replace each entry position with the sorted entry
        let original_entries: Vec<_> = mapping.entries().iter().collect();
        for (original, sorted) in original_entries.iter().zip(entries.iter()) {
            if original.syntax().text_trimmed_range() != sorted.syntax().text_trimmed_range() {
                mutation.replace_node(original.clone(), sorted.clone());
            }
        }

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Sort the keys alphabetically." }.to_owned(),
            mutation,
        ))
    }
}
