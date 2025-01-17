use debug::DebugWithDb;
use smol_str::SmolStr;
use syntax::node::ast::OptionAttributeArgs;
use syntax::node::db::SyntaxGroup;
use syntax::node::{ast, Terminal, TypedSyntaxNode};

use crate::db::SemanticGroup;

/// Semantic representation of an attribute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub id: SmolStr,
    pub args: Vec<ast::Expr>,
}

impl DebugWithDb<dyn SemanticGroup> for Attribute {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &(dyn SemanticGroup + 'static),
    ) -> std::fmt::Result {
        write!(f, r#"Attribute {{ id: "{}""#, self.id)?;
        if !self.args.is_empty() {
            write!(f, ", args: [")?;
            for arg in self.args.iter() {
                write!(f, "{:?}, ", arg.as_syntax_node().get_text(db.upcast()))?;
            }
            write!(f, "]")?;
        }
        write!(f, " }}")
    }
}

/// Returns the semantic attributes for the given AST attribute list.
pub fn ast_attributes_to_semantic(
    syntax_db: &dyn SyntaxGroup,
    attributes: ast::AttributeList,
) -> Vec<Attribute> {
    // TODO(ilya): Consider checking for attribute repetitions.
    attributes
        .elements(syntax_db)
        .into_iter()
        .map(|attribute| Attribute {
            id: attribute.attr(syntax_db).text(syntax_db),
            args: match attribute.args(syntax_db) {
                OptionAttributeArgs::AttributeArgs(attribute_args) => {
                    attribute_args.arg_list(syntax_db).elements(syntax_db)
                }
                OptionAttributeArgs::Empty(_) => vec![],
            },
        })
        .collect()
}
