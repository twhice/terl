use std::{collections::HashMap, fmt::Debug};

use crate::{
    ast,
    error::{Error, ErrorKind, Warn},
    lexer::Location,
};

#[derive(Debug, Clone)]
pub struct GlobalSpace {
    spaces: Vec<Space>,
    mapping: HashMap<usize, usize>,
    this_space: usize,
}

impl GlobalSpace {
    pub fn new() -> Self {
        Self {
            spaces: vec![Space::new(0)],
            this_space: 0,
            mapping: HashMap::new(),
        }
    }

    pub fn new_sapce(&mut self) -> usize {
        let space = Space::new(self.this_space);
        self.this_space = self.spaces.len();
        self.spaces.push(space);
        self.spaces.len() - 1
    }

    pub fn close_space(&mut self) -> usize {
        self.this_space = self[self.this_space].super_space;
        self.this_space
    }

    pub fn global_get_var(&mut self, name: &str) -> Option<(&Value, Location)> {
        let mut space = self.this_space;

        loop {
            if let Some(vaule) = self[space].local_get_var(name) {
                return Some(vaule);
            } else if space == 0 {
                return None;
            } else {
                space = self[space].super_space
            }
        }
    }

    pub fn global_use_var(&mut self, name: &str, location: Location) -> Option<()> {
        let mut space = self.this_space;

        loop {
            if self[space].local_use_var(name, location).is_some() {
                return Some(());
            } else if space == 0 {
                return None;
            } else {
                space = self[space].super_space
            }
        }
    }

    pub fn global_ass_var(&mut self, name: &str, val: Value) -> Result<(), Value> {
        let mut space = self.this_space;
        let mut val = Some(val);
        loop {
            match self[space].local_ass_var(name, val.take().unwrap()) {
                Ok(..) => return Ok(()),
                Err(v) => val = Some(v),
            }
            if space == 0 {
                return Err(val.take().unwrap());
            } else {
                space = self[space].super_space
            }
        }
    }

    pub fn global_use_fn(
        &mut self,
        name: &str,
        location: Location,
        nr_args: usize,
    ) -> Result<(), Error> {
        let mut space = self.this_space;

        loop {
            match self[space].local_use_fn(name, location, nr_args) {
                Some(result) => return result,
                None => {
                    if space == 0 {
                        return Err(
                            ErrorKind::CallUnDefinedFn(name.to_owned()).make_error(location)
                        );
                    } else {
                        space = self[space].super_space;
                    }
                }
            };
        }
    }

    pub fn in_space<F, R>(&mut self, index: usize, active: F) -> Option<R>
    where
        F: FnOnce(&mut Self) -> R,
    {
        if index >= self.spaces.len() {
            return None;
        }
        let backup = self.this_space;
        self.this_space = index;
        let r = active(self);
        self.this_space = backup;
        Some(r)
    }
}

impl std::ops::Index<usize> for GlobalSpace {
    type Output = Space;

    fn index(&self, index: usize) -> &Self::Output {
        &self.spaces[index]
    }
}

impl std::ops::IndexMut<usize> for GlobalSpace {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.spaces[index]
    }
}

impl std::ops::Index<&ast::Block<'_>> for GlobalSpace {
    type Output = Space;

    fn index(&self, index: &ast::Block) -> &Self::Output {
        &self[self.mapping.get(&index.index).copied().unwrap()]
    }
}

impl std::ops::IndexMut<&ast::Block<'_>> for GlobalSpace {
    fn index_mut(&mut self, index: &ast::Block) -> &mut Self::Output {
        let index = self.mapping.get(&index.index).copied().unwrap();
        &mut self[index]
    }
}

impl std::ops::Deref for GlobalSpace {
    type Target = Space;

    fn deref(&self) -> &Self::Target {
        &self.spaces[self.this_space]
    }
}

impl std::ops::DerefMut for GlobalSpace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let this = self.this_space;
        &mut self[this]
    }
}

#[derive(Debug, Clone)]
pub struct Space {
    super_space: usize,
    vars: HashMap<String, VarRecords>,
    fns: HashMap<String, FnRecords>,
}

impl Space {
    pub fn new(super_space: usize) -> Self {
        Self {
            super_space,
            vars: HashMap::new(),
            fns: HashMap::new(),
        }
    }

    pub fn local_get_var(&self, name: &str) -> Option<(&Value, Location)> {
        let var = self.vars.get(name)?;
        Some((var.vul.as_ref()?, *var.defines.last()?))
    }

    pub fn local_ass_var(&mut self, name: &str, val: Value) -> Result<(), Value> {
        match self.vars.get_mut(name) {
            Some(v) => {
                v.vul = Some(val);
                Ok(())
            }
            None => Err(val),
        }
    }

    fn local_use_var(&mut self, name: &str, location: Location) -> Option<()> {
        self.vars.get_mut(name)?.uses.push(location);
        Some(())
    }

    pub fn define_var(&mut self, name: &str, location: Location) {
        self.vars
            .entry(name.to_owned())
            .or_insert_with(VarRecords::new)
            .defines
            .push(location);
    }

