//! The syntax tree as dump nodes: one node per AST node, in source order.
//!
//! Kind names match the Rust variant names so a port can map them one to one.
//! Attributes are written in a fixed order, and a child that is absent from the
//! source stays absent from the dump.

use super::Node;
use crate::ast::*;

pub fn program(p: &Program) -> Node {
    Node::new("Program")
        .num("expr_count", i64::from(p.expr_count))
        .kids(p.items.iter().map(item))
}

fn item(item: &Item) -> Node {
    match item {
        Item::Fn(f) => fn_decl("Fn", f),
        Item::Const(c) => {
            let n = Node::new("Const")
                .text("name", &c.name)
                .span("name_span", c.name_span)
                .flag("pub", c.is_pub);
            let n = match &c.ty {
                Some(t) => n.child(type_ann(t)),
                None => n,
            };
            n.child(expr(&c.value))
        }
        Item::Class(c) => Node::new("Class")
            .text("name", &c.name)
            .span("name_span", c.name_span)
            .flag("pub", c.is_pub)
            .kids(c.type_params.iter().map(|(name, span)| {
                Node::new("TypeParam")
                    .text("name", name)
                    .span("span", *span)
            }))
            .kids(c.implements.iter().map(|(name, span)| {
                Node::new("Implements")
                    .text("name", name)
                    .span("span", *span)
            }))
            .kids(c.fields.iter().map(field))
            .kids(c.methods.iter().map(|m| fn_decl("Method", m))),
        Item::Interface(i) => Node::new("Interface")
            .text("name", &i.name)
            .span("name_span", i.name_span)
            .flag("pub", i.is_pub)
            .kids(i.methods.iter().map(|m| {
                Node::new("IfaceMethod")
                    .text("name", &m.name)
                    .span("name_span", m.name_span)
                    .span("span", m.span)
                    .kids(m.params.iter().map(param))
                    .kids(m.ret.iter().map(type_ann))
            })),
        Item::Enum(e) => Node::new("Enum")
            .text("name", &e.name)
            .span("name_span", e.name_span)
            .flag("pub", e.is_pub)
            .kids(e.variants.iter().map(|v| {
                Node::new("Variant")
                    .text("name", &v.name)
                    .span("span", v.name_span)
                    .kids(v.fields.iter().map(param))
            })),
        Item::Import(i) => Node::new("Import")
            .text("path", &i.path)
            .span("path_span", i.path_span)
            .kids(
                i.names
                    .iter()
                    .map(|(name, span)| Node::new("Name").text("name", name).span("span", *span)),
            )
            .kids(
                i.alias
                    .iter()
                    .map(|(name, span)| Node::new("Alias").text("name", name).span("span", *span)),
            ),
        Item::Test(t) => Node::new("Test")
            .text("name", &t.name)
            .span("name_span", t.name_span)
            .child(block(&t.body)),
        Item::Stmt(s) => stmt(s),
    }
}

fn fn_decl(kind: &'static str, f: &FnDecl) -> Node {
    let n = Node::new(kind)
        .text("name", &f.name)
        .span("name_span", f.name_span)
        .flag("pub", f.is_pub)
        .span("span", f.span)
        .kids(f.type_params.iter().map(|(name, span)| {
            Node::new("TypeParam")
                .text("name", name)
                .span("span", *span)
        }))
        .kids(f.params.iter().map(param));
    let n = match &f.ret {
        Some(t) => n.child(type_ann(t)),
        None => n,
    };
    match &f.body {
        FnBody::Block(b) => n.child(block(b)),
        FnBody::Expr(e) => n.child(Node::new("ExprBody").child(expr(e))),
    }
}

fn param(p: &Param) -> Node {
    Node::new("Param")
        .text("name", &p.name)
        .span("name_span", p.name_span)
        .child(type_ann(&p.ty))
}

fn field(f: &FieldDecl) -> Node {
    let n = Node::new("Field")
        .text("name", &f.name)
        .span("name_span", f.name_span)
        .child(type_ann(&f.ty));
    match &f.default {
        Some(v) => n.child(Node::new("Default").child(expr(v))),
        None => n,
    }
}

