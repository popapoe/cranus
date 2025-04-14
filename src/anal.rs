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
    for routine in tree.routines {
        anal.anal(routine)?;
    }
    Ok(anal.into_graph()?)
}

struct Anal {
    nodees: std::vec::Vec<crate::graph::Node>,
    end: usize,
    patchs: std::collections::HashMap<
        std::string::String,
        Patch<std::vec::Vec<crate::graph::Node>, crate::graph::Routine>,
    >,
}

impl Anal {
    fn new() -> Self {
        Anal {
            nodees: vec![crate::graph::Node::End],
            end: 0,
            patchs: std::collections::HashMap::new(),
        }
    }
    fn add_node(&mut self, node: crate::graph::Node) -> usize {
        let index = self.nodees.len();
        self.nodees.push(node);
        index
    }
    fn into_graph(
        self,
    ) -> std::result::Result<crate::graph::Graph, std::boxed::Box<dyn std::error::Error>> {
        let mut routines = std::collections::HashMap::new();
        for (name, patch) in self.patchs {
            if let Some(routine) = patch.into_inner() {
                routines.insert(name, routine);
            } else {
                return Err(std::boxed::Box::new(Error::UnknownRoutine(name)));
            }
        }
        Ok(crate::graph::Graph {
            nodees: self.nodees,
            routines,
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
                self.patchs
                    .entry(name.clone())
                    .or_insert(Patch::new())
                    .call_back(&mut self.nodees, std::boxed::Box::new(callback))?;
                Ok(())
            }
        }
    }
    fn anal(
        &mut self,
        routine: crate::tree::Routine,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        let end = self.end;
        let mut routine_anal = RoutineAnal::with_anal(self);
        let start = routine_anal.anal_statements(end, routine.body)?;
        routine_anal.finish()?;
        self.patchs
            .entry(routine.name.clone())
            .or_insert(Patch::new())
            .patch(
                &mut self.nodees,
                crate::graph::Routine {
                    start,
                    formals: routine.formals.clone(),
                },
            )?;
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
                        .patchs
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
