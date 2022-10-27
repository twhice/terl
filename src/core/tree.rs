use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum Tree<T>
where
    T: BeTree + Debug + Clone,
{
    Node(Vec<Self>),
    Dot(Box<T>),
}

pub trait BeTree
where
    Self: Sized + Debug + Clone,
{
    // type Self = dyn Tree;
    fn deep(&self) -> usize;
    fn build_tree(ss: Vec<Self>, start_index: usize) -> Tree<Self> {
        if !ss.is_empty() {
            let last_deep: usize = ss[0].deep();
            let mut deep: usize;
            let mut ret: Vec<Tree<Self>> = Vec::new();
            let mut wait = false;
            for index in start_index..ss.len() {
                deep = ss[index].deep();
                if deep > last_deep && !wait {
                    wait = !wait;
                    ret.push(Self::build_tree(ss.clone(), index));
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
}
pub enum Tower<T>
where
    T: Sized + Debug + Clone,
{
    Node(Vec<Self>),
    Dot(Box<T>),
}
pub trait BeTower
where
    Self: Sized + Debug + Clone,
{
    fn is_left_part(&self) -> bool;
    fn is_right_part(&self) -> bool;
    fn build_tower(ss: Vec<Self>, deep: usize) -> Tower<Self> {
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
                ret.push(Self::build_tower(ss.clone(), deep_now));
                wait = true;
            } else {
                wait = false;
                if deep == deep_now {
                    ret.push(Tower::Dot(Box::new(s.clone())));
                } else if deep > deep_now {
                    return Tower::Node(ret);
                }
            }
        }

        return Tower::Node(ret);
    }
}
