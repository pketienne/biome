# Quality Audit Framework — Complexity vs Duplication Independence

Working notes examining whether Code Duplication is a subtype of Code Complexity or an independent category.

---

## The question

Code duplication increases the cognitive effort to work with a codebase. Code complexity increases the cognitive effort to work with a codebase. Are they the same dimension?

## The ambiguity comes from two scopes of "cognitive complexity"

**Cognitive complexity as formally defined** (the SonarSource metric, what the framework measures) is a property of a single function: nesting depth, control flow breaks, logical operator sequences. It analyzes one function's AST in isolation. A duplicated function has the exact same cognitive complexity score whether zero copies or fifty copies exist elsewhere.

**"Cognitive complexity" as informally used** — "how hard is this codebase to work with mentally" — absolutely includes duplication. You have to hold in your head that changes need to be made in N places, that copies may have silently diverged, that fixing a bug here doesn't fix it there.

These are two different scopes of analysis. The framework's Code Complexity category measures the first. Duplication lives in the second.

## Conceptual distinction

| | Code Complexity | Code Duplication |
|---|---|---|
| **Property of** | A single function (node) | Relationships between code locations (edges) |
| **Measured by** | Analyzing one function's AST in isolation | Comparing code across the entire codebase |
| **Detection tool** | Complexity analyzer (never looks at more than one function) | Clone detector (must see all code simultaneously to find matches) |
| **Risk** | Bugs hiding in hard-to-follow local logic | Bugs fixed in one place but not N others |
| **Scope of harm** | Local — this function is fragile | Global — the maintenance surface is multiplied |
| **Remediation** | Simplify the function (extract helpers, early returns, flatten nesting) | Extract shared abstraction that replaces N copies |
| **Remediation scope** | Local operation — one function gets simpler | Architectural operation — creates shared modules, changes dependency graphs, touches multiple files |
| **Can exist without the other** | A function can have cyclomatic 50 with zero duplication anywhere | A codebase can have massive duplication of trivially simple one-liners |

The last row is the decisive test. If each can exist independently of the other at any level, neither is a subtype of the other.

They do **interact** — duplicated complex code is the worst of both worlds, and the framework already acknowledges this in the "complementary refactoring" design principle. But interaction isn't subsumption. Test Coverage and Code Complexity also interact (the dual-axis risk principle), and nobody would fold them together.

## Algorithmic distinction

The conceptual distinction maps to a real technical one. The tools that measure these properties use completely unrelated algorithms.

### Complexity analysis: single-function graph/tree metrics

All complexity tools operate on **one function at a time**. They never see any other function.

**Cyclomatic complexity (McCabe, 1976):**
- Parse the function into a control flow graph (CFG) — nodes are basic blocks, edges are control flow transitions
- Compute: `edges - nodes + 2`
- Or equivalently: count decision points (`if`, `while`, `for`, `case`, `&&`, `||`, `catch`) + 1
- This is a graph-theoretic metric on a single function's CFG

**Cognitive complexity (SonarSource, 2016):**
- Walk the AST of a single function top-down
- Increment a counter for each control flow break (`if`, `else`, `for`, `while`, `switch`, `catch`, `goto`, labeled `break`/`continue`, logical operator sequences, recursion)
- Apply a nesting penalty: each level of nesting depth adds +1 to subsequent increments
- This is a weighted AST traversal of a single function

**Halstead complexity:**
- Count distinct and total operators and operands (pure token counting)
- Compute volume, difficulty, effort from those counts
- No structural analysis at all — just token frequencies

The data structures are: control flow graphs, abstract syntax trees, token streams. The algorithms are: graph metrics, tree traversal with accumulation, token counting. The input boundary is always: **one function**.

### Clone detection: whole-codebase comparison

All duplication tools operate on **the entire codebase simultaneously**. The concept of "duplication" is meaningless when looking at one function in isolation — you need at least two code locations to have a duplicate.

There are several algorithm families, each progressively more powerful:

**Text-based (string matching):**
- Treat source as raw character sequences
- Build a suffix tree or suffix array of the entire codebase
- Find long repeated substrings
- Tools: Duploc, NICAD (text mode)
- Fast but brittle — whitespace or comment changes break matches

