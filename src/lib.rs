use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::iter::FromIterator;
use std::sync::Arc;

#[cfg(test)]
extern crate rand;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
mod tests;

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

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V> where K: Borrow<Q>, Q: Ord {
        self.get_by(|node_key| key.cmp(node_key.borrow())).map(|(_, v)| v)
    }

    pub fn get_by<F>(&self, compare: F) -> Option<(&K, &V)>
                     where F: for<'a> FnMut(&'a K) -> Ordering {
        self.root.get_by(compare)
    }

    pub fn insert(&self, key: K, value: V) -> Tree<K, V> {
        Tree {
            root: self.root.insert(key, value),
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        Iter {
            start: self.root.to_option(),
            stack: vec![],
        }
    }
}

impl<K, V> Link<K, V> where K: Clone + PartialOrd + Ord, V: Clone {
    fn to_option(&self) -> Option<&Arc<Node<K, V>>> {
        match *self {
            Link::Empty => None,
            Link::Node(ref node) => Some(node),
        }
    }

    fn get_by<F>(&self, mut compare: F) -> Option<(&K, &V)>
                 where F: for<'a> FnMut(&'a K) -> Ordering {
        match *self {
            Link::Empty => None,
            Link::Node(ref node) => {
                match compare(&node.key) {
                    Ordering::Less => node.left.get_by(compare),
                    Ordering::Greater => node.right.get_by(compare),
                    Ordering::Equal => Some((&node.key, &node.value)),
                }
            }
        }
    }

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

pub struct Iter<'a, K, V> where K: 'a, V: 'a {
    start: Option<&'a Arc<Node<K, V>>>,
    stack: Vec<&'a Arc<Node<K, V>>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> where K: Clone + PartialOrd + Ord, V: Clone {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        let mut current = self.start.take();
        while !self.stack.is_empty() || current.is_some() {
            match current {
                Some(node) => {
                    self.stack.push(node);
                    current = node.left.to_option()
                }
                None => {
                    let node = self.stack.pop().unwrap();
                    let item = (&node.key, &node.value);
                    self.start = node.right.to_option();
                    return Some(item)
                }
            }
        }
        None
    }
}

impl<K, V> FromIterator<(K, V)> for Tree<K, V> where K: Clone + PartialOrd + Ord, V: Clone {
    fn from_iter<T>(iter: T) -> Tree<K, V> where T: IntoIterator<Item = (K, V)> {
        let mut tree = Tree::new();
        for (key, value) in iter {
            tree = tree.insert(key, value)
        }
        tree
    }
}

impl<K, V> Debug for Tree<K, V> where K: Clone + PartialOrd + Ord + Debug, V: Clone + Debug {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        try!(write!(formatter, "["));
        let mut iter = self.iter();
        if let Some(pair) = iter.next() {
            try!(pair.fmt(formatter));
            for pair in iter {
                try!(write!(formatter, ",{:?}", pair))
            }
        }
        write!(formatter, "]")
    }
}

