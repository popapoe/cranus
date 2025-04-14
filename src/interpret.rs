#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    NoMain,
    WrongMainFormalCount,
    TypeError,
    Overwriting(std::string::String),
    UnboundVariable(std::string::String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoMain => {
                write!(f, "no main routine")?;
            }
            Error::WrongMainFormalCount => {
                write!(f, "wrong main formal count")?;
            }
            Error::TypeError => {
                write!(f, "type error")?;
            }
            Error::Overwriting(name) => {
                write!(f, "overwriting {:?}", name)?;
            }
            Error::UnboundVariable(name) => {
                write!(f, "unbound variable {:?}", name)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

pub fn interpret(
    graph: crate::graph::Graph,
) -> std::result::Result<u32, std::boxed::Box<dyn std::error::Error>> {
    let mut state = 0;
    let mut interpreter = Interpreter::create(&graph, &mut state)?;
    while interpreter.step()? {}
    Ok(state)
}

struct ActiveRoutine<'a> {
    node: usize,
    children: std::collections::HashMap<std::string::String, InactiveRoutine<'a>>,
    to_interaction: std::string::String,
}

enum InactiveRoutine<'a> {
    Interaction {
        state: &'a mut InteractionState,
    },
    InteractionEnd,
    Graph {
        node: usize,
        children: std::collections::HashMap<std::string::String, InactiveRoutine<'a>>,
        parent: std::string::String,
        to_interaction: std::string::String,
    },
}

type InteractionState = u32;

struct Interpreter<'a> {
    graph: &'a crate::graph::Graph,
    active: ActiveRoutine<'a>,
}

impl<'a> Interpreter<'a> {
    fn create(
        graph: &'a crate::graph::Graph,
        state: &'a mut InteractionState,
    ) -> std::result::Result<Self, std::boxed::Box<dyn std::error::Error>> {
        let routine = if let Some(routine) = graph.routines.get("main") {
            routine
        } else {
            return Err(std::boxed::Box::new(Error::NoMain));
        };
        let node = routine.start;
        if routine.formals.len() != 1 {
            return Err(std::boxed::Box::new(Error::WrongMainFormalCount));
        }
        let formal = &routine.formals[0];
        let mut children = std::collections::HashMap::new();
        children.insert(formal.clone(), InactiveRoutine::Interaction { state });
        Ok(Interpreter {
            graph,
            active: ActiveRoutine {
                node,
                children,
                to_interaction: formal.clone(),
            },
        })
    }
    fn step(&mut self) -> std::result::Result<bool, std::boxed::Box<dyn std::error::Error>> {
        match &self.graph.nodes[self.active.node] {
            crate::graph::Node::Branch { next } => self.active.node = *next,
            crate::graph::Node::Assign {
                name,
                value: value_expression,
                next,
            } => {
                self.active.node = *next;
                let value = self.active.evaluate(self.graph, value_expression)?;
                if !value.is_parent_to_interaction() {
                    self.active.to_interaction = name.clone();
                }
                if self.active.children.insert(name.clone(), value).is_some() {
                    return Err(std::boxed::Box::new(Error::Overwriting(name.clone())));
                }
            }
            crate::graph::Node::Call {
                name,
                actuals: actual_expressions,
                next,
            } => {
                let routine = self.graph.routines.get(name).unwrap();
                let mut to_interaction = None;
                let mut children = std::collections::HashMap::new();
                for (formal, actual_expression) in
                    routine.formals.iter().zip(actual_expressions.iter())
                {
                    let actual = self.active.evaluate(self.graph, actual_expression)?;
                    if !actual.is_parent_to_interaction() {
                        to_interaction = Some(formal);
                    }
                    children.insert(formal.clone(), actual);
                }
                if let Some(to_interaction) = to_interaction {
                    self.active = ActiveRoutine {
                        node: routine.start,
                        children,
                        to_interaction: to_interaction.clone(),
                    };
                } else {
                    self.active.node = *next;
                }
            }
            crate::graph::Node::Receive {
                source,
                variable,
                next,
            } => {
                if !self.active.children.contains_key(source) {
                    return Err(std::boxed::Box::new(Error::UnboundVariable(source.clone())));
                }
                if *source == self.active.to_interaction
                    && self
                        .active
                        .children
                        .get(source)
                        .unwrap()
                        .is_parent_principal(self.graph)
                {
                    let value = self
                        .active
                        .children
                        .get_mut(source)
                        .unwrap()
                        .receive(self.graph)?;
                    self.active.node = *next;
                    if !value.is_parent_to_interaction() {
                        self.active.to_interaction = variable.clone();
                    }
                    if self
                        .active
                        .children
                        .insert(variable.clone(), value)
                        .is_some()
                    {
                        return Err(std::boxed::Box::new(Error::Overwriting(variable.clone())));
                    }
                } else {
                    self.active.flip(source);
                }
            }
            crate::graph::Node::Send {
                destination,
                variable,
                next,
            } => {
                if !self.active.children.contains_key(destination) {
                    return Err(std::boxed::Box::new(Error::UnboundVariable(
                        destination.clone(),
                    )));
                }
                if *destination == self.active.to_interaction
                    && self
                        .active
                        .children
                        .get(destination)
                        .unwrap()
                        .is_parent_principal(self.graph)
                {
                    let value = if let Some(value) = self.active.children.remove(variable) {
                        value
                    } else {
                        return Err(std::boxed::Box::new(Error::UnboundVariable(
                            variable.clone(),
                        )));
                    };
                    self.active.node = *next;
                    if !value.is_parent_to_interaction() {
                        self.active.to_interaction = destination.clone();
                    }
                    self.active
                        .children
                        .get_mut(destination)
                        .unwrap()
                        .send(self.graph, value)?;
                } else {
                    self.active.flip(destination);
                }
            }
            crate::graph::Node::Offer {
                client,
                accepted,
                denied,
            } => {
                if !self.active.children.contains_key(client) {
                    return Err(std::boxed::Box::new(Error::UnboundVariable(client.clone())));
                }
                if *client == self.active.to_interaction
                    && self
                        .active
                        .children
                        .get(client)
                        .unwrap()
                        .is_parent_principal(self.graph)
                {
                    self.active.node = if self
                        .active
                        .children
                        .get_mut(client)
                        .unwrap()
                        .offer(self.graph)?
                    {
                        *accepted
                    } else {
                        *denied
                    };
                } else {
                    self.active.flip(client);
                }
            }
            crate::graph::Node::Accept { server, next } => {
                if !self.active.children.contains_key(server) {
                    return Err(std::boxed::Box::new(Error::UnboundVariable(server.clone())));
                }
                if *server == self.active.to_interaction
                    && self
                        .active
                        .children
                        .get(server)
                        .unwrap()
                        .is_parent_principal(self.graph)
                {
                    self.active
                        .children
                        .get_mut(server)
                        .unwrap()
                        .choose(self.graph, true)?;
                    self.active.node = *next;
                } else {
                    self.active.flip(server);
                }
            }
            crate::graph::Node::Deny { server, next } => {
                if !self.active.children.contains_key(server) {
                    return Err(std::boxed::Box::new(Error::UnboundVariable(server.clone())));
                }
                if *server == self.active.to_interaction
                    && self
                        .active
                        .children
                        .get(server)
                        .unwrap()
                        .is_parent_principal(self.graph)
                {
                    self.active
                        .children
                        .get_mut(server)
                        .unwrap()
                        .choose(self.graph, false)?;
                    self.active.node = *next;
                } else {
                    self.active.flip(server);
                }
            }
            crate::graph::Node::Close { name, .. } => {
                if !self.active.children.contains_key(name) {
                    return Err(std::boxed::Box::new(Error::UnboundVariable(name.clone())));
                }
                if *name == self.active.to_interaction
                    && self
                        .active
                        .children
                        .get(name)
                        .unwrap()
                        .is_parent_principal(self.graph)
                {
                    match self.active.children.remove(name).unwrap() {
                        InactiveRoutine::Interaction { .. } => {
                            return Err(std::boxed::Box::new(Error::TypeError));
                        }
                        InactiveRoutine::InteractionEnd => return Ok(false),
                        InactiveRoutine::Graph {
                            node,
                            children,
                            to_interaction,
                            ..
                        } => match self.graph.nodes[node] {
                            crate::graph::Node::Close { next, .. } => {
                                self.active.node = next;
                                self.active.children = children;
                                self.active.to_interaction = to_interaction;
                            }
                            _ => return Err(std::boxed::Box::new(Error::TypeError)),
                        },
                    }
                } else {
                    self.active.flip(name);
                }
            }
            crate::graph::Node::End => return Err(std::boxed::Box::new(Error::TypeError)),
        }
        Ok(true)
    }
}

