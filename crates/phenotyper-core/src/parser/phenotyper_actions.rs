/// This file is maintained by rustemo but can be modified manually.
/// All manual changes will be preserved except non-doc comments.
use rustemo::Token as RustemoToken;
use super::phenotyper::{TokenKind, Context};
pub type Input = str;
pub type Ctx<'i> = Context<'i, Input>;
#[allow(dead_code)]
pub type Token<'i> = RustemoToken<'i, Input, TokenKind>;
pub type Ident = String;
pub fn ident(_ctx: &Ctx, token: Token) -> Ident {
    token.value.into()
}
pub type StringLiteral = String;
pub fn string_literal(_ctx: &Ctx, token: Token) -> StringLiteral {
    token.value.into()
}
#[derive(Debug, Clone)]
pub struct File {
    pub ns: NamespaceScope,
}
pub fn file_c1(_ctx: &Ctx, ns: NamespaceScope) -> File {
    File { ns }
}
#[derive(Debug, Clone)]
pub struct NamespaceScope {
    pub path: NamespacePath,
    pub uses: UsesDecl0,
    pub decls: TopLevelDecl0,
}
pub fn namespace_scope_c1(
    _ctx: &Ctx,
    path: NamespacePath,
    uses: UsesDecl0,
    decls: TopLevelDecl0,
) -> NamespaceScope {
    NamespaceScope {
        path,
        uses,
        decls,
    }
}
pub type UsesDecl1 = Vec<UsesDecl>;
pub fn uses_decl1_c1(
    _ctx: &Ctx,
    mut uses_decl1: UsesDecl1,
    uses_decl: UsesDecl,
) -> UsesDecl1 {
    uses_decl1.push(uses_decl);
    uses_decl1
}
pub fn uses_decl1_uses_decl(_ctx: &Ctx, uses_decl: UsesDecl) -> UsesDecl1 {
    vec![uses_decl]
}
pub type UsesDecl0 = Option<UsesDecl1>;
pub fn uses_decl0_uses_decl1(_ctx: &Ctx, uses_decl1: UsesDecl1) -> UsesDecl0 {
    Some(uses_decl1)
}
pub fn uses_decl0_empty(_ctx: &Ctx) -> UsesDecl0 {
    None
}
pub type TopLevelDecl1 = Vec<TopLevelDecl>;
pub fn top_level_decl1_c1(
    _ctx: &Ctx,
    mut top_level_decl1: TopLevelDecl1,
    top_level_decl: TopLevelDecl,
) -> TopLevelDecl1 {
    top_level_decl1.push(top_level_decl);
    top_level_decl1
}
pub fn top_level_decl1_top_level_decl(
    _ctx: &Ctx,
    top_level_decl: TopLevelDecl,
) -> TopLevelDecl1 {
    vec![top_level_decl]
}
pub type TopLevelDecl0 = Option<TopLevelDecl1>;
pub fn top_level_decl0_top_level_decl1(
    _ctx: &Ctx,
    top_level_decl1: TopLevelDecl1,
) -> TopLevelDecl0 {
    Some(top_level_decl1)
}
pub fn top_level_decl0_empty(_ctx: &Ctx) -> TopLevelDecl0 {
    None
}
pub type NamespacePath = Ident1;
pub fn namespace_path_ident1(_ctx: &Ctx, ident1: Ident1) -> NamespacePath {
    ident1
}
pub type Ident1 = Vec<Ident>;
pub fn ident1_c1(_ctx: &Ctx, mut ident1: Ident1, ident: Ident) -> Ident1 {
    ident1.push(ident);
    ident1
}
pub fn ident1_ident(_ctx: &Ctx, ident: Ident) -> Ident1 {
    vec![ident]
}
#[derive(Debug, Clone)]
pub struct UsesDecl {
    pub path: NamespacePath,
}
pub fn uses_decl_c1(_ctx: &Ctx, path: NamespacePath) -> UsesDecl {
    UsesDecl { path }
}
#[derive(Debug, Clone)]
pub enum TopLevelDecl {
    TypeDecl(TypeDecl),
    TypeDef(TypeDef),
}
pub fn top_level_decl_type_decl(_ctx: &Ctx, type_decl: TypeDecl) -> TopLevelDecl {
    TopLevelDecl::TypeDecl(type_decl)
}
pub fn top_level_decl_type_def(_ctx: &Ctx, type_def: TypeDef) -> TopLevelDecl {
    TopLevelDecl::TypeDef(type_def)
}
#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: Ident,
    pub body: TypeDeclBody,
}
pub fn type_decl_c1(_ctx: &Ctx, name: Ident, body: TypeDeclBody) -> TypeDecl {
    TypeDecl { name, body }
}
#[derive(Debug, Clone)]
pub struct Alias {
    pub alias: TypeExpr,
}
#[derive(Debug, Clone)]
pub struct Enum {
    pub members: EnumMembers,
}
#[derive(Debug, Clone)]
pub enum TypeDeclBody {
    Alias(Alias),
    Enum(Enum),
}
pub fn type_decl_body_alias(_ctx: &Ctx, alias: TypeExpr) -> TypeDeclBody {
    TypeDeclBody::Alias(Alias { alias })
}
pub fn type_decl_body_enum(_ctx: &Ctx, members: EnumMembers) -> TypeDeclBody {
    TypeDeclBody::Enum(Enum { members })
}
#[derive(Debug, Clone)]
pub struct Cons {
    pub first: Ident,
    pub rest: Box<EnumMembers>,
}
#[derive(Debug, Clone)]
pub struct Single {
    pub last: Ident,
}
#[derive(Debug, Clone)]
pub enum EnumMembers {
    Cons(Cons),
    Single(Single),
}
pub fn enum_members_cons(_ctx: &Ctx, first: Ident, rest: EnumMembers) -> EnumMembers {
    EnumMembers::Cons(Cons {
        first,
        rest: Box::new(rest),
    })
}
pub fn enum_members_single(_ctx: &Ctx, last: Ident) -> EnumMembers {
    EnumMembers::Single(Single { last })
}
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: Ident,
    pub plural_clause: PluralClauseOpt,
    pub items: BodyItem1,
}
pub fn type_def_c1(
    _ctx: &Ctx,
    name: Ident,
    plural_clause: PluralClauseOpt,
    items: BodyItem1,
) -> TypeDef {
    TypeDef {
        name,
        plural_clause,
        items,
    }
}
pub type PluralClauseOpt = Option<PluralClause>;
pub fn plural_clause_opt_plural_clause(
    _ctx: &Ctx,
    plural_clause: PluralClause,
) -> PluralClauseOpt {
    Some(plural_clause)
}
pub fn plural_clause_opt_empty(_ctx: &Ctx) -> PluralClauseOpt {
    None
}
pub type BodyItem1 = Vec<BodyItem>;
pub fn body_item1_c1(
    _ctx: &Ctx,
    mut body_item1: BodyItem1,
    body_item: BodyItem,
) -> BodyItem1 {
    body_item1.push(body_item);
    body_item1
}
pub fn body_item1_body_item(_ctx: &Ctx, body_item: BodyItem) -> BodyItem1 {
    vec![body_item]
}
#[derive(Debug, Clone)]
pub struct PluralClause {
    pub plural_name: Ident,
}
pub fn plural_clause_c1(_ctx: &Ctx, plural_name: Ident) -> PluralClause {
    PluralClause { plural_name }
}
#[derive(Debug, Clone)]
pub struct Field {
    pub field: FieldDecl,
}
#[derive(Debug, Clone)]
pub struct Render {
    pub render: RenderExpr,
}
#[derive(Debug, Clone)]
pub enum BodyItem {
    Field(Field),
    Render(Render),
}
pub fn body_item_field(_ctx: &Ctx, field: FieldDecl) -> BodyItem {
    BodyItem::Field(Field { field })
}
pub fn body_item_render(_ctx: &Ctx, render: RenderExpr) -> BodyItem {
    BodyItem::Render(Render { render })
}
#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: Ident,
    pub req: Requiredness,
    pub type_expr: TypeExpr,
}
pub fn field_decl_c1(
    _ctx: &Ctx,
    name: Ident,
    req: Requiredness,
    type_expr: TypeExpr,
) -> FieldDecl {
    FieldDecl { name, req, type_expr }
}
#[derive(Debug, Clone)]
pub enum Requiredness {
    Req,
    Opt,
}
pub fn requiredness_req(_ctx: &Ctx) -> Requiredness {
    Requiredness::Req
}
pub fn requiredness_opt(_ctx: &Ctx) -> Requiredness {
    Requiredness::Opt
}
#[derive(Debug, Clone)]
pub struct Cardinalized {
    pub base: TypeName,
    pub card: CardinalityOp,
}
#[derive(Debug, Clone)]
pub struct Union {
    pub members: TypeName1,
}
#[derive(Debug, Clone)]
pub struct Simple {
    pub name: TypeName,
}
#[derive(Debug, Clone)]
pub enum TypeExpr {
    Cardinalized(Cardinalized),
    Union(Union),
    Simple(Simple),
}
pub fn type_expr_cardinalized(
    _ctx: &Ctx,
    base: TypeName,
    card: CardinalityOp,
) -> TypeExpr {
    TypeExpr::Cardinalized(Cardinalized { base, card })
}
pub fn type_expr_union(_ctx: &Ctx, members: TypeName1) -> TypeExpr {
    TypeExpr::Union(Union { members })
}
pub fn type_expr_simple(_ctx: &Ctx, name: TypeName) -> TypeExpr {
    TypeExpr::Simple(Simple { name })
}
pub type TypeName1 = Vec<TypeName>;
pub fn type_name1_c1(
    _ctx: &Ctx,
    mut type_name1: TypeName1,
    type_name: TypeName,
) -> TypeName1 {
    type_name1.push(type_name);
    type_name1
}
pub fn type_name1_type_name(_ctx: &Ctx, type_name: TypeName) -> TypeName1 {
    vec![type_name]
}
#[derive(Debug, Clone)]
pub enum CardinalityOp {
    Plus,
    Star,
}
pub fn cardinality_op_plus(_ctx: &Ctx) -> CardinalityOp {
    CardinalityOp::Plus
}
pub fn cardinality_op_star(_ctx: &Ctx) -> CardinalityOp {
    CardinalityOp::Star
}
#[derive(Debug, Clone)]
pub struct UserDefined {
    pub name: Ident,
}
#[derive(Debug, Clone)]
pub enum TypeName {
    UserDefined(UserDefined),
    String,
    Int64,
    Real64,
    Bool,
    Date,
    Time,
    DateTime,
}
pub fn type_name_user_defined(_ctx: &Ctx, name: Ident) -> TypeName {
    TypeName::UserDefined(UserDefined { name })
}
pub fn type_name_string(_ctx: &Ctx) -> TypeName {
    TypeName::String
}
pub fn type_name_int64(_ctx: &Ctx) -> TypeName {
    TypeName::Int64
}
pub fn type_name_real64(_ctx: &Ctx) -> TypeName {
    TypeName::Real64
}
pub fn type_name_bool(_ctx: &Ctx) -> TypeName {
    TypeName::Bool
}
pub fn type_name_date(_ctx: &Ctx) -> TypeName {
    TypeName::Date
}
pub fn type_name_time(_ctx: &Ctx) -> TypeName {
    TypeName::Time
}
pub fn type_name_date_time(_ctx: &Ctx) -> TypeName {
    TypeName::DateTime
}
#[derive(Debug, Clone)]
pub struct Directive {
    pub name: Ident,
    pub suffix: DirectiveSuffix,
}
#[derive(Debug, Clone)]
pub struct BareDirective {
    pub name: Ident,
}
#[derive(Debug, Clone)]
pub struct FieldRef {
    pub ref_name: Ident,
}
#[derive(Debug, Clone)]
pub struct StringLit {
    pub value: StringLiteral,
}
#[derive(Debug, Clone)]
pub enum RenderExpr {
    Directive(Directive),
    BareDirective(BareDirective),
    FieldRef(FieldRef),
    StringLit(StringLit),
}
pub fn render_expr_directive(
    _ctx: &Ctx,
    name: Ident,
    suffix: DirectiveSuffix,
) -> RenderExpr {
    RenderExpr::Directive(Directive { name, suffix })
}
pub fn render_expr_bare_directive(_ctx: &Ctx, name: Ident) -> RenderExpr {
    RenderExpr::BareDirective(BareDirective { name })
}
pub fn render_expr_field_ref(_ctx: &Ctx, ref_name: Ident) -> RenderExpr {
    RenderExpr::FieldRef(FieldRef { ref_name })
}
pub fn render_expr_string_lit(_ctx: &Ctx, value: StringLiteral) -> RenderExpr {
    RenderExpr::StringLit(StringLit { value })
}
#[derive(Debug, Clone)]
pub struct WithArgs {
    pub args: Argument1,
    pub block: BlockBodyOpt,
}
#[derive(Debug, Clone)]
pub struct EmptyParen {
    pub block: BlockBodyOpt,
}
#[derive(Debug, Clone)]
pub enum DirectiveSuffix {
    WithArgs(WithArgs),
    EmptyParen(EmptyParen),
}
pub fn directive_suffix_with_args(
    _ctx: &Ctx,
    args: Argument1,
    block: BlockBodyOpt,
) -> DirectiveSuffix {
    DirectiveSuffix::WithArgs(WithArgs { args, block })
}
pub fn directive_suffix_empty_paren(_ctx: &Ctx, block: BlockBodyOpt) -> DirectiveSuffix {
    DirectiveSuffix::EmptyParen(EmptyParen { block })
}
pub type Argument1 = Vec<Argument>;
pub fn argument1_c1(
    _ctx: &Ctx,
    mut argument1: Argument1,
    argument: Argument,
) -> Argument1 {
    argument1.push(argument);
    argument1
}
pub fn argument1_argument(_ctx: &Ctx, argument: Argument) -> Argument1 {
    vec![argument]
}
pub type BlockBodyOpt = Option<BlockBody>;
pub fn block_body_opt_block_body(_ctx: &Ctx, block_body: BlockBody) -> BlockBodyOpt {
    Some(block_body)
}
pub fn block_body_opt_empty(_ctx: &Ctx) -> BlockBodyOpt {
    None
}
#[derive(Debug, Clone)]
pub struct BlockBody {
    pub items: RenderExpr1,
}
pub fn block_body_c1(_ctx: &Ctx, items: RenderExpr1) -> BlockBody {
    BlockBody { items }
}
pub type RenderExpr1 = Vec<Box<RenderExpr>>;
pub fn render_expr1_c1(
    _ctx: &Ctx,
    mut render_expr1: RenderExpr1,
    render_expr: RenderExpr,
) -> RenderExpr1 {
    render_expr1.push(Box::new(render_expr));
    render_expr1
}
pub fn render_expr1_render_expr(_ctx: &Ctx, render_expr: RenderExpr) -> RenderExpr1 {
    vec![Box::new(render_expr)]
}
#[derive(Debug, Clone)]
pub struct IdentArg {
    pub val: Ident,
}
#[derive(Debug, Clone)]
pub struct LiteralArg {
    pub val: StringLiteral,
}
#[derive(Debug, Clone)]
pub enum Argument {
    IdentArg(IdentArg),
    LiteralArg(LiteralArg),
}
pub fn argument_ident_arg(_ctx: &Ctx, val: Ident) -> Argument {
    Argument::IdentArg(IdentArg { val })
}
pub fn argument_literal_arg(_ctx: &Ctx, val: StringLiteral) -> Argument {
    Argument::LiteralArg(LiteralArg { val })
}
