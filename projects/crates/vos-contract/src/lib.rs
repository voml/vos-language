//! VOS semantic consumer boundary for Oak-owned syntax trees.

#![warn(missing_docs)]

use oak_vos::{VosDeclaration, VosRoot, VosSyntaxNode};

/// Oak output accepted by the VOS semantic pipeline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractInput {
    root: VosRoot,
}

impl ContractInput {
    /// Wraps an Oak-built VOS root without reparsing its source.
    pub fn from_oak(root: VosRoot) -> Self {
        Self { root }
    }

    /// Returns the exact source owned by Oak.
    pub fn source(&self) -> &str {
        &self.root.source
    }

    /// Returns the lossless Oak CST for semantic traversal.
    pub fn syntax(&self) -> &VosSyntaxNode {
        &self.root.syntax
    }

    /// Returns top-level declarations emitted by Oak Builder.
    pub fn declarations(&self) -> &[VosDeclaration] {
        &self.root.declarations
    }

    /// Consumes the wrapper and returns the Oak root.
    pub fn into_oak(self) -> VosRoot {
        self.root
    }
}

#[cfg(test)]
mod tests {
    use super::ContractInput;
    use oak_vos::{VosDeclarationKind, parse};

    #[test]
    fn consumes_oak_root_without_reparsing() {
        let root = parse("table User { @@id: uuid, }").expect("Oak parses VOS");
        let input = ContractInput::from_oak(root);

        assert_eq!(input.declarations().len(), 1);
        assert_eq!(input.declarations()[0].kind, VosDeclarationKind::Table);
        assert!(input.syntax().span.end > input.syntax().span.start);
    }
}
