use biome_analyze::{
    AddVisitor, FromServices, Phase, Phases, QueryKey, Queryable, RuleKey, RuleMetadata,
    ServiceBag, ServicesDiagnostic, SyntaxVisitor,
};
use biome_rowan::AstNode;
use biome_turtle_semantic::model::SemanticModel;
use biome_turtle_syntax::{TurtleLanguage, TurtleRoot, TurtleSyntaxNode};

pub struct SemanticServices {
    model: SemanticModel,
}

impl SemanticServices {
    pub fn model(&self) -> &SemanticModel {
        &self.model
    }
}

impl FromServices for SemanticServices {
    fn from_services(
        rule_key: &RuleKey,
        _rule_metadata: &RuleMetadata,
        services: &ServiceBag,
    ) -> Result<Self, ServicesDiagnostic> {
        let model: &SemanticModel = services
            .get_service()
            .ok_or_else(|| ServicesDiagnostic::new(rule_key.rule_name(), &["SemanticModel"]))?;

        Ok(Self {
            model: model.clone(),
        })
    }
}

impl Phase for SemanticServices {
    fn phase() -> Phases {
        Phases::Syntax
    }
}

/// The [Semantic] type usable by lint rules **that use the semantic model** to match on specific [AstNode] types.
///
/// ```ignore
/// impl Rule for SampleTurtleLintRule {
///    type Query = Semantic<TurtleRoot>;
///    type State = ();
///    type Signals = Option<Self::State>;
///    type Options = ();
///    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
///     let node = ctx.query();
///     let model = ctx.model();
///     for triple in model.triples() {
///       // Do something with the triples
///     }
///     None
///    }
/// }
/// ```
#[derive(Clone)]
pub struct Semantic<N>(pub N);

impl<N> Queryable for Semantic<N>
where
    N: AstNode<Language = TurtleLanguage> + 'static,
{
    type Input = TurtleSyntaxNode;
    type Output = N;

    type Language = TurtleLanguage;
    type Services = SemanticServices;

    fn build_visitor(analyzer: &mut impl AddVisitor<TurtleLanguage>, _root: &TurtleRoot) {
        analyzer.add_visitor(Phases::Syntax, SyntaxVisitor::default);
    }

    fn key() -> QueryKey<Self::Language> {
        QueryKey::Syntax(N::KIND_SET)
    }

    fn unwrap_match(_: &ServiceBag, node: &Self::Input) -> Self::Output {
        N::unwrap_cast(node.clone())
    }
}
