use crate::{session::SessionInfo, Metadata, Report, Rule, Suggestion};
use rowan::ast::AstNode;

use macros::lint;
use rnix::{
    ast::{HasEntry, LetIn},
    NodeOrToken, SyntaxElement, SyntaxKind,
};

/// ## What it does
/// Checks for `let-in` expressions which create no new bindings.
///
/// ## Why is this bad?
/// `let-in` expressions that create no new bindings are useless.
/// These are probably remnants from debugging or editing expressions.
///
/// ## Example
///
/// ```nix
/// let in pkgs.statix
/// ```
///
/// Preserve only the body of the `let-in` expression:
///
/// ```nix
/// pkgs.statix
/// ```
#[lint(
    name = "empty_let_in",
    note = "Useless let-in expression",
    code = 2,
    match_with = SyntaxKind::NODE_LET_IN
)]
struct EmptyLetIn;

impl Rule for EmptyLetIn {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };
        let let_in_expr = LetIn::cast(node.clone())?;
        let entries = let_in_expr.entries();
        let inherits = let_in_expr.inherits();

        if entries.count() != 0 {
            return None;
        };
        if inherits.count() != 0 {
            return None;
        };

        let body = let_in_expr.body()?;

        // ensure that the let-in-expr does not have comments
        let has_comments = node
            .children_with_tokens()
            .any(|el| el.kind() == SyntaxKind::TOKEN_COMMENT);
        let at = node.text_range();
        let replacement = body;
        let message = "This let-in expression has no entries";
        Some(if has_comments {
            self.report().diagnostic(at, message)
        } else {
            self.report().suggest(
                at,
                message,
                Suggestion::new(at, replacement.syntax().clone()),
            )
        })
    }
}
