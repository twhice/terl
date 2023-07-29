use std::{collections::HashMap, fmt::Debug};

use crate::lexer::Location;

pub struct MetaPools {
    pools: Vec<Pool>,
    this_pool: usize,
}

impl MetaPools {
    pub fn new() -> Self {
        Self {
            pools: vec![Pool::new(0)],
            this_pool: 0,
        }
    }

    pub fn apply_space_record(&mut self, record: SpaceRecord) {
        match record {
            SpaceRecord::Start => {
                self.pools.push(Pool::new(self.this_pool));
                self.this_pool = self.pools.len() - 1;
            }
            SpaceRecord::End => {
                self.this_pool = self.pools[self.this_pool].father;
            }
        }
    }
}
impl std::ops::Index<usize> for MetaPools {
    type Output = Pool;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pools[index]
    }
}

impl std::ops::IndexMut<usize> for MetaPools {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.pools[index]
    }
}

pub trait CompileRecord: Debug {
    fn effect(&self, state: &mut MetaPools);
    fn test(&self, state: &mut MetaPools);
}

pub enum SpaceRecord {
    Start,
    End,
}

pub struct Pool {
    father: usize,
    vars: HashMap<String, Records>,
}

impl Pool {
    pub fn new(father: usize) -> Self {
        Self {
            father,
            vars: HashMap::new(),
        }
    }
}

pub struct Records {
    metas: Vec<Record>,
    defines: Vec<usize>,
    uses: Vec<usize>,
    asses: Vec<usize>,
}

impl Records {
    pub fn new() -> Self {
        Self {
            defines: vec![],
            uses: vec![],
            asses: vec![],
            metas: vec![],
        }
    }

    pub fn apply(&mut self, meta: Record) -> &mut Self {
        match &meta {
            Record::Use { .. } => self.uses.push(self.metas.len()),
            Record::Define { .. } => self.defines.push(self.metas.len()),
            Record::Ass { .. } => self.asses.push(self.metas.len()),
        }
        self.metas.push(meta);
        self
    }
}

pub enum Record {
    Use { location: Location },
    Define { location: Location },
    Ass { location: Location, value: Value },
}

pub enum Value {
    Var(String),
    Num(f64),
}
