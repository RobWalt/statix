use crate::{
    make,
    session::{SessionInfo, Version},
    Metadata, Report, Rule, Suggestion,
};
use rowan::ast::AstNode;

use macros::lint;
use rnix::{ast::Select, NodeOrToken, SyntaxElement, SyntaxKind};

/// ## What it does
/// Checks for `lib.groupBy`.
///
/// ## Why is this bad?
/// Nix 2.5 introduces `builtins.groupBy` which is faster and does
/// not require a lib import.
///
/// ## Example
///
/// ```nix
/// lib.groupBy (x: if x > 2 then "big" else "small") [ 1 2 3 4 5 6 ];
/// # { big = [ 3 4 5 6 ]; small = [ 1 2 ]; }
/// ```
///
/// Replace `lib.groupBy` with `builtins.groupBy`:
///
/// ```nix
/// builtins.groupBy (x: if x > 2 then "big" else "small") [ 1 2 3 4 5 6 ];
/// ```
#[lint(
    name = "faster_groupby",
    note = "Found lib.groupBy",
    code = 15,
    match_with = SyntaxKind::NODE_SELECT
)]
struct FasterGroupBy;

impl Rule for FasterGroupBy {
    fn validate(&self, node: &SyntaxElement, sess: &SessionInfo) -> Option<Report> {
        let lint_version = "2.5".parse::<Version>().unwrap();

        if sess.version() < &lint_version {
            return None;
        };
        let NodeOrToken::Node(node) = node else {
            return None;
        };
        let select_expr = Select::cast(node.clone())?;
        let select_from = select_expr.expr()?;
        let group_by_attr = select_expr.attrpath()?;

        let select_from_text = select_from.syntax().text().to_string();
        let group_by_prefix = group_by_attr
            .attrs()
            .take(group_by_attr.attrs().count() - 1)
            .map(|attr| attr.to_string())
            .collect::<Vec<_>>();
        let group_by_from = std::iter::once(select_from_text)
            .chain(group_by_prefix)
            .collect::<Vec<_>>()
            .join(".");

        // a heuristic to lint on nixpkgs.lib.groupBy
        // and lib.groupBy and its variants
        if group_by_from == "builtins" {
            return None;
        };
        if !group_by_attr
            .syntax()
            .text()
            .to_string()
            .ends_with("groupBy")
        {
            return None;
        };

        let at = node.text_range();
        let replacement = {
            let builtins = make::ident("builtins");
            make::select(builtins.syntax(), group_by_attr.syntax())
                .syntax()
                .clone()
        };
        let message = format!("Prefer `builtins.groupBy` over `{}.groupBy`", group_by_from);
        Some(
            self.report()
                .suggest(at, message, Suggestion::new(at, replacement)),
        )
    }
}
