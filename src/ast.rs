#[derive(Debug)]
pub struct CompUnit {
    pub items:Vec<GlobalItem>,
}
#[derive(Debug)]
pub enum GlobalItem {
    Decl(Decl),
    FuncDef(FuncDef)
}
#[derive(Debug)]
pub enum Decl {
    ConstDecl(ConstDecl),
    VarDecl(VarDecl)
}
#[derive(Debug)]
pub struct ConstDecl {
    pub btype:Btype,
    pub const_defs:Vec<ConstDef>
}
#[derive(Debug)]
pub enum Btype {
    Int,
    Float
}
#[derive(Debug)]
pub struct ConstDef {
    pub ident:String,
    pub const_exps:Vec<ConstExp>,
    pub const_init_val:ConstInitVal
}
#[derive(Debug)]
pub enum ConstInitVal {
    ConstExp(ConstExp),
    ConstInitVals(Vec<ConstInitVal>)
}
#[derive(Debug)]
pub struct VarDecl {
    pub btype:Btype,
    pub var_defs: Vec<VarDef>
}
#[derive(Debug)]
pub enum VarDef {
    WithInitVal(String, Vec<ConstExp>, InitVal),
    Without(String, Vec<ConstExp>)
}
#[derive(Debug)]
pub enum InitVal {
    Exp(Exp),
    InitVals(Vec<InitVal>)
}

#[derive(Debug)]
pub struct FuncDef {
    pub func_type:FuncType,
    pub ident: String,
    pub block: Block,
    pub func_f_params: Option<FuncFParams>
}
#[derive(Debug)]
pub enum FuncType{
    Int,
    Float,
    Void
}
#[derive(Debug)]
pub struct FuncFParams {
    pub items: Vec<FuncFParam>  
}
#[derive(Debug)]
pub struct FuncFParam {
    pub btype: Btype,
    pub ident: String,
    pub dims: Option<Vec<Exp>>
}
#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>
}
#[derive(Debug)]
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt)
}
#[derive(Debug)]
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
#[derive(Debug)]
pub struct Exp {
    pub item: AddExp
}
#[derive(Debug)]
pub struct Cond {
    pub item: LOrExp
}
#[derive(Debug)]
pub struct LVal {
    pub ident: String,
    pub dims: Vec<Exp>
}
#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    LVal(LVal),
    Number(Number)
}
#[derive(Debug)]
pub enum Number {
    IntConst(i32),
    FloatConst(f32)
}
#[derive(Debug)]
pub enum UnaryExp {
    PrimaryExp(Box<PrimaryExp>),
    CallExp(String, Option<FuncRParams>),//? 此处的函数调用一定要有返回值吗
    UnaryExp(UnaryOp, Box<UnaryExp>)
}
#[derive(Debug)]
pub enum UnaryOp {
    Minus,
    Not
}
#[derive(Debug)]
pub struct FuncRParams {
    pub items: Vec<Exp>
}
#[derive(Debug)]
pub enum MulExp {
    UnaryExp(Box<UnaryExp>),
    MulExp(Box<MulExp>, MulOp, UnaryExp)
}
#[derive(Debug)]
pub enum MulOp {
    Mul,
    Div,
    Mod
}
#[derive(Debug)]
pub enum AddExp {
    MulExp(Box<MulExp>),
    AddExp(Box<AddExp>, AddOp, MulExp)
}
#[derive(Debug)]
pub enum AddOp {
    Add,
    Sub
}
#[derive(Debug)]
pub enum RelExp {
    AddExp(AddExp),
    RelExp(Box<RelExp>, RelOp, AddExp)
}
#[derive(Debug)]
pub enum RelOp {
    GT,
    LT,
    GE,
    LE
}
#[derive(Debug)]
pub enum EqExp {
    RelExp(RelExp),
    EqExp(Box<EqExp>, EqOp, RelExp)
}
#[derive(Debug)]
pub enum EqOp {
    EQ,
    NE
}
#[derive(Debug)]
pub enum LAndExp {
    EqExp(EqExp),
    LAndExp(Box<LAndExp>, EqExp)
}
#[derive(Debug)]
pub enum LOrExp {
    LAndExp(LAndExp),
    LOrExp(Box<LOrExp>, LAndExp)
}
#[derive(Debug)]
pub struct  ConstExp {
    pub item: AddExp
}