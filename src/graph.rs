#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    pub typees: std::vec::Vec<TypeNode>,
    pub nodees: std::vec::Vec<Node>,
    pub routinees: std::collections::HashMap<std::string::String, Routine>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Routine {
    pub start: usize,
    pub formals: std::vec::Vec<Formal>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Formal {
    pub name: std::string::String,
    pub r#type: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeNode {
    Variable {
        node: usize,
        is_dual: bool,
        dual: usize,
    },
    Lollipop {
        value: usize,
        next: usize,
        dual: usize,
    },
    Times {
        value: usize,
        next: usize,
        dual: usize,
    },
    With {
        accept: usize,
        deny: usize,
        dual: usize,
    },
    Plus {
        accept: usize,
        deny: usize,
        dual: usize,
    },
    One,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Node {
    Branch {
        next: usize,
    },
    Assign {
        name: std::string::String,
        value: std::boxed::Box<Expression>,
        next: usize,
    },
    Call {
        name: std::string::String,
        actuals: std::vec::Vec<Expression>,
        next: usize,
    },
    Receive {
        source: std::string::String,
        variable: std::string::String,
        next: usize,
    },
    Send {
        destination: std::string::String,
        variable: std::string::String,
        next: usize,
    },
    Offer {
        client: std::string::String,
        accepted: usize,
        denied: usize,
    },
    Accept {
        server: std::string::String,
        next: usize,
    },
    Deny {
        server: std::string::String,
        next: usize,
    },
    Close {
        name: std::string::String,
        next: usize,
    },
    End,
}

pub type Expression = crate::tree::Expression;

pub fn get_dual(typees: &[crate::graph::TypeNode], node: usize) -> usize {
    match &typees[node] {
        crate::graph::TypeNode::Variable { dual, .. } => *dual,
        crate::graph::TypeNode::Lollipop { dual, .. } => *dual,
        crate::graph::TypeNode::Times { dual, .. } => *dual,
        crate::graph::TypeNode::With { dual, .. } => *dual,
        crate::graph::TypeNode::Plus { dual, .. } => *dual,
        crate::graph::TypeNode::One => node,
    }
}