**Token-based (token sequence matching):**
- Lex all source files into token streams (stripping whitespace, comments)
- Use sliding-window hashing or suffix trees over token sequences to find repeated subsequences
- Can normalize identifiers so `int x = a + b` matches `int y = c + d`
- Tools: CPD (PMD's Copy-Paste Detector), CCFinder, Simian
- Catches "Type 2" clones (renamed variables)

**AST-based (tree matching):**
- Parse all source into ASTs
- Hash subtrees, find hash collisions, verify structural equivalence
- Deckard uses characteristic vectors (numerical summaries of subtree structure) with locality-sensitive hashing to find *similar* subtrees, not just identical ones
- Tools: CloneDR, Deckard, NICAD (AST mode)
- Catches "Type 3" clones (structural similarity with statement additions/deletions)

**PDG-based (dependence graph isomorphism):**
- Build program dependence graphs (control flow + data flow combined)
- Find isomorphic subgraphs
- Can detect semantically equivalent code even with reordered statements
- Computationally expensive — subgraph isomorphism is NP-complete in the general case
- Tools: Komondoor & Horwitz's research tools

**Embedding-based (learned similarity):**
- Encode code fragments into vector representations
- Find nearest neighbors in the embedding space
- Can detect "Type 4" clones (functionally equivalent but syntactically unrelated)
- Tools: SourcererCC, Code2vec-derived tools

The data structures are: suffix trees, token hash tables, AST fingerprints, dependence graphs, embedding vectors. The algorithms are: string matching, sequence hashing, tree hashing, subgraph isomorphism, nearest-neighbor search. The input boundary is always: **the entire codebase**.

### The algorithmic gap

| | Complexity | Duplication |
|---|---|---|
| **Input** | One function | Entire codebase |
| **Output** | A number (score) | Pairs/clusters of similar locations |
| **Core algorithm family** | Graph metrics, weighted tree traversal | String/sequence/tree matching, subgraph isomorphism |
| **Data structures** | CFG, AST, token stream (of one function) | Suffix trees, hash tables, fingerprint databases (of all code) |
| **Computational model** | Map: function → score | Compare: all functions × all functions |
| **Can produce the other's output?** | No — has no concept of "similarity between locations" | No — has no concept of "control flow nesting depth" |

Tools that report both (SonarQube, PMD) run them as **completely separate analyses** that happen to be packaged in the same suite. SonarQube's cognitive complexity analyzer and its duplication detector share no code, no data structures, no intermediate representations. PMD's complexity rules and CPD (Copy-Paste Detector) are literally different executables.

## Conclusion

They are independent categories. The argument has three layers that reinforce each other:

1. **Conceptual:** They measure different properties (node vs edge), at different scopes (local vs global), with different risk profiles and different remediations. Each can exist at any level without the other.

2. **Algorithmic:** The tools that detect them use completely unrelated algorithms, data structures, and computational models. You couldn't combine them into a single analysis even if you wanted to.

3. **Practical:** Tools that report both always implement them as separate analyses. No tool has ever unified them because there's no unifying algorithm.

---

## Reference: Abstract Syntax Tree (AST)

An AST is the data structure a parser produces from source code — a tree where each node represents a syntactic construct (function declaration, if statement, binary expression, variable reference, etc.) and the tree structure represents how those constructs nest inside each other.

Given this code:

```
if (x > 0) {
    return x + 1;
}
```

The AST looks roughly like:

```
IfStatement
├── condition: BinaryExpression (>)
│   ├── left: Identifier (x)
│   └── right: Literal (0)
└── body: Block
    └── ReturnStatement
        └── value: BinaryExpression (+)
            ├── left: Identifier (x)
            └── right: Literal (1)
```

The "abstract" in AST means it drops syntactic noise that doesn't affect meaning — parentheses, semicolons, braces, whitespace. The tree captures *structure and relationships*, not formatting. `x+1` and `x + 1` and `(x + 1)` all produce the same AST node.

This is why AST-based tools are more powerful than text-based tools for both complexity and duplication analysis — they operate on meaning rather than characters. It's also why complexity and duplication tools, despite both using ASTs, do fundamentally different things with them: complexity tools walk one tree and count; duplication tools compare many trees for similarity.
