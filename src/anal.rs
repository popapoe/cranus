#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    WrongActualCount(std::string::String),
    UnknownLabel(std::string::String),
    UnknownRoutine(std::string::String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WrongActualCount(name) => {
                write!(f, "incorrect number of actuals for {:?}", name)?;
            }
            Error::UnknownLabel(name) => {
                write!(f, "unknown label {:?}", name)?;
            }
            Error::UnknownRoutine(name) => {
                write!(f, "unknown routine {:?}", name)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

pub fn anal(
    tree: crate::tree::Tree,
) -> std::result::Result<crate::graph::Graph, std::boxed::Box<dyn std::error::Error>> {
    let mut anal = Anal::new();
    for r#type in tree.typees {
        anal.anal_type(r#type)?;
    }
    for routine in tree.routinees {
        anal.anal_routine(routine)?;
    }
    Ok(anal.into_graph()?)
}

struct Anal {
    typees: std::vec::Vec<crate::graph::TypeNode>,
    nodees: std::vec::Vec<crate::graph::Node>,
    type_end: usize,
    node_end: usize,
    type_patchs: std::collections::HashMap<
        std::string::String,
        Patch<std::vec::Vec<crate::graph::TypeNode>, usize>,
    >,
    routine_patchs: std::collections::HashMap<
        std::string::String,
        Patch<std::vec::Vec<crate::graph::Node>, crate::graph::Routine>,
    >,
}

impl Anal {
    fn new() -> Self {
        Anal {
            typees: vec![crate::graph::TypeNode::One],
            nodees: vec![crate::graph::Node::End],
            type_end: 0,
            node_end: 0,
            type_patchs: std::collections::HashMap::new(),
            routine_patchs: std::collections::HashMap::new(),
        }
    }
    fn add_variable(&mut self, node: usize, is_dual: bool) -> usize {
        let index = self.typees.len();
        self.typees.push(crate::graph::TypeNode::Variable {
            node,
            is_dual,
            dual: index + 1,
        });
        self.typees.push(crate::graph::TypeNode::Variable {
            node,
            is_dual: !is_dual,
            dual: index,
        });
        index
    }
    fn add_receive(&mut self, value: usize, next: usize) -> usize {
        let index = self.typees.len();
        self.typees.push(crate::graph::TypeNode::Lollipop {
            value,
            next,
            dual: index + 1,
        });
        self.typees.push(crate::graph::TypeNode::Times {
            value,
            next: crate::graph::get_dual(&self.typees, next),
            dual: index,
        });
        index
    }
    fn add_send(&mut self, value: usize, next: usize) -> usize {
        let index = self.typees.len();
        self.typees.push(crate::graph::TypeNode::Times {
            value,
            next,
            dual: index + 1,
        });
        self.typees.push(crate::graph::TypeNode::Lollipop {
            value,
            next: crate::graph::get_dual(&self.typees, next),
            dual: index,
        });
        index
    }
    fn add_offer(&mut self, accept: usize, deny: usize) -> usize {
        let index = self.typees.len();
        self.typees.push(crate::graph::TypeNode::With {
            accept,
            deny,
            dual: index + 1,
        });
        self.typees.push(crate::graph::TypeNode::Plus {
            accept: crate::graph::get_dual(&self.typees, accept),
            deny: crate::graph::get_dual(&self.typees, deny),
            dual: index,
        });
        index
    }
    fn add_choose(&mut self, accept: usize, deny: usize) -> usize {
        let index = self.typees.len();
        self.typees.push(crate::graph::TypeNode::Plus {
            accept,
            deny,
            dual: index + 1,
        });
        self.typees.push(crate::graph::TypeNode::With {
            accept: crate::graph::get_dual(&self.typees, accept),
            deny: crate::graph::get_dual(&self.typees, deny),
            dual: index,
        });
        index
    }
    fn add_node(&mut self, node: crate::graph::Node) -> usize {
        let index = self.nodees.len();
        self.nodees.push(node);
        index
    }
    fn into_graph(
        self,
    ) -> std::result::Result<crate::graph::Graph, std::boxed::Box<dyn std::error::Error>> {
        let mut routinees = std::collections::HashMap::new();
        for (name, patch) in self.routine_patchs {
            if let Some(routine) = patch.into_inner() {
                routinees.insert(name, routine);
            } else {
                return Err(std::boxed::Box::new(Error::UnknownRoutine(name)));
            }
        }
        Ok(crate::graph::Graph {
            typees: self.typees,
            nodees: self.nodees,
            routinees,
        })
    }
    fn check_expression(
        &mut self,
        expression: &crate::tree::Expression,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        match expression {
            crate::tree::Expression::Variable { .. } => Ok(()),
            crate::tree::Expression::Call {
                name,
                before,
                after,
            } => {
                for actual in before {
                    self.check_expression(actual)?;
                }
                for actual in after {
                    self.check_expression(actual)?;
                }
                let actual_count = before.len() + 1 + after.len();
                let name_clone = name.clone();
                let callback = move |_: &mut std::vec::Vec<crate::graph::Node>,
                                     routine: &crate::graph::Routine|
                      -> std::result::Result<
                    (),
                    std::boxed::Box<dyn std::error::Error>,
                > {
                    if routine.formals.len() != actual_count {
                        return Err(std::boxed::Box::new(Error::WrongActualCount(name_clone)));
                    }
                    Ok(())
                };
                self.routine_patchs
                    .entry(name.clone())
                    .or_insert(Patch::new())
                    .call_back(&mut self.nodees, std::boxed::Box::new(callback))?;
                Ok(())
            }
        }
    }
    fn anal_type(
        &mut self,
        r#type: crate::tree::Type,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        let value = self.anal_type_expression(*r#type.value)?;
        self.type_patchs
            .entry(r#type.name.clone())
            .or_insert(Patch::new())
            .patch(&mut self.typees, value)?;
        Ok(())
    }
    fn anal_type_expression(
        &mut self,
        expression: crate::tree::TypeExpression,
    ) -> std::result::Result<usize, std::boxed::Box<dyn std::error::Error>> {
        match expression {
            crate::tree::TypeExpression::Variable { name, is_dual } => {
                let index = self.add_variable(0, is_dual);
                let callback = move |typees: &mut std::vec::Vec<crate::graph::TypeNode>,
                                     node: &usize|
                      -> std::result::Result<
                    (),
                    std::boxed::Box<dyn std::error::Error>,
                > {
                    let dual = crate::graph::get_dual(&typees, index);
                    match &mut typees[index] {
                        crate::graph::TypeNode::Variable { node: pointer, .. } => *pointer = *node,
                        _ => unreachable!(),
                    }
                    match &mut typees[dual] {
                        crate::graph::TypeNode::Variable { node: pointer, .. } => *pointer = *node,
                        _ => unreachable!(),
                    }
                    Ok(())
                };
                self.type_patchs
                    .entry(name)
                    .or_insert(Patch::new())
                    .call_back(&mut self.typees, std::boxed::Box::new(callback))?;
                Ok(index)
            }
            crate::tree::TypeExpression::Lollipop { value, next } => {
                let value = self.anal_type_expression(*value)?;
                let next = self.anal_type_expression(*next)?;
                Ok(self.add_receive(value, next))
            }
            crate::tree::TypeExpression::Times { value, next } => {
                let value = self.anal_type_expression(*value)?;
                let next = self.anal_type_expression(*next)?;
                Ok(self.add_send(value, next))
            }
            crate::tree::TypeExpression::With { accept, deny } => {
                let accept = self.anal_type_expression(*accept)?;
                let deny = self.anal_type_expression(*deny)?;
                Ok(self.add_offer(accept, deny))
            }
            crate::tree::TypeExpression::Plus { accept, deny } => {
                let accept = self.anal_type_expression(*accept)?;
                let deny = self.anal_type_expression(*deny)?;
                Ok(self.add_choose(accept, deny))
            }
            crate::tree::TypeExpression::One => Ok(self.type_end),
        }
    }
    fn anal_routine(
        &mut self,
        routine: crate::tree::Routine,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        let end = self.node_end;
        let mut formals = std::vec::Vec::with_capacity(routine.formals.len());
        for crate::tree::Formal { name, r#type } in routine.formals {
            let r#type = self.anal_type_expression(r#type)?;
            formals.push(crate::graph::Formal { name, r#type });
        }
        let mut routine_anal = RoutineAnal::with_anal(self);
        let start = routine_anal.anal_statements(end, routine.body)?;
        routine_anal.finish()?;
        self.routine_patchs
            .entry(routine.name.clone())
            .or_insert(Patch::new())
            .patch(&mut self.nodees, crate::graph::Routine { start, formals })?;
        Ok(())
    }
}

struct RoutineAnal<'a> {
    anal: &'a mut Anal,
    patchs: std::collections::HashMap<
        std::string::String,
        Patch<std::vec::Vec<crate::graph::Node>, usize>,
    >,
}

impl<'a> RoutineAnal<'a> {
    fn with_anal(anal: &'a mut Anal) -> Self {
        RoutineAnal {
            anal,
            patchs: std::collections::HashMap::new(),
        }
    }
    fn finish(self) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        for (name, patch) in self.patchs {
            if patch.get().is_none() {
                return Err(std::boxed::Box::new(Error::UnknownLabel(name)));
            }
        }
        Ok(())
    }
    fn anal_statements(
        &mut self,
        mut last: usize,
        statements: std::vec::Vec<crate::tree::Statement>,
    ) -> std::result::Result<usize, std::boxed::Box<dyn std::error::Error>> {
        for statement in statements.into_iter().rev() {
            match statement {
                crate::tree::Statement::Branch { name } => {
                    last = self.anal.add_node(crate::graph::Node::Branch { next: 0 });
                    let index = last;
                    let callback = move |nodees: &mut std::vec::Vec<crate::graph::Node>,
                                         next: &usize|
                          -> std::result::Result<
                        (),
                        std::boxed::Box<dyn std::error::Error>,
                    > {
                        match &mut nodees[index] {
                            crate::graph::Node::Branch { next: pointer } => *pointer = *next,
                            _ => unreachable!(),
                        }
                        Ok(())
                    };
                    self.patchs
                        .entry(name)
                        .or_insert(Patch::new())
                        .call_back(&mut self.anal.nodees, std::boxed::Box::new(callback))?;
                }
                crate::tree::Statement::Label { name } => self
                    .patchs
                    .entry(name)
                    .or_insert(Patch::new())
                    .patch(&mut self.anal.nodees, last)?,
                crate::tree::Statement::Assign { name, value } => {
                    self.anal.check_expression(&value)?;
                    last = self.anal.add_node(crate::graph::Node::Assign {
                        name,
                        value,
                        next: last,
                    });
                }
                crate::tree::Statement::Call { name, actuals } => {
                    for actual in actuals.iter() {
                        self.anal.check_expression(actual)?;
                    }
                    let actual_count = actuals.len();
                    last = self.anal.add_node(crate::graph::Node::Call {
                        name: name.clone(),
                        actuals,
                        next: last,
                    });
                    let name_clone = name.clone();
                    let callback = move |_: &mut std::vec::Vec<crate::graph::Node>,
                                         routine: &crate::graph::Routine|
                          -> std::result::Result<
                        (),
                        std::boxed::Box<dyn std::error::Error>,
                    > {
                        if routine.formals.len() != actual_count {
                            return Err(std::boxed::Box::new(Error::WrongActualCount(name_clone)));
                        }
                        Ok(())
                    };
                    self.anal
                        .routine_patchs
                        .entry(name)
                        .or_insert(Patch::new())
                        .call_back(&mut self.anal.nodees, std::boxed::Box::new(callback))?;
                }
                crate::tree::Statement::Receive { source, variable } => {
                    last = self.anal.add_node(crate::graph::Node::Receive {
                        source,
                        variable,
                        next: last,
                    });
                }
                crate::tree::Statement::Send {
                    destination,
                    variable,
                } => {
                    last = self.anal.add_node(crate::graph::Node::Send {
                        destination,
                        variable,
                        next: last,
                    });
                }
                crate::tree::Statement::Offer {
                    client,
                    accepted,
                    denied,
                } => {
                    let accepted = self.anal_statements(last, accepted)?;
                    let denied = self.anal_statements(last, denied)?;
                    last = self.anal.add_node(crate::graph::Node::Offer {
                        client,
                        accepted,
                        denied,
                    });
                }
                crate::tree::Statement::Accept { server } => {
                    last = self
                        .anal
                        .add_node(crate::graph::Node::Accept { server, next: last });
                }
                crate::tree::Statement::Deny { server } => {
                    last = self
                        .anal
                        .add_node(crate::graph::Node::Deny { server, next: last });
                }
                crate::tree::Statement::Close { name } => {
                    last = self
                        .anal
                        .add_node(crate::graph::Node::Close { name, next: last });
                }
                crate::tree::Statement::Connect { left, right } => {
                    last = self.anal.add_node(crate::graph::Node::Connect {
                        left,
                        right,
                        next: last,
                    });
                }
            }
        }
        Ok(last)
    }
}

enum Patch<C, T> {
    Unpatched(
        std::vec::Vec<
            std::boxed::Box<
                dyn FnOnce(
                    &mut C,
                    &T,
                )
                    -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>>,
            >,
        >,
    ),
    Patched(T),
}

impl<C, T> Patch<C, T> {
    fn new() -> Self {
        Patch::Unpatched(vec![])
    }
    fn get(&self) -> std::option::Option<&T> {
        match self {
            Patch::Unpatched(_) => None,
            Patch::Patched(value) => Some(value),
        }
    }
    fn into_inner(self) -> std::option::Option<T> {
        match self {
            Patch::Unpatched(_) => None,
            Patch::Patched(value) => Some(value),
        }
    }
    fn patch(
        &mut self,
        context: &mut C,
        value: T,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        match std::mem::replace(self, Patch::Patched(value)) {
            Patch::Unpatched(callbacks) => match self {
                Patch::Unpatched(_) => unreachable!(),
                Patch::Patched(value) => {
                    for callback in callbacks {
                        callback(context, value)?;
                    }
                }
            },
            Patch::Patched(_) => panic!(),
        }
        Ok(())
    }
    fn call_back(
        &mut self,
        context: &mut C,
        callback: std::boxed::Box<
            dyn FnOnce(
                &mut C,
                &T,
            )
                -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>>,
        >,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        match self {
            Patch::Unpatched(callbacks) => callbacks.push(callback),
            Patch::Patched(value) => callback(context, value)?,
        }
        Ok(())
    }
}