impl<'a> ActiveRoutine<'a> {
    fn flip(&mut self, name: &std::string::String) {
        let (name, node, mut children, parent, to_interaction) =
            match self.children.remove_entry(name).unwrap() {
                (
                    name,
                    InactiveRoutine::Graph {
                        node,
                        children,
                        parent,
                        to_interaction,
                    },
                ) => (name, node, children, parent, to_interaction),
                _ => panic!(),
            };
        children.insert(
            parent,
            InactiveRoutine::Graph {
                node: std::mem::take(&mut self.node),
                children: std::mem::take(&mut self.children),
                parent: name,
                to_interaction: std::mem::take(&mut self.to_interaction),
            },
        );
        self.node = node;
        self.children = children;
        self.to_interaction = to_interaction;
    }
    fn evaluate(
        &mut self,
        graph: &crate::graph::Graph,
        expression: &crate::graph::Expression,
    ) -> std::result::Result<InactiveRoutine<'a>, std::boxed::Box<dyn std::error::Error>> {
        match expression {
            crate::graph::Expression::Variable { name } => {
                let value = if let Some(value) = self.children.remove(name) {
                    value
                } else {
                    return Err(std::boxed::Box::new(Error::UnboundVariable(name.clone())));
                };
                Ok(value)
            }
            crate::graph::Expression::Call {
                name,
                before,
                after,
            } => {
                let routine = graph.routines.get(name).unwrap();
                let mut children = std::collections::HashMap::new();
                let mut to_interaction_index = before.len();
                for (index, actual_expression) in (0..).zip(before.iter()) {
                    let actual = self.evaluate(graph, actual_expression)?;
                    if !actual.is_parent_to_interaction() {
                        to_interaction_index = index;
                    }
                    children.insert(routine.formals[index].clone(), actual);
                }
                for (index, actual_expression) in ((before.len() + 1)..).zip(after.iter()) {
                    let actual = self.evaluate(graph, actual_expression)?;
                    if !actual.is_parent_to_interaction() {
                        to_interaction_index = index;
                    }
                    children.insert(routine.formals[index].clone(), actual);
                }
                Ok(InactiveRoutine::Graph {
                    node: routine.start,
                    children,
                    parent: routine.formals[before.len()].clone(),
                    to_interaction: routine.formals[to_interaction_index].clone(),
                })
            }
        }
    }
}

