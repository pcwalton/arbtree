use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Clone)]
pub struct Tree<K, V> {
    root: Link<K, V>,
}

#[derive(Clone, Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    left: Link<K, V>,
    right: Link<K, V>,
    color: Color,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Color {
    Red,
    Black,
}

#[derive(Clone, Debug)]
enum Link<K, V> {
    Empty,
    Node(Arc<Node<K, V>>),
}

impl<K, V> Tree<K, V> where K: Clone + PartialOrd + Ord, V: Clone {
    pub fn new() -> Tree<K, V> {
        Tree {
            root: Link::Empty,
        }
    }

    pub fn insert(&self, key: K, value: V) -> Tree<K, V> {
        Tree {
            root: self.root.insert(key, value),
        }
    }
}

impl<K, V> Link<K, V> where K: Clone + PartialOrd + Ord, V: Clone {
    fn insert(&self, key: K, value: V) -> Link<K, V> {
        match *self {
            Link::Empty => {
                Link::Node(Arc::new(Node {
                    key: key,
                    value: value,
                    left: Link::Empty,
                    right: Link::Empty,
                    color: Color::Red,
                }))
            }
            Link::Node(ref node) => {
                match key.cmp(&node.key) {
                    Ordering::Less => {
                        node.color.balance(node.key.clone(),
                                           node.value.clone(),
                                           node.left.insert(key, value),
                                           node.right.clone())
                    }
                    Ordering::Greater => {
                        node.color.balance(node.key.clone(),
                                           node.value.clone(),
                                           node.left.clone(),
                                           node.right.insert(key, value))
                    }
                    Ordering::Equal => {
                        Link::Node(Arc::new(Node {
                            key: key,
                            value: value,
                            left: node.left.clone(),
                            right: node.right.clone(),
                            color: node.color,
                        }))
                    }
                }
            }
        }
    }

    fn get_if_red(&self) -> Option<&Arc<Node<K, V>>> {
        match *self {
            Link::Node(ref node) if node.color.is_red() => Some(node),
            Link::Node(_) | Link::Empty => None,
        }
    }

    fn rearrangement(parent_key: K,
                     parent_value: V,
                     left_key: K,
                     left_value: V,
                     left_left: Link<K, V>,
                     left_right: Link<K, V>,
                     right_key: K,
                     right_value: V,
                     right_left: Link<K, V>,
                     right_right: Link<K, V>)
                     -> Link<K, V> {
        Link::Node(Arc::new(Node {
            key: parent_key,
            value: parent_value,
            left: Link::Node(Arc::new(Node {
                key: left_key,
                value: left_value,
                left: left_left,
                right: left_right,
                color: Color::Black,
            })),
            right: Link::Node(Arc::new(Node {
                key: right_key,
                value: right_value,
                left: right_left,
                right: right_right,
                color: Color::Black,
            })),
            color: Color::Red,
        }))
    }
}

impl Color {
    fn balance<K, V>(self, key: K, value: V, left: Link<K, V>, right: Link<K, V>) -> Link<K, V>
                     where K: Clone + PartialOrd + Ord, V: Clone {
        if self.is_black() {
            return Link::Node(Arc::new(Node {
                key: key,
                value: value,
                left: left,
                right: right,
                color: self,
            }))
        }

        if let Some(ref left) = left.get_if_red() {
            if let Some(ref left_left) = left.left.get_if_red() {
                return Link::rearrangement(left.key.clone(),
                                           left.value.clone(),
                                           left_left.key.clone(),
                                           left_left.value.clone(),
                                           left_left.left.clone(),
                                           left_left.right.clone(),
                                           key,
                                           value,
                                           left.right.clone(),
                                           right)
            }
            if let Some(ref left_right) = left.right.get_if_red() {
                return Link::rearrangement(left_right.key.clone(),
                                           left_right.value.clone(),
                                           left.key.clone(),
                                           left.value.clone(),
                                           left.left.clone(),
                                           left_right.left.clone(),
                                           key,
                                           value,
                                           left_right.right.clone(),
                                           right)
            }
        }
        if let Some(ref right) = right.get_if_red() {
            if let Some(ref right_left) = right.left.get_if_red() {
                return Link::rearrangement(right_left.key.clone(),
                                           right_left.value.clone(),
                                           key,
                                           value,
                                           left,
                                           right_left.left.clone(),
                                           right.key.clone(),
                                           right.value.clone(),
                                           right_left.right.clone(),
                                           right.right.clone())
            }
            if let Some(ref right_right) = right.right.get_if_red() {
                return Link::rearrangement(right.key.clone(),
                                           right.value.clone(),
                                           key,
                                           value,
                                           left,
                                           right.left.clone(),
                                           right_right.key.clone(),
                                           right_right.value.clone(),
                                           right_right.left.clone(),
                                           right_right.right.clone())
            }
        }

        Link::Node(Arc::new(Node {
            key: key,
            value: value,
            left: left,
            right: right,
            color: self,
        }))
    }

    fn is_red(&self) -> bool {
        *self == Color::Red
    }

    fn is_black(&self) -> bool {
        *self == Color::Black
    }
}

struct Iter<'a, K, V> {
    /*
    phase: IterPhase<'a, K, V>,
    stack: Vec<IterStep<'a, K, V>>,
    */
    start: Option<&'a Arc<Node<K, V>>>,
    stack: Vec<&'a Arc<Node<K, V>>>,
}

/*
enum IterPhase<'a, K, V> {
    Start(&'a Arc<Node<K, V>>),
    Emit(&'a Arc<Node<K, V>>),
    Stop,
}

enum IterStep<'a, K, V> {
    Left(&'a Arc<Node<K, V>>),
    Right(&'a Arc<Node<K, V>>),
}*/

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        let mut node = self.start.take();
        while !self.stack.is_empty() || node.is_some() {
            match node {
                Some(node) => {
                    self.stack.push(node);
                    node = node.left
                }
                None => {
                    node = self.stack.pop();
                    let item = (&node.key, &node.value);
                    self.start = node.right.as_ref();
                    return Some(item)
                }
            }
        }
        None

        /*
        match self.phase {
            IterPhase::Start(mut node) => {
                loop {
                    match node.left {
                        Link::Node(left) => {
                            self.stack.push(IterStep::Left(node));
                            node = left
                        }
                        Link::Empty => {
                            self.phase = IterPhase::Emit(node);
                            return Some(node)
                        }
                    }
                }
            }
            IterPhase::Emit(mut node) => {
                loop {
                    match node.right {
                        Link::Node(right) => {
                            self.stack.push(IterStep::Right(node));
                            node = right;
                            break
                        }
                        Link::Empty => {
                            loop {
                                match self.stack.pop() {
                                    Some(IterStep::Left(node)) => {
                                        self.phase = IterPhase::Emit(node);
                                        return Some(node)
                                    }
                                    Some(IterStep::Right(node)) => {}
                                    None => {
                                        self.phase = IterPhase::Stop;
                                        return None
                                    }
                                }
                            }
                        }
                    }
                    match node.left {
                        None => {
                            self.phase = IterPhase::Emit(node);
                            return Some(node)
                        }
                        Some(left) => {

                        }
                    }
                }
            }
        }
        let next = self.next.take() {
            None => return None,
            Some(next) => next,
        };

        loop {
            let step = match self.stack.pop() {
                None => return None,
                Some(step) => step,
            };
            match *step {
                IterStep::Left(left) => {
                    match left.right {
                        None => continue,
                        Some(ref right) => {
                            self.stack.push(right);

                        }
                    }
                }
            }
        }*/
    }
}

