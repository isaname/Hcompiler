#[derive(Debug, Clone)]
pub struct CompUnit {
    pub items:Vec<GlobalItem>,
}
#[derive(Debug, Clone)]
pub enum GlobalItem {
    Decl(Decl),
    FuncDef(FuncDef)
}
#[derive(Debug, Clone)]
pub enum Decl {
    ConstDecl(ConstDecl),
    VarDecl(VarDecl)
}
#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub btype:Btype,
    pub const_defs:Vec<ConstDef>
}
#[derive(Debug, Clone)]
pub enum Btype {
    Int,
    Float
}
#[derive(Debug, Clone)]
pub struct ConstDef {
    pub ident:String,
    pub const_exps:Vec<ConstExp>,
    pub const_init_val:ConstInitVal
}
#[derive(Debug, Clone)]
pub enum ConstInitVal {
    ConstExp(ConstExp),
    ConstInitVals(Vec<ConstInitVal>)
}
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub btype:Btype,
    pub var_defs: Vec<VarDef>
}
#[derive(Debug, Clone)]
pub enum VarDef {
    WithInitVal(String, Vec<ConstExp>, InitVal),
    Without(String, Vec<ConstExp>)
}
#[derive(Debug, Clone)]
pub enum InitVal {
    Exp(Exp),
    InitVals(Vec<InitVal>)
}

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub func_type:FuncType,
    pub ident: String,
    pub block: Block,
    pub func_f_params: Option<FuncFParams>
}
#[derive(Debug, Clone)]
pub enum FuncType{
    Int,
    Float,
    Void
}
#[derive(Debug, Clone)]
pub struct FuncFParams {
    pub items: Vec<FuncFParam>
}
#[derive(Debug, Clone)]
pub struct FuncFParam {
    pub btype: Btype,
    pub ident: String,
    pub dims: Option<Vec<ConstExp>>
}
#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<BlockItem>
}
#[derive(Debug, Clone)]
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt)
}
#[derive(Debug, Clone)]
pub enum Stmt {
    AssignStmt(LVal,Exp),
    ExpStmt(Option<Exp>),
    BlockStmt(Block),
    IfStmt(Cond, Box<Stmt>, Option<Box<Stmt>>),
    WhileStmt(Cond, Box<Stmt>),
    BreakStmt,
    ContStmt,
    RetStmt(Option<Exp>)
}
#[derive(Debug, Clone)]
pub struct Exp {
    pub item: AddExp
}
#[derive(Debug, Clone)]
pub struct Cond {
    pub item: LOrExp
}
#[derive(Debug, Clone)]
pub struct LVal {
    pub ident: String,
    pub dims: Vec<Exp>
}
#[derive(Debug, Clone)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    LVal(LVal),
    Number(Number)
}
#[derive(Debug, Clone)]
pub enum Number {
    IntConst(i32),
    FloatConst(f32)
}
#[derive(Debug, Clone)]
pub enum UnaryExp {
    PrimaryExp(Box<PrimaryExp>),
    CallExp(String, Option<FuncRParams>),//? 此处的函数调用一定要有返回值吗
    UnaryExp(UnaryOp, Box<UnaryExp>)
}
#[derive(Debug, Clone)]
pub enum UnaryOp {
    Pos,
    Minus,
    Not
}
#[derive(Debug, Clone)]
pub struct FuncRParams {
    pub items: Vec<Exp>
}
#[derive(Debug, Clone)]
pub enum MulExp {
    UnaryExp(Box<UnaryExp>),
    MulExp(Box<MulExp>, MulOp, UnaryExp)
}
#[derive(Debug, Clone)]
pub enum MulOp {
    Mul,
    Div,
    Mod
}
#[derive(Debug, Clone)]
pub enum AddExp {
    MulExp(Box<MulExp>),
    AddExp(Box<AddExp>, AddOp, MulExp)
}
#[derive(Debug, Clone)]
pub enum AddOp {
    Add,
    Sub
}
#[derive(Debug, Clone)]
pub enum RelExp {
    AddExp(AddExp),
    RelExp(Box<RelExp>, RelOp, AddExp)
}
#[derive(Debug, Clone)]
pub enum RelOp {
    GT,
    LT,
    GE,
    LE
}
#[derive(Debug, Clone)]
pub enum EqExp {
    RelExp(RelExp),
    EqExp(Box<EqExp>, EqOp, RelExp)
}
#[derive(Debug, Clone)]
pub enum EqOp {
    EQ,
    NE
}
#[derive(Debug, Clone)]
pub enum LAndExp {
    EqExp(EqExp),
    LAndExp(Box<LAndExp>, EqExp)
}
#[derive(Debug, Clone)]
pub enum LOrExp {
    LAndExp(LAndExp),
    LOrExp(Box<LOrExp>, LAndExp)
}
#[derive(Debug, Clone)]
pub struct  ConstExp {
    pub item: AddExp
}