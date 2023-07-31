/*
* IO:
 * Read
 * Write
 * Draw
 * Print

* CONTROL:
 * DrawFlush
 * PrintFlush
 * GetLink
 * Control
 * Radar
 * Sensor

* OPTION:
 * Set
 * Operation
 * Lookup
 * PackColor

* CONTROLFLOW:
 * Wait
 * Stop
 * End
 * Jump

* UNITCONTROL:
 * UnitBInd
 * UnitControl
 * UnitRadar
 * UnitLocate
*/

use crate::lexer::Symbol;

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: VariableName,
    pub value: VariableValue,
}

impl Variable {
    pub fn new(name: VariableName, value: VariableValue) -> Self {
        Self { name, value }
    }

    /// 序号命名，最终都会被优化掉
    pub fn alloc() -> Self {
        Self {
            name: VariableName::index(),
            value: VariableValue::UnknowType,
        }
    }

    /// 零，或者说zero/null，不应该作为左值出现
    pub fn zero() -> Self {
        Self {
            name: VariableName::None,
            value: VariableValue::Number(0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum VariableName {
    Named(String),
    Index(usize),
    None,
}

impl VariableName {
    pub fn index() -> Self {
        use std::sync::atomic;
        static VAR_INDEX: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
        Self::Index(VAR_INDEX.fetch_add(1, atomic::Ordering::Relaxed))
    }

    pub fn named(name: &str) -> Self {
        Self::Named(name.to_string())
    }

    pub fn as_named(&self) -> Option<&String> {
        if let Self::Named(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_index(&self) -> Option<&usize> {
        if let Self::Index(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the variable name is [`Named`].
    ///
    /// [`Named`]: VariableName::Named
    #[must_use]
    pub fn is_named(&self) -> bool {
        matches!(self, Self::Named(..))
    }

    /// Returns `true` if the variable name is [`Index`].
    ///
    /// [`Index`]: VariableName::Index
    #[must_use]
    pub fn is_index(&self) -> bool {
        matches!(self, Self::Index(..))
    }

    /// Returns `true` if the variable name is [`None`].
    ///
    /// [`None`]: VariableName::None
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Debug, Clone)]
pub enum VariableValue {
    Number(f64),
    String(String),
    // UnknowString,
    Variable(String),
    FnReturn(String),
    UnknowType,
    MetaAttrib,
}

impl VariableValue {
    /// Returns `true` if the variable value is [`Number`].
    ///
    /// [`Number`]: VariableValue::Number
    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(..))
    }

    /// Returns `true` if the variable value is [`String`].
    ///
    /// [`String`]: VariableValue::String
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }

    /// Returns `true` if the variable value is [`MetaAttrib`].
    ///
    /// [`MetaAttrib`]: VariableValue::MetaAttrib
    #[must_use]
    pub fn is_meta_attrib(&self) -> bool {
        matches!(self, Self::MetaAttrib)
    }

    pub fn as_number(&self) -> Option<&f64> {
        if let Self::Number(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the variable value is [`Variable`].
    ///
    /// [`Variable`]: VariableValue::Variable
    #[must_use]
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(..))
    }

    pub fn as_variable(&self) -> Option<&String> {
        if let Self::Variable(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the variable value is [`FnReturn`].
    ///
    /// [`FnReturn`]: VariableValue::FnReturn
    #[must_use]
    pub fn is_fn_return(&self) -> bool {
        matches!(self, Self::FnReturn(..))
    }

    pub fn as_fn_return(&self) -> Option<&String> {
        if let Self::FnReturn(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the variable value is [`UnknowType`].
    ///
    /// [`UnknowType`]: VariableValue::UnknowType
    #[must_use]
    pub fn is_unknow_type(&self) -> bool {
        matches!(self, Self::UnknowType)
    }
}

/// 逻辑语句
#[derive(Debug, Clone)]
pub enum Statement {
    Set {
        result: Variable,
        value: Variable,
    },
    Operation {
        result: Variable,
        op: Symbol,
        v1: Variable,
        v2: Variable, //可能为空
    },

    Jump {
        left: Variable,
        cond: JumpCondition,
        right: Variable,
        target: usize,
    },
}

impl Statement {
    pub fn jump_target(&self) -> Option<usize> {
        if let Self::Jump { target, .. } = self {
            Some(*target)
        } else {
            None
        }
    }

    pub fn reset(&mut self, new_target: usize) {
        if let Self::Jump { target, .. } = self {
            *target = new_target
        }
    }

    pub fn set(result: Variable, value: Variable) -> Self {
        Self::Set { result, value }
    }

    pub fn operation(result: Variable, op: Symbol, v1: Variable, v2: Variable) -> Self {
        Self::Operation { result, op, v1, v2 }
    }

    pub fn jump(left: Variable, cond: JumpCondition, right: Variable, target: usize) -> Self {
        Self::Jump {
            left,
            cond,
            right,
            target,
        }
    }
}

#[derive(Debug, Clone)]
pub enum JumpCondition {
    Eq,
    Neq,
    Lr,
    LrE,
    Gr,
    GrE,
    Seq,
    Always,
}

impl TryFrom<Symbol> for JumpCondition {
    type Error = ();

    fn try_from(value: Symbol) -> Result<Self, Self::Error> {
        let cond = match value {
            Symbol::Eq => Self::Eq,
            Symbol::Neq => Self::Neq,
            Symbol::Gr => Self::Gr,
            Symbol::GrE => Self::GrE,
            Symbol::Lr => Self::Lr,
            Symbol::LrE => Self::LrE,
            Symbol::Seq => Self::Seq,
            _ => return Err(()),
        };
        Ok(cond)
    }
}

// pub enum VariableType {
//     Number,
//     String,
//     Color, // pack_color(r,g,b,a) -> Color
//     Meta,
// }

// pub enum LookUp {
//     UnitCount,
//     ItemCount,
//     BlockCount,
//     LiquidCount,
// }

// pub enum Radar {
//     Any,
//     Enemy,
//     Ally,
//     Player,
//     Attacker,
//     Flying,
//     Boss,
//     Ground,
// }
