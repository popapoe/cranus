#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Closed(std::string::String),
    NotClosed(std::string::String),
    NotLollipop(std::string::String),
    NotTimes(std::string::String),
    NotWith(std::string::String),
    NotPlus(std::string::String),
    NotOne(std::string::String),
    TypeMismatch,
    NotInReverseTopologicalOrder,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Closed(name) => {
                write!(f, "{} is closed", name)?;
            }
            Error::NotClosed(name) => {
                write!(f, "{} is not closed", name)?;
            }
            Error::NotLollipop(name) => {
                write!(f, "{} is not lollipop", name)?;
            }
            Error::NotTimes(name) => {
                write!(f, "{} is not times", name)?;
            }
            Error::NotWith(name) => {
                write!(f, "{} is not with", name)?;
            }
            Error::NotPlus(name) => {
                write!(f, "{} is not plus", name)?;
            }
            Error::NotOne(name) => {
                write!(f, "{} is not one", name)?;
            }
            Error::TypeMismatch => {
                write!(f, "type mismatch")?;
            }
            Error::NotInReverseTopologicalOrder => {
                write!(f, "not in reverse topological order")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

pub fn check(
    graph: &crate::graph::Graph,
) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    let mut checker = Checker::with_graph(graph);
    for routine in graph.routinees.values() {
        checker.initialize_routine(routine);
    }
    for index in (0..graph.nodees.len()).rev() {
        checker.check_node(index)?;
    }
    Ok(())
}

struct Epsilon(std::vec::Vec<usize>);

impl Epsilon {
    fn create(size: usize) -> Self {
        let mut transitions = std::vec::Vec::with_capacity(size);
        for index in 0..size {
            transitions.push(index);
        }
        Epsilon(transitions)
    }
    fn add(&mut self, from: usize, to: usize) {
        self.0[from] = to;
    }
    fn get(&mut self, mut index: usize) -> usize {
        while index != self.0[index] {
            self.0[index] = self.0[self.0[index]];
            index = self.0[index];
        }
        index
    }
}

struct Checker<'a> {
    graph: &'a crate::graph::Graph,
    typees:
        std::vec::Vec<std::option::Option<std::collections::HashMap<std::string::String, usize>>>,
    epsilon: Epsilon,
    classs: std::vec::Vec<usize>,
}

impl<'a> Checker<'a> {
    fn with_graph(graph: &'a crate::graph::Graph) -> Self {
        let mut typees = std::vec::Vec::with_capacity(graph.nodees.len());
        for _ in 0..graph.nodees.len() {
            typees.push(None);
        }
        let mut classs = std::vec::Vec::with_capacity(graph.typees.len() + 1);
        let mut left = std::vec::Vec::with_capacity(graph.typees.len() + 1);
        let mut right = std::vec::Vec::with_capacity(graph.typees.len() + 1);
        for _ in 0..=graph.typees.len() {
            classs.push(0);
            left.push(std::vec::Vec::new());
            right.push(std::vec::Vec::new());
        }
        let mut epsilon = Epsilon::create(graph.typees.len());
        let mut lollipop = std::vec::Vec::new();
        let mut times = std::vec::Vec::new();
        let mut with = std::vec::Vec::new();
        let mut plus = std::vec::Vec::new();
        let mut one = std::vec::Vec::new();
        for (index, node) in graph.typees.iter().enumerate() {
            match node {
                crate::graph::TypeNode::Variable { node, is_dual, .. } => {
                    epsilon.add(
                        index,
                        if *is_dual {
                            crate::graph::get_dual(&graph.typees, *node)
                        } else {
                            *node
                        },
                    );
                }
                _ => {}
            }
        }
        for (index, node) in graph.typees.iter().enumerate() {
            match node {
                crate::graph::TypeNode::Lollipop { value, next, .. } => {
                    left[epsilon.get(*value)].push(index);
                    right[epsilon.get(*next)].push(index);
                    lollipop.push(index);
                }
                crate::graph::TypeNode::Times { value, next, .. } => {
                    left[epsilon.get(*value)].push(index);
                    right[epsilon.get(*next)].push(index);
                    times.push(index);
                }
                crate::graph::TypeNode::With { accept, deny, .. } => {
                    left[epsilon.get(*accept)].push(index);
                    right[epsilon.get(*deny)].push(index);
                    with.push(index);
                }
                crate::graph::TypeNode::Plus { accept, deny, .. } => {
                    left[epsilon.get(*accept)].push(index);
                    right[epsilon.get(*deny)].push(index);
                    plus.push(index);
                }
                crate::graph::TypeNode::One => {
                    left[graph.typees.len()].push(index);
                    right[graph.typees.len()].push(index);
                    one.push(index);
                }
                _ => {}
            }
        }
        let mut permutation = std::vec::Vec::with_capacity(graph.typees.len() + 1);
        let mut partitions = std::collections::BTreeSet::new();
        let mut next_partitions = std::collections::BTreeSet::new();
        let mut worklist = std::collections::BTreeSet::new();
        let mut last = 0;
        for list in [&lollipop, &times, &with, &plus, &one] {
            for index in list.iter() {
                permutation.push(*index);
            }
            if permutation.len() != last {
                partitions.insert((last, permutation.len()));
                worklist.insert((last, permutation.len()));
                last = permutation.len();
            }
        }
        permutation.push(graph.typees.len());
        partitions.insert((last, permutation.len()));
        worklist.insert((last, permutation.len()));
        while let Some((low, high)) = worklist.pop_first() {
            for symbol in [&left, &right] {
                let mut preimage = std::collections::HashSet::new();
                for node in permutation[low..high].iter() {
                    for previous in symbol[*node].iter() {
                        preimage.insert(epsilon.get(*previous));
                    }
                }
                next_partitions.clear();
                for (low, high) in partitions.iter() {
                    let mut left = *low;
                    let mut right = *high;
                    while left != right {
                        if preimage.contains(&permutation[left]) {
                            right -= 1;
                            permutation.swap(left, right);
                        } else {
                            left += 1;
                        }
                    }
                    if left != *low && right != *high {
                        next_partitions.insert((*low, left));
                        next_partitions.insert((right, *high));
                        if worklist.remove(&(*low, *high)) {
                            worklist.insert((*low, left));
                            worklist.insert((right, *high));
                        } else {
                            if left - *low < *high - right {
                                worklist.insert((*low, left));
                            } else {
                                worklist.insert((right, *high));
                            }
                        }
                    } else {
                        next_partitions.insert((*low, *high));
                    }
                }
                std::mem::swap(&mut partitions, &mut next_partitions);
            }
        }
        for (low, high) in partitions {
            for node in permutation[low..high].iter() {
                classs[*node] = permutation[low];
            }
        }
        for index in 0..graph.typees.len() {
            classs[index] = classs[epsilon.get(index)];
        }
        Checker {
            graph,
            typees,
            epsilon,
            classs,
        }
    }
    fn initialize_routine(&mut self, routine: &crate::graph::Routine) {
        let mut gamma = std::collections::HashMap::new();
        for crate::graph::Formal { name, r#type } in routine.formals.iter() {
            gamma.insert(name.clone(), self.epsilon.get(*r#type));
        }
        self.set_gamma(routine.start, gamma).unwrap();
    }
    fn set_gamma(
        &mut self,
        index: usize,
        gamma: std::collections::HashMap<std::string::String, usize>,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        if let Some(delta) = &self.typees[index] {
            if delta.len() > gamma.len() {
                for name in delta.keys() {
                    if !gamma.contains_key(name) {
                        return Err(std::boxed::Box::new(Error::Closed(name.clone())));
                    }
                }
            }
            for (name, gamma_type) in gamma {
                if let Some(delta_type) = delta.get(&name) {
                    if self.classs[gamma_type] != self.classs[*delta_type] {
                        return Err(std::boxed::Box::new(Error::TypeMismatch));
                    }
                } else {
                    return Err(std::boxed::Box::new(Error::NotClosed(name)));
                }
            }
        } else {
            self.typees[index] = Some(gamma);
        }
        Ok(())
    }
    fn check_expression(
        &mut self,
        gamma: &mut std::collections::HashMap<std::string::String, usize>,
        expression: &crate::graph::Expression,
    ) -> std::result::Result<usize, std::boxed::Box<dyn std::error::Error>> {
        match expression {
            crate::graph::Expression::Variable { name } => {
                if let Some(r#type) = gamma.remove(name) {
                    Ok(r#type)
                } else {
                    return Err(std::boxed::Box::new(Error::Closed(name.clone())));
                }
            }
            crate::graph::Expression::Call {
                name,
                before,
                after,
            } => {
                let formals = &self.graph.routinees.get(name).unwrap().formals;
                for (index, actual) in (0..).zip(before) {
                    let r#type = self.check_expression(gamma, actual)?;
                    if self.classs[formals[index].r#type] != self.classs[r#type] {
                        return Err(std::boxed::Box::new(Error::TypeMismatch));
                    }
                }
                for (index, actual) in (before.len() + 1..).zip(after) {
                    let r#type = self.check_expression(gamma, actual)?;
                    if self.classs[formals[index].r#type] != self.classs[r#type] {
                        return Err(std::boxed::Box::new(Error::TypeMismatch));
                    }
                }
                Ok(crate::graph::get_dual(
                    &self.graph.typees,
                    formals[before.len()].r#type,
                ))
            }
        }
    }
    fn check_node(
        &mut self,
        index: usize,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        let mut gamma = if let Some(gamma) = &self.typees[index] {
            gamma.clone()
        } else {
            return Err(std::boxed::Box::new(Error::NotInReverseTopologicalOrder));
        };
        match &self.graph.nodees[index] {
            crate::graph::Node::Branch { next } => {
                self.set_gamma(*next, gamma)?;
            }
            crate::graph::Node::Assign { name, value, next } => {
                let r#type = self.check_expression(&mut gamma, value)?;
                if gamma
                    .insert(name.clone(), self.epsilon.get(r#type))
                    .is_some()
                {
                    return Err(std::boxed::Box::new(Error::NotClosed(name.clone())));
                }
                self.set_gamma(*next, gamma)?;
            }
            crate::graph::Node::Call {
                name,
                actuals,
                next,
            } => {
                let formals = &self.graph.routinees.get(name).unwrap().formals;
                for (formal, actual) in formals.iter().zip(actuals) {
                    let r#type = self.check_expression(&mut gamma, actual)?;
                    if self.classs[formal.r#type] != self.classs[r#type] {
                        return Err(std::boxed::Box::new(Error::TypeMismatch));
                    }
                }
                self.set_gamma(*next, gamma)?;
            }
            crate::graph::Node::Receive {
                source,
                variable,
                next,
            } => {
                let r#type = if let Some(r#type) = gamma.get(source) {
                    r#type
                } else {
                    return Err(std::boxed::Box::new(Error::Closed(source.clone())));
                };
                match &self.graph.typees[self.epsilon.get(*r#type)] {
                    crate::graph::TypeNode::Times {
                        value,
                        next: next_type,
                        ..
                    } => {
                        if gamma
                            .insert(variable.clone(), self.epsilon.get(*value))
                            .is_some()
                        {
                            return Err(std::boxed::Box::new(Error::NotClosed(variable.clone())));
                        }
                        gamma.insert(source.clone(), self.epsilon.get(*next_type));
                        self.set_gamma(*next, gamma)?;
                    }
                    _ => return Err(std::boxed::Box::new(Error::NotTimes(source.clone()))),
                }
            }
            crate::graph::Node::Send {
                destination,
                variable,
                next,
            } => {
                let r#type = if let Some(r#type) = gamma.get(destination) {
                    r#type
                } else {
                    return Err(std::boxed::Box::new(Error::Closed(destination.clone())));
                };
                match &self.graph.typees[self.epsilon.get(*r#type)] {
                    crate::graph::TypeNode::Lollipop {
                        value,
                        next: next_type,
                        ..
                    } => {
                        if let Some(r#type) = gamma.remove(variable) {
                            if self.classs[r#type] != self.classs[*value] {
                                return Err(std::boxed::Box::new(Error::TypeMismatch));
                            }
                        } else {
                            return Err(std::boxed::Box::new(Error::Closed(variable.clone())));
                        }
                        gamma.insert(destination.clone(), self.epsilon.get(*next_type));
                        self.set_gamma(*next, gamma)?;
                    }
                    _ => {
                        return Err(std::boxed::Box::new(Error::NotLollipop(
                            destination.clone(),
                        )));
                    }
                }
            }
            crate::graph::Node::Offer {
                client,
                accepted,
                denied,
            } => {
                let r#type = if let Some(r#type) = gamma.get(client) {
                    r#type
                } else {
                    return Err(std::boxed::Box::new(Error::Closed(client.clone())));
                };
                match &self.graph.typees[self.epsilon.get(*r#type)] {
                    crate::graph::TypeNode::Plus { accept, deny, .. } => {
                        let mut delta = gamma.clone();
                        gamma.insert(client.clone(), self.epsilon.get(*accept));
                        delta.insert(client.clone(), self.epsilon.get(*deny));
                        self.set_gamma(*accepted, gamma)?;
                        self.set_gamma(*denied, delta)?;
                    }
                    _ => return Err(std::boxed::Box::new(Error::NotPlus(client.clone()))),
                }
            }
            crate::graph::Node::Accept { server, next } => {
                let r#type = if let Some(r#type) = gamma.get(server) {
                    r#type
                } else {
                    return Err(std::boxed::Box::new(Error::Closed(server.clone())));
                };
                match &self.graph.typees[self.epsilon.get(*r#type)] {
                    crate::graph::TypeNode::With { accept, .. } => {
                        gamma.insert(server.clone(), self.epsilon.get(*accept));
                        self.set_gamma(*next, gamma)?;
                    }
                    _ => return Err(std::boxed::Box::new(Error::NotWith(server.clone()))),
                }
            }
            crate::graph::Node::Deny { server, next } => {
                let r#type = if let Some(r#type) = gamma.get(server) {
                    r#type
                } else {
                    return Err(std::boxed::Box::new(Error::Closed(server.clone())));
                };
                match &self.graph.typees[self.epsilon.get(*r#type)] {
                    crate::graph::TypeNode::With { deny, .. } => {
                        gamma.insert(server.clone(), self.epsilon.get(*deny));
                        self.set_gamma(*next, gamma)?;
                    }
                    _ => return Err(std::boxed::Box::new(Error::NotWith(server.clone()))),
                }
            }
            crate::graph::Node::Close { name, next } => {
                let r#type = if let Some(r#type) = gamma.remove(name) {
                    r#type
                } else {
                    return Err(std::boxed::Box::new(Error::Closed(name.clone())));
                };
                if self.graph.typees[self.epsilon.get(r#type)] != crate::graph::TypeNode::One {
                    return Err(std::boxed::Box::new(Error::NotOne(name.clone())));
                }
                self.set_gamma(*next, gamma)?;
            }
            crate::graph::Node::Connect { left, right, next } => {
                let left_type = if let Some(left_type) = gamma.remove(left) {
                    left_type
                } else {
                    return Err(std::boxed::Box::new(Error::Closed(left.clone())));
                };
                let right_type = if let Some(right_type) = gamma.remove(right) {
                    right_type
                } else {
                    return Err(std::boxed::Box::new(Error::Closed(right.clone())));
                };
                if self
                    .epsilon
                    .get(crate::graph::get_dual(&self.graph.typees, left_type))
                    != self.epsilon.get(right_type)
                {
                    return Err(std::boxed::Box::new(Error::TypeMismatch));
                }
                self.set_gamma(*next, gamma)?;
            }
            crate::graph::Node::End => {
                if let Some(name) = gamma.into_keys().next() {
                    return Err(std::boxed::Box::new(Error::NotClosed(name.clone())));
                }
            }
        }
        Ok(())
    }
}
