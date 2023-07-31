use std::{collections::HashMap, fmt::Debug};

use crate::{
    abi::VariableValue,
    error::{Error, ErrorKind},
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

    pub fn new_space(&mut self) -> usize {
        let space = Space::new(self.this_space);
        self.this_space = self.spaces.len();
        self.spaces.push(space);
        self.spaces.len() - 1
    }

    pub fn close_space(&mut self) -> usize {
        self.this_space = self[self.this_space].super_space;
        self.this_space
    }

    pub fn global_lookup_var(
        &mut self,
        name: &str,
        location: Location,
    ) -> Result<&VariableValue, Error> {
        let mut space = self.this_space;

        loop {
            if let Some(value) = self[space].local_lookup_var(name, location) {
                return Ok(value);
            }
            if space == 0 {
                return Err(ErrorKind::UnDefinedVar(name.to_owned()).make_error(location));
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

    pub fn global_ass_var(
        &mut self,
        name: &str,
        location: Location,
        val: VariableValue,
    ) -> Result<(), Error> {
        let mut space = self.this_space;
        let mut val = Some(val);
        loop {
            match self[space].local_ass_var(name, val.take().unwrap()) {
                Ok(..) => return Ok(()),
                Err(v) => val = Some(v),
            }
            if space == 0 {
                return Err(ErrorKind::UnDefinedVar(name.to_string()).make_error(location));
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

    pub fn local_lookup_var(&self, name: &str, location: Location) -> Option<&VariableValue> {
        let var = self.vars.get(name)?;
        var.vul.as_ref()
    }

    pub fn local_ass_var(&mut self, name: &str, val: VariableValue) -> Result<(), VariableValue> {
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
    vul: Option<VariableValue>,
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
