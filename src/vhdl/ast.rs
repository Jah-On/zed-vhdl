//! AST nodes produced by the custom VHDL parser.
//!
//! The AST is intentionally focused on the constructs that matter for
//! highlighting and formatting; nested bodies are represented as a block
//! of statements. Recursive nodes use `Box` to break the type cycle.

/// A VHDL AST node.
#[derive(Debug)]
pub enum Node {
    // Top-level declarations
    EntityDeclaration {
        name: String,
    },
    ComponentDeclaration {
        name: String,
    },
    PackageDeclaration {
        name: String,
    },
    FullTypeDeclaration {
        name: String,
    },
    SubtypeDeclaration {
        name: String,
    },
    IncompleteTypeDeclaration {
        name: String,
    },
    RecordTypeDefinition {
        name: String,
    },
    ConstantDeclaration {
        names: Vec<String>,
    },
    SignalDeclaration {
        names: Vec<String>,
    },
    VariableDeclaration {
        names: Vec<String>,
    },
    SharedVariableDeclaration {
        names: Vec<String>,
    },
    FileDeclaration {
        names: Vec<String>,
    },
    AliasDeclaration {
        name: String,
        target: Box<Node>,
    },
    AttributeDeclaration {
        name: String,
    },

    // Bodies and declarations with bodies
    FunctionBody {
        name: String,
        body: Box<Node>,
    },
    ProcedureBody {
        name: String,
        body: Box<Node>,
    },
    FunctionDeclaration {
        name: String,
        body: Box<Node>,
    },
    ProcedureDeclaration {
        name: String,
        body: Box<Node>,
    },
    AttributeSpecification {
        name: String,
        entity: Box<Node>,
    },

    // Hierarchy / structure
    ComponentDeclarationInstantiation {
        component: String,
        name: Option<String>,
    },
    ComponentInstantiationStatement {
        name: Option<String>,
        entity: Box<Node>,
    },
    ConfigurationDeclaration {
        name: String,
        entity: String,
    },
    BlockSpecification {
        name_or_label: String,
    },
    ContextDeclaration {
        name: String,
    },
    GroupDeclaration {
        name: String,
    },
    GroupTemplateDeclaration {
        name: String,
    },
    PrimaryUnitDeclaration {
        name: String,
    },
    SecondaryUnitDeclaration {
        name: String,
    },
    ElementDeclaration {
        names: Vec<String>,
    },

    // Bodies
    PackageBody {
        package: String,
        body: Box<Node>,
    },
    ArchitectureBody {
        entity: String,
        name: String,
        body: Box<Node>,
    },
    ProcessStatement {
        name: Option<String>,
        body: Box<Node>,
    },
    IfStatement {
        branches: Vec<(Box<Node>, Box<Node>)>,
    },

    // Generic block of statements (function/procedure/package/architecture
    // bodies, expression bodies, etc.)
    Block {
        items: Vec<Box<Node>>,
    },
}

/// The label used by the parser for node type reporting.
pub const NODE_TYPES: &str = "node";

#[cfg(test)]
mod tests {
    #[test]
    fn node_types_label_exists() {
        use crate::vhdl::ast::NODE_TYPES;
        assert_eq!(NODE_TYPES, "node");
    }
}