impl<'a> InactiveRoutine<'a> {
    fn is_parent_principal(&self, graph: &crate::graph::Graph) -> bool {
        match self {
            InactiveRoutine::Interaction { .. } => true,
            InactiveRoutine::InteractionEnd => true,
            InactiveRoutine::Graph { node, parent, .. } => {
                let principal = match &graph.nodes[*node] {
                    crate::graph::Node::Receive { source, .. } => source,
                    crate::graph::Node::Send { destination, .. } => destination,
                    crate::graph::Node::Offer { client, .. } => client,
                    crate::graph::Node::Accept { server, .. } => server,
                    crate::graph::Node::Deny { server, .. } => server,
                    crate::graph::Node::Close { name, .. } => name,
                    _ => return false,
                };
                *parent == *principal
            }
        }
    }
    fn is_parent_to_interaction(&self) -> bool {
        match self {
            InactiveRoutine::Interaction { .. } => false,
            InactiveRoutine::InteractionEnd => false,
            InactiveRoutine::Graph {
                parent,
                to_interaction,
                ..
            } => parent == to_interaction,
        }
    }
    fn send(
        &mut self,
        graph: &crate::graph::Graph,
        value: InactiveRoutine<'a>,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        match self {
            InactiveRoutine::Interaction { .. } => Err(std::boxed::Box::new(Error::TypeError)),
            InactiveRoutine::InteractionEnd { .. } => Err(std::boxed::Box::new(Error::TypeError)),
            InactiveRoutine::Graph {
                node,
                children,
                to_interaction,
                ..
            } => match &graph.nodes[*node] {
                crate::graph::Node::Receive { variable, next, .. } => {
                    *node = *next;
                    if !value.is_parent_to_interaction() {
                        *to_interaction = variable.clone();
                    }
                    if children.insert(variable.clone(), value).is_some() {
                        return Err(std::boxed::Box::new(Error::Overwriting(variable.clone())));
                    }
                    Ok(())
                }
                _ => Err(std::boxed::Box::new(Error::TypeError)),
            },
        }
    }
    fn receive(
        &mut self,
        graph: &crate::graph::Graph,
    ) -> std::result::Result<InactiveRoutine<'a>, std::boxed::Box<dyn std::error::Error>> {
        match self {
            InactiveRoutine::Interaction { .. } => Err(std::boxed::Box::new(Error::TypeError)),
            InactiveRoutine::InteractionEnd { .. } => Err(std::boxed::Box::new(Error::TypeError)),
            InactiveRoutine::Graph {
                node,
                children,
                parent,
                to_interaction,
            } => match &graph.nodes[*node] {
                crate::graph::Node::Send { variable, next, .. } => {
                    *node = *next;
                    let value = if let Some(value) = children.remove(variable) {
                        value
                    } else {
                        return Err(std::boxed::Box::new(Error::UnboundVariable(
                            variable.clone(),
                        )));
                    };
                    if !value.is_parent_to_interaction() {
                        *to_interaction = parent.clone();
                    }
                    Ok(value)
                }
                _ => Err(std::boxed::Box::new(Error::TypeError)),
            },
        }
    }
    fn choose(
        &mut self,
        graph: &crate::graph::Graph,
        accept: bool,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        match self {
            InactiveRoutine::Interaction { state } => {
                if accept {
                    **state += 1;
                } else {
                    *self = InactiveRoutine::InteractionEnd;
                }
                Ok(())
            }
            InactiveRoutine::InteractionEnd { .. } => Err(std::boxed::Box::new(Error::TypeError)),
            InactiveRoutine::Graph { node, .. } => match &graph.nodes[*node] {
                crate::graph::Node::Offer {
                    accepted, denied, ..
                } => {
                    *node = if accept { *accepted } else { *denied };
                    Ok(())
                }
                _ => Err(std::boxed::Box::new(Error::TypeError)),
            },
        }
    }
    fn offer(
        &mut self,
        graph: &crate::graph::Graph,
    ) -> std::result::Result<bool, std::boxed::Box<dyn std::error::Error>> {
        match self {
            InactiveRoutine::Interaction { .. } => Err(std::boxed::Box::new(Error::TypeError)),
            InactiveRoutine::InteractionEnd { .. } => Err(std::boxed::Box::new(Error::TypeError)),
            InactiveRoutine::Graph { node, .. } => match &graph.nodes[*node] {
                crate::graph::Node::Accept { next, .. } => {
                    *node = *next;
                    Ok(true)
                }
                crate::graph::Node::Deny { next, .. } => {
                    *node = *next;
                    Ok(false)
                }
                _ => Err(std::boxed::Box::new(Error::TypeError)),
            },
        }
    }
}