fn type_ann(t: &TypeAnn) -> Node {
    let n = match &t.kind {
        TypeAnnKind::Name(name) => Node::new("TypeName").text("name", name),
        TypeAnnKind::Generic(name, args) => Node::new("TypeGeneric")
            .text("name", name)
            .kids(args.iter().map(type_ann)),
        TypeAnnKind::List(inner) => Node::new("TypeList").child(type_ann(inner)),
        TypeAnnKind::Map(value) => Node::new("TypeMap").child(type_ann(value)),
        TypeAnnKind::Opt(inner) => Node::new("TypeOpt").child(type_ann(inner)),
        TypeAnnKind::Task(inner) => Node::new("TypeTask").child(type_ann(inner)),
        TypeAnnKind::Channel(inner) => Node::new("TypeChannel").child(type_ann(inner)),
        TypeAnnKind::Fn(args, ret) => Node::new("TypeFn")
            .kids(args.iter().map(type_ann))
            .child(type_ann(ret)),
    };
    n.span("span", t.span)
}

fn block(stmts: &Block) -> Node {
    Node::new("Block").kids(stmts.iter().map(stmt))
}

fn stmt(s: &Stmt) -> Node {
    match s {
        Stmt::Assign {
            target,
            ty,
            value,
            span,
        } => {
            let n = Node::new("Assign")
                .span("span", *span)
                .child(target_node(target));
            let n = match ty {
                Some(t) => n.child(type_ann(t)),
                None => n,
            };
            n.child(expr(value))
        }
        Stmt::Expr(e) => Node::new("ExprStmt").child(expr(e)),
        Stmt::If { cond, then, else_ } => {
            let n = Node::new("If")
                .child(expr(cond))
                .child(Node::new("Then").child(block(then)));
            match else_ {
                Some(b) => n.child(Node::new("Else").child(block(b))),
                None => n,
            }
        }
        Stmt::While { cond, body } => Node::new("While").child(expr(cond)).child(block(body)),
        Stmt::For {
            var,
            var_span,
            start,
            end,
            inclusive,
            body,
        } => Node::new("For")
            .text("var", var)
            .span("var_span", *var_span)
            .flag("inclusive", *inclusive)
            .child(expr(start))
            .child(expr(end))
            .child(block(body)),
        Stmt::ForIn { vars, iter, body } => Node::new("ForIn")
            .kids(
                vars.iter()
                    .map(|(name, span)| Node::new("Var").text("name", name).span("span", *span)),
            )
            .child(Node::new("Iter").child(expr(iter)))
            .child(block(body)),
        Stmt::Match {
            scrutinee,
            arms,
            span,
        } => Node::new("Match")
            .span("span", *span)
            .child(Node::new("Scrutinee").child(expr(scrutinee)))
            .kids(arms.iter().map(arm)),
        Stmt::Throw { value, span } => Node::new("Throw").span("span", *span).child(expr(value)),
        Stmt::Try {
            body,
            var,
            handler,
            finally,
        } => {
            let n = Node::new("Try").child(block(body));
            let n = match var {
                Some((name, span)) => {
                    n.child(Node::new("CatchVar").text("name", name).span("span", *span))
                }
                None => n,
            };
            let n = match handler {
                Some(b) => n.child(Node::new("Handler").child(block(b))),
                None => n,
            };
            match finally {
                Some(b) => n.child(Node::new("Finally").child(block(b))),
                None => n,
            }
        }
        Stmt::Break(span) => Node::new("Break").span("span", *span),
        Stmt::Continue(span) => Node::new("Continue").span("span", *span),
        Stmt::Return { value, span } => {
            let n = Node::new("Return").span("span", *span);
            match value {
                Some(v) => n.child(expr(v)),
                None => n,
            }
        }
    }
}

fn target_node(t: &Target) -> Node {
    match t {
        Target::Var(name, span) => Node::new("TargetVar")
            .text("name", name)
            .span("span", *span),
        Target::Index { obj, index } => {
            Node::new("TargetIndex").child(expr(obj)).child(expr(index))
        }
        Target::Field {
            obj,
            name,
            name_span,
        } => Node::new("TargetField")
            .text("name", name)
            .span("name_span", *name_span)
            .child(expr(obj)),
    }
}

fn arm(a: &Arm) -> Node {
    Node::new("Arm")
        .span("pattern_span", a.pattern_span)
        .child(pattern(&a.pattern))
        .child(block(&a.body))
}

fn pattern(p: &Pattern) -> Node {
    match p {
        Pattern::Wildcard => Node::new("Wildcard"),
        Pattern::Value(e) => Node::new("PatternValue").child(expr(e)),
    }
}

