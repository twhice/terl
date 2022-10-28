use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum Tree<T>
where
    T: BeTree + Debug + Clone,
{
    Node(Vec<Self>),
    Dot(Box<T>),
}
impl<T> Tree<T>
where
    T: BeTree + Debug + Clone,
{
    pub fn push(&mut self, e: T) {
        let new = Self::Dot(Box::new(e));
        match self {
            Tree::Node(_vec) => _vec.push(new),
            Tree::Dot(_) => *self = Self::Node(vec![self.clone(), new]),
        }
    }
}

pub trait BeTree
where
    Self: Sized + Debug + Clone,
{
    // type Self = dyn Tree;
    fn deep(&self) -> usize;
    fn is_left_part(&self) -> bool;
    fn is_right_part(&self) -> bool;
    fn build_deep_tree(ss: Vec<Self>, start_index: usize) -> Tree<Self> {
        if !ss.is_empty() {
            let last_deep: usize = ss[0].deep();
            let mut deep: usize;
            let mut ret: Vec<Tree<Self>> = Vec::new();
            let mut wait = false;
            for index in start_index..ss.len() {
                deep = ss[index].deep();
                if deep > last_deep && !wait {
                    wait = !wait;
                    ret.push(Self::build_deep_tree(ss.clone(), index));
                } else if deep < last_deep {
                    return Tree::Node(ret);
                } else {
                    wait = false;
                    ret.push(Tree::Dot(Box::new(ss[index].clone())))
                }
            }
            return Tree::Node(ret);
        } else {
            todo!()
        }
    }
    fn build_node_tree(ss: Vec<Self>, deep: usize) -> Tree<Self> {
        let mut deep_now = deep;
        let mut wait = false;
        let mut ret = Vec::new();
        for s in &ss {
            if s.is_left_part() {
                deep_now += 1;
                continue;
            } else if s.is_right_part() {
                deep_now -= 1;
                continue;
            }
            if deep < deep_now && !wait {
                ret.push(Self::build_node_tree(ss.clone(), deep_now));
                wait = true;
            } else {
                wait = false;
                if deep == deep_now {
                    ret.push(Tree::Dot(Box::new(s.clone())));
                } else if deep > deep_now {
                    return Tree::Node(ret);
                }
            }
        }

        return Tree::Node(ret);
    }
}