    pub fn local_use_fn(
        &mut self,
        name: &str,
        location: Location,
        nr_args: usize,
    ) -> Option<Result<(), Error>> {
        let fn_record = self.fns.get_mut(name)?;
        if fn_record.define.nr_args != nr_args {
            Some(Err(ErrorKind::CallFnWithIncorrectArgs(
                fn_record.define.location,
                fn_record.define.nr_args,
                nr_args,
            )
            .make_error(location)))
        } else {
            fn_record.uses.push(location);
            Some(Ok(()))
        }
    }

    pub fn define_fn(
        &mut self,
        name: &str,
        location: Location,
        nr_args: usize,
    ) -> Result<(), Error> {
        let entity = self.fns.get(name);
        match &entity {
            Some(record) => {
                return Err(ErrorKind::DoubleFnDefine(record.define.location).make_error(location))
            }
            None => self.fns.insert(
                name.to_owned(),
                FnRecords::new(FnDefine { nr_args, location }),
            ),
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VarRecords {
    pub defines: Vec<Location>,
    pub uses: Vec<Location>,
    // r#type : abi::Type
    vul: Option<Value>,
}

impl VarRecords {
    pub fn new() -> Self {
        Self {
            defines: vec![],
            uses: vec![],
            vul: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Var(String),
    Number(f64),
    String(String),
    Expr,
}

#[derive(Debug, Clone)]
pub struct FnRecords {
    define: FnDefine,
    uses: Vec<Location>,
}

#[derive(Debug, Clone)]
pub struct FnDefine {
    nr_args: usize,
    location: Location,
}

impl FnRecords {
    pub fn new(define: FnDefine) -> Self {
        Self {
            define,
            uses: vec![],
        }
    }
}

pub trait Record: Debug + 'static {
    fn effect(&mut self, global: &mut GlobalSpace) -> Result<(), Error>;
    fn test(&mut self, global: &mut GlobalSpace) -> Result<Warn, Error>;
}

pub mod records {
    use crate::error::ErrorKind;

    use super::*;
    #[derive(Debug, Clone, Copy)]
    pub enum SpaceAllocRrcord {
        Start(usize),
        End,
    }

    impl Record for SpaceAllocRrcord {
        fn effect(&mut self, global: &mut GlobalSpace) -> Result<(), Error> {
            match self {
                SpaceAllocRrcord::Start(space_index) => {
                    let index_in_global = global.new_sapce();
                    global.mapping.insert(*space_index, index_in_global);
                }
                SpaceAllocRrcord::End => {
                    global.close_space();
                }
            };
            Ok(())
        }

        fn test(&mut self, _global: &mut GlobalSpace) -> Result<Warn, Error> {
            Ok(Warn::empty())
        }
    }

    #[derive(Debug, Clone)]
    pub enum ValueRecord {
        Use(String, Location),
        Def(String, Location),
        Ass(String, Location, Value),
    }

    impl Record for ValueRecord {
        fn effect(&mut self, global: &mut GlobalSpace) -> Result<(), Error> {
            // 遵循 先定义再使用的规则
            match self {
                ValueRecord::Use(name, loc) => {
                    global
                        .global_get_var(name)
                        .map(|_| ())
                        .ok_or_else(|| ErrorKind::UnDefinedVar(name.to_owned()).make_error(*loc))?;
                    global.global_use_var(name, *loc);
                    Ok(())
                }
                ValueRecord::Def(name, loc) => {
                    global.define_var(name, *loc);
                    Ok(())
                }
                ValueRecord::Ass(name, loc, val) => global
                    .global_ass_var(name, val.clone())
                    .map_err(|_| ErrorKind::UnDefinedVar(name.clone()).make_error(*loc)),
            }
        }

        fn test(&mut self, _global: &mut GlobalSpace) -> Result<Warn, Error> {
            Ok(Warn::empty())
        }
    }

    #[derive(Debug, Clone)]
    pub enum FnRecord {
        /// 函数名 位置 参数个数
        Define(String, Location, usize),
        /// 函数名 位置 参数个数 空间位置
        Call(String, Location, usize, usize),
    }

    impl FnRecord {
        pub fn define(name: &str, location: Location, nr_args: usize) -> Self {
            Self::Define(name.to_owned(), location, nr_args)
        }

        pub fn call(name: &str, location: Location, nr_args: usize) -> Self {
            Self::Call(name.to_owned(), location, nr_args, 0)
        }
    }

    impl Record for FnRecord {
        fn effect(&mut self, global: &mut GlobalSpace) -> Result<(), Error> {
            match self {
                FnRecord::Define(name, location, nr_args) => {
                    global.define_fn(name, *location, *nr_args)?
                }
                FnRecord::Call(_, _, _, space_idx) => *space_idx = global.this_space,
            }

            Ok(())
        }

        fn test(&mut self, global: &mut GlobalSpace) -> Result<Warn, Error> {
            if let Self::Call(name, location, nr_args, sapce_idx) = self {
                global
                    .in_space(*sapce_idx, |global| {
                        global.global_use_fn(name, *location, *nr_args)
                    })
                    .unwrap()?;
            }
            Ok(Warn::empty())
        }
    }
}