fn expr(e: &Expr) -> Node {
    let n = match &e.kind {
        ExprKind::Int(v) => Node::new("Int").num("value", *v),
        ExprKind::Float(v) => Node::new("Float").text("value", format!("{v:?}")),
        ExprKind::Bool(v) => Node::new("Bool").flag("value", *v),
        ExprKind::Str(s) => Node::new("Str").text("value", s),
        ExprKind::Interp(parts) => Node::new("Interp").kids(parts.iter().map(interp)),
        ExprKind::None => Node::new("None"),
        ExprKind::Ident(name) => Node::new("Ident").text("name", name),
        ExprKind::List(items) => Node::new("List").kids(items.iter().map(expr)),
        ExprKind::Map(entries) => Node::new("Map").kids(
            entries
                .iter()
                .map(|(k, v)| Node::new("Entry").child(expr(k)).child(expr(v))),
        ),
        ExprKind::Index { obj, index } => Node::new("Index").child(expr(obj)).child(expr(index)),
        ExprKind::Slice {
            obj,
            start,
            end,
            inclusive,
        } => {
            let n = Node::new("Slice")
                .flag("inclusive", *inclusive)
                .child(expr(obj));
            let n = match start {
                Some(s) => n.child(Node::new("Start").child(expr(s))),
                None => n,
            };
            match end {
                Some(s) => n.child(Node::new("End").child(expr(s))),
                None => n,
            }
        }
        ExprKind::Is {
            obj,
            enum_name,
            variant,
            name_span,
        } => Node::new("Is")
            .text("enum", enum_name)
            .text("variant", variant)
            .span("name_span", *name_span)
            .child(expr(obj)),
        ExprKind::Member {
            obj,
            name,
            name_span,
            safe,
        } => Node::new("Member")
            .text("name", name)
            .span("name_span", *name_span)
            .flag("safe", *safe)
            .child(expr(obj)),
        ExprKind::MethodCall {
            obj,
            name,
            name_span,
            args,
        } => Node::new("MethodCall")
            .text("name", name)
            .span("name_span", *name_span)
            .child(expr(obj))
            .kids(args.iter().map(arg)),
        ExprKind::Unary { op, expr: inner } => {
            Node::new("Unary").raw("op", un_op(*op)).child(expr(inner))
        }
        ExprKind::Binary { op, lhs, rhs } => Node::new("Binary")
            .raw("op", op.symbol())
            .child(expr(lhs))
            .child(expr(rhs)),
        ExprKind::Coalesce { lhs, rhs } => Node::new("Coalesce").child(expr(lhs)).child(expr(rhs)),
        ExprKind::Unwrap(inner) => Node::new("Unwrap").child(expr(inner)),
        ExprKind::Call {
            callee,
            callee_span,
            args,
        } => Node::new("Call")
            .text("callee", callee)
            .span("callee_span", *callee_span)
            .kids(args.iter().map(arg)),
        ExprKind::Closure { params, body } => Node::new("Closure")
            .kids(params.iter().map(closure_param))
            .child(expr(body)),
        ExprKind::Spawn {
            callee,
            callee_span,
            args,
        } => Node::new("Spawn")
            .text("callee", callee)
            .span("callee_span", *callee_span)
            .kids(args.iter().map(arg)),
        ExprKind::Await(inner) => Node::new("Await").child(expr(inner)),
        ExprKind::Catch {
            expr: inner,
            var,
            var_span,
            block: body,
        } => Node::new("Catch")
            .text("var", var)
            .span("var_span", *var_span)
            .child(expr(inner))
            .child(block(body)),
    };
    // The id is deliberately absent: it is allocation order (see the module doc).
    n.span("span", e.span)
}

fn closure_param(p: &ClosureParam) -> Node {
    let n = Node::new("ClosureParam")
        .text("name", &p.name)
        .span("name_span", p.name_span);
    match &p.ty {
        Some(t) => n.child(type_ann(t)),
        None => n,
    }
}

fn interp(part: &InterpPart) -> Node {
    match part {
        InterpPart::Lit(text) => Node::new("Lit").text("text", text),
        InterpPart::Expr(e) => Node::new("Hole").child(expr(e)),
    }
}

fn arg(a: &Arg) -> Node {
    let n = Node::new("Arg");
    let n = match &a.name {
        Some((name, span)) => n.text("name", name).span("name_span", *span),
        None => n,
    };
    n.child(expr(&a.value))
}

fn un_op(op: UnOp) -> &'static str {
    match op {
        UnOp::Neg => "-",
        UnOp::Not => "not",
        UnOp::BitNot => "~",
    }
}
