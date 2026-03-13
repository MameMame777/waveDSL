use crate::error::Span;

/// Top-level program: statements plus optional head/foot/config blocks.
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub head: Option<Vec<KeyValue>>,
    pub foot: Option<Vec<KeyValue>>,
    pub config: Option<Vec<KeyValue>>,
}

/// A key-value pair used in head/foot/config blocks.
#[derive(Debug, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: Value,
    pub span: Span,
}

/// A signal-level attribute (e.g. period=2, phase=0.5).
#[derive(Debug, Clone)]
pub struct SignalAttr {
    pub name: String,
    pub value: Value,
    pub span: Span,
}

/// A statement is a signal, group, const declaration, or assert block.
#[derive(Debug, Clone)]
pub enum Statement {
    Signal {
        name: String,
        sequence: Vec<WaveExpr>,
        attrs: Vec<SignalAttr>,
        span: Span,
    },
    Group {
        name: Option<String>,
        statements: Vec<Statement>,
        span: Span,
    },
    ConstDecl {
        name: String,
        value: Value,
        span: Span,
    },
    AssertBlock(AssertBlock),
}

// ─── Assert block ────────────────────────────────────────────────────────────

/// An assert block with either wave-pattern body or when/then conditions.
#[derive(Debug, Clone)]
pub struct AssertBlock {
    pub name: String,
    pub clock: String,
    pub body: AssertBody,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AssertBody {
    /// Signals whose waveforms define the expected pattern.
    Wave(Vec<AssertSignal>),
    /// Implication conditions: `when <cond> then <seq>`.
    Conditions(Vec<WhenStmt>),
}

/// A signal declaration inside a wave-body assert block.
#[derive(Debug, Clone)]
pub struct AssertSignal {
    pub name: String,
    pub sequence: Vec<WaveExpr>,
    pub span: Span,
}

/// One `when <antecedent> then <consequent>` statement.
#[derive(Debug, Clone)]
pub struct WhenStmt {
    pub antecedent: CondExpr,
    pub consequent: SeqExpr,
    pub span: Span,
}

// ─── Condition expressions ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum CondExpr {
    Compare {
        signal: String,
        op: CmpOp,
        state: StateVal,
        span: Span,
    },
    SysFunc {
        func: SysFunc,
        signal: String,
        span: Span,
    },
    And(Box<CondExpr>, Box<CondExpr>),
    Or(Box<CondExpr>, Box<CondExpr>),
}

// ─── Sequence expressions ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SeqExpr {
    /// `##N <expr>`
    Delay {
        cycles: u64,
        expr: Box<SeqExpr>,
        span: Span,
    },
    /// A condition used as a one-cycle sequence step.
    Cond(CondExpr),
    /// `<expr>[*N]` — consecutive repetition.
    RepeatConsec {
        expr: Box<SeqExpr>,
        count: u64,
        span: Span,
    },
    /// `<expr>[->N]` — goto repetition.
    RepeatGoto {
        expr: Box<SeqExpr>,
        count: u64,
        span: Span,
    },
    And(Box<SeqExpr>, Box<SeqExpr>),
    Or(Box<SeqExpr>, Box<SeqExpr>),
}

// ─── Enumerations ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CmpOp {
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateVal {
    High,
    Low,
    Data,
    X,
    Z,
}

#[derive(Debug, Clone)]
pub enum SysFunc {
    Rose,
    Fell,
    Stable,
}

/// A wave expression: a built-in function call or repeat.
#[derive(Debug, Clone)]
pub enum WaveExpr {
    Call {
        name: String,
        args: Vec<Arg>,
        span: Span,
    },
    Repeat {
        count: u64,
        sequence: Vec<WaveExpr>,
        span: Span,
    },
}

/// Function argument: positional or keyword.
#[derive(Debug, Clone)]
pub enum Arg {
    Positional(Value, Span),
    Keyword(String, Value, Span),
}

/// Argument value.
#[derive(Debug, Clone)]
pub enum Value {
    Number(u64),
    Float(f64),
    Str(String),
    Enum(String),
    VarRef(String),
}
