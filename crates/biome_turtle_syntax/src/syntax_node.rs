//! This module defines the Concrete Syntax Tree used by Biome.
//!
//! The tree is entirely lossless, whitespace, comments, and errors are preserved.
//! It also provides traversal methods including parent, children, and siblings of nodes.
//!
//! This is a simple wrapper around the `rowan` crate which does most of the heavy lifting and is language agnostic.

use crate::{TurtleRoot, TurtleSyntaxKind};
use biome_rowan::Language;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TurtleLanguage;

impl Language for TurtleLanguage {
    type Kind = TurtleSyntaxKind;
    type Root = TurtleRoot;
}

pub type TurtleSyntaxNode = biome_rowan::SyntaxNode<TurtleLanguage>;
pub type TurtleSyntaxToken = biome_rowan::SyntaxToken<TurtleLanguage>;
pub type TurtleSyntaxElement = biome_rowan::SyntaxElement<TurtleLanguage>;
pub type TurtleSyntaxElementChildren = biome_rowan::SyntaxElementChildren<TurtleLanguage>;
pub type TurtleSyntaxList = biome_rowan::SyntaxList<TurtleLanguage>;
