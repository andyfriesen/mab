use std::borrow::Cow;
use tokenizer::StringLiteral;
use std::sync::atomic::{AtomicUsize, Ordering};

pub type Id = usize;

static mut NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub fn gen_id() -> Id {
    unsafe {
        NEXT_ID.fetch_add(1, Ordering::SeqCst)
    }
}

trait HasId {
    fn id(&self) -> Id;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOpKind {
    Negate, // -
    BooleanNot, // not
    Length, // #
}

impl UnaryOpKind {
    pub fn precedence(&self) -> u8 {
        11
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BinaryOpKind {
    Add, // +
    Subtract, // -
    Multiply, // *
    Divide, // /
    Exponent, // ^
    Concat, // ..
}

impl BinaryOpKind {
    // From the 5.3 manual, ranked lowest to highest:
    //  1 or
    //  2 and
    //  3 <     >     <=    >=    ~=    ==
    //  4 |
    //  5 ~
    //  6 &
    //  7 <<    >>
    //  8 ..
    //  9 +     -
    // 10 *     /     //    %
    // 11 unary operators (not   #     -     ~)
    // 12 ^
    pub fn precedence(&self) -> u8 {
        match *self {
            BinaryOpKind::Concat => 8,
            BinaryOpKind::Add | BinaryOpKind::Subtract => 9,
            BinaryOpKind::Multiply | BinaryOpKind::Divide => 10,
            BinaryOpKind::Exponent => 12,
        }
    }

    pub fn is_right_associative(&self) -> bool {
        match *self {
            BinaryOpKind::Exponent | BinaryOpKind::Concat => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryOp<'a> {
    pub operator: UnaryOpKind,
    #[serde(borrow)]
    pub argument: Box<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryOp<'a> {
    pub operator: BinaryOpKind,
    #[serde(borrow)]
    pub left: Box<Expression<'a>>,
    pub right: Box<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall<'a> {
    #[serde(borrow)]
    pub name_expression: Box<Expression<'a>>,
    pub arguments: Vec<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment<'a> {
    #[serde(borrow)]
    pub names: Vec<Cow<'a, str>>,
    pub values: Vec<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalAssignment<'a> {
    #[serde(borrow)]
    pub names: Vec<Cow<'a, str>>,
    pub values: Vec<Expression<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumericFor<'a> {
    #[serde(borrow)]
    pub var: Cow<'a, str>,
    pub start: Expression<'a>,
    pub end: Expression<'a>,
    pub step: Option<Expression<'a>>,
    pub body: Chunk<'a>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericFor<'a> {
    #[serde(borrow)]
    pub vars: Vec<Cow<'a, str>>,
    pub item_source: Vec<Expression<'a>>,
    pub body: Chunk<'a>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStatement<'a> {
    #[serde(borrow)]
    pub condition: Expression<'a>,
    pub body: Chunk<'a>,
    pub else_if_branches: Vec<(Expression<'a>, Chunk<'a>)>,
    pub else_branch: Option<Chunk<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhileLoop<'a> {
    #[serde(borrow)]
    pub condition: Expression<'a>,
    pub body: Chunk<'a>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepeatLoop<'a> {
    #[serde(borrow)]
    pub condition: Expression<'a>,
    pub body: Chunk<'a>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDeclaration<'a> {
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    pub body: Chunk<'a>,
    pub parameters: Vec<Cow<'a, str>>,
    pub local: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression<'a> {
    Nil{id: Id},
    Bool{id: Id, value: bool},
    Number{id: Id, #[serde(borrow)] value: Cow<'a, str>},
    String{id: Id, value: StringLiteral<'a>},
    VarArg{id: Id},
    Table{id: Id, value: TableLiteral<'a>},
    FunctionCall{id: Id, value: FunctionCall<'a>},
    Name{id: Id, value: Cow<'a, str>},
    ParenExpression{id: Id, value: Box<Expression<'a>>},
    UnaryOp{id: Id, value: UnaryOp<'a>},
    BinaryOp{id: Id, value: BinaryOp<'a>},
}

impl <'a> HasId for Expression<'a> {
    fn id(&self) -> Id {
        *match self {
            Expression::Nil{id} => id,
            Expression::Bool{id, ..} => id,
            Expression::Number{id, ..} => id,
            Expression::String{id, ..} => id,
            Expression::VarArg{id} => id,
            Expression::Table{id, ..} => id,
            Expression::FunctionCall{id, ..} => id,
            Expression::Name{id, ..} => id,
            Expression::ParenExpression{id, ..} => id,
            Expression::UnaryOp{id, ..} => id,
            Expression::BinaryOp{id, ..} => id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TableKey<'a> {
    #[serde(borrow)]
    // '[' expression ']'
    Expression(Expression<'a>),

    // identifier
    Name(Cow<'a, str>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableLiteral<'a> {
    #[serde(borrow)]
    pub items: Vec<(Option<TableKey<'a>>, Expression<'a>)>,
}

// stat ::=  ‘;’ |
//     varlist ‘=’ explist |
//     functioncall |
//     label |
//     break |
//     goto Name |
//     do block end |
//     while exp do block end |
//     repeat block until exp |
//     if exp then block {elseif exp then block} [else block] end |
//     for Name ‘=’ exp ‘,’ exp [‘,’ exp] do block end |
//     for namelist in explist do block end |
//     function funcname funcbody |
//     local function Name funcbody |
//     local namelist [‘=’ explist]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement<'a> {
    // #[serde(borrow)]
    Assignment{id: Id, #[serde(borrow)] value: Assignment<'a>},
    LocalAssignment{id: Id, value: LocalAssignment<'a>},
    FunctionCall{id: Id, value: FunctionCall<'a>},
    NumericFor{id: Id, value: NumericFor<'a>},
    GenericFor{id: Id, value: GenericFor<'a>},
    IfStatement{id: Id, value: IfStatement<'a>},
    WhileLoop{id: Id, value: WhileLoop<'a>},
    RepeatLoop{id: Id, value: RepeatLoop<'a>},
    FunctionDeclaration{id: Id, value: FunctionDeclaration<'a>},
}

impl<'a> HasId for Statement<'a> {
    fn id(&self) -> Id {
        *match self {
            Statement::Assignment {id, ..} => id,
            Statement::LocalAssignment {id, ..} => id,
            Statement::FunctionCall {id, ..} => id,
            Statement::NumericFor {id, ..} => id,
            Statement::GenericFor {id, ..} => id,
            Statement::IfStatement {id, ..} => id,
            Statement::WhileLoop {id, ..} => id,
            Statement::RepeatLoop {id, ..} => id,
            Statement::FunctionDeclaration {id, ..} => id,
        }
    }
}

// chunk ::= block
// block ::= {stat} [retstat]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk<'a> {
    #[serde(borrow)]
    pub statements: Vec<Statement<'a>>,
}