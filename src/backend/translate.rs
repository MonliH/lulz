use dynasmrt::{dynasm, AssemblyOffset, DynasmApi, DynasmLabelApi, Executor};
use libc::{c_char, c_void, printf};
use std::{io::Read, mem, rc::Rc};

use super::lco::{CompilationCtx, LazyCode, ValueInfo::*};
use crate::{
    backend::lco::Function,
    frontend::ast::{Expr, ExprKind, OpTy, Statement, StatementKind},
};

pub fn dump_hex(exec: &Executor) -> String {
    exec.lock()
        .bytes()
        .map(|b| format!("{:02X?}", b.unwrap()))
        .collect::<Vec<String>>()
        .join(" ")
}

extern "sysv64" fn run_closure(closure: *mut c_void, ctx: *mut c_void) {
    // Closure is a &mut &mut LazyCode
    let lazy_code: &mut &mut LazyCode = unsafe { mem::transmute(closure) };
    let ctx_ptr: &mut CompilationCtx = unsafe { mem::transmute(ctx) };
    lazy_code(ctx_ptr);
}

extern "sysv64" fn print_bytes(bytes: *mut c_char) {
    unsafe { printf(bytes) };
}

pub fn translate_ast<'a>(ast: &'a [Statement], succ: LazyCode<'a>) -> LazyCode<'a> {
    if ast.is_empty() {
        return succ;
    }
    let stmt = &ast[0];
    match &stmt.statement_kind {
        StatementKind::Expr(e) => translate_expr(e, succ),
        StatementKind::FunctionDef(id, args, block) => LazyCode(Rc::new(move |ctx| {
            let poped = ctx.commit_current(id.0, Function::new());
            let closure_fat_ref = ctx.bump.alloc(LazyCode(Rc::new(|ctx| {
                let poped = ctx.commit_current(id.0, Function::new());
                translate_ast(
                    &block.0,
                    LazyCode(Rc::new(|ctx| {
                        println!("function gen over!");
                    })),
                )(ctx);
                ctx.pop_commit(poped);
            })));
            let closure_ref = ctx.bump.alloc(closure_fat_ref);
            let closure_ptr: *mut c_void = unsafe { mem::transmute(closure_ref) };
            let ctx_ptr: *mut c_void = unsafe {
                // Copy the mutable reference to ctx for the assembly reference to use
                // FIXME: This is unsound, as we have two mutable references as a time.
                mem::transmute_copy(&ctx)
            };

            dynasm!(ctx.f.asm
            ; mov rdi, QWORD closure_ptr as _
            ; mov rsi, QWORD ctx_ptr as _
            ; mov rax, QWORD run_closure as _
            ; call rax
            // Return 0 as a fallback
            ; xor rax, rax
            ; ret
            );
            ctx.pop_commit(poped);
            translate_ast(&ast[1..], succ.clone())(ctx);
        })),
        StatementKind::DecAssign(id, expr) => LazyCode(Rc::new(move |ctx| {
            let ex = match &expr {
                Some(Ok(e)) => e,
                _ => todo!(),
            };
            translate_expr(
                ex,
                LazyCode(Rc::new(|ctx| {
                    let info = ctx.pop().unwrap();
                    ctx.define_var(id.0, info);
                    translate_ast(&ast[1..], succ.clone())(ctx);
                })),
            )(ctx);
        })),
        StatementKind::Print(es, no_newline) => translate_expr(
            &es[0],
            LazyCode(Rc::new(move |ctx| {
                let info = ctx.pop().unwrap();
                dynasm!(ctx.f.asm
                ; jmp >raw_bytes_end
                );
                match info {
                    // Load expression
                    ConstInt(i) => {
                        dynasm!(ctx.f.asm
                        ; raw_bytes:
                        ; .bytes i.to_string().as_bytes());
                        if !*no_newline {
                            // Add newline
                            dynasm!(ctx.f.asm
                            ; .byte 10
                            );
                        }
                        // null terminator
                        dynasm!(ctx.f.asm
                        ; .byte 0
                        );
                    }
                    ConstFloat(f) => {
                        dynasm!(ctx.f.asm
                        ; raw_bytes:
                        ; .bytes f.to_string().as_bytes());
                        if !*no_newline {
                            dynasm!(ctx.f.asm
                            ; .byte 10
                            );
                        }
                        dynasm!(ctx.f.asm
                        ; .byte 0
                        );
                    }
                    _ => panic!("not implemented"),
                };
                dynasm!(ctx.f.asm
                ; raw_bytes_end:
                ; lea rdi, [<raw_bytes]
                ; mov rax, QWORD print_bytes as _
                ; call rax
                );
                translate_ast(&ast[1..], succ.clone())(ctx);
            })),
        ),
        _ => todo!("Statement not implemented"),
    }
}

fn translate_expr<'a>(expr: &'a Expr, succ: LazyCode<'a>) -> LazyCode<'a> {
    match &expr.expr_kind {
        ExprKind::Int(i) => LazyCode(Rc::new(move |ctx| {
            ctx.push(ConstInt(*i));
            succ(ctx);
        })),
        ExprKind::Float(f) => LazyCode(Rc::new(move |ctx| {
            ctx.push(ConstFloat(*f));
            succ(ctx);
        })),
        ExprKind::FunctionCall(id, args) => LazyCode(Rc::new(move |ctx| {
            let func = ctx.functions.get_mut(&id.0).unwrap();
            func.asm.commit().unwrap();
            let func_reader = func.asm.reader();
            let fn_ptr = func_reader.lock().ptr(AssemblyOffset(0));
            dynasm!(ctx.f.asm
            ; mov rax, QWORD fn_ptr as _
            ; call rax
            );
            succ(ctx);
        })),
        ExprKind::Variable(id) => LazyCode(Rc::new(move |ctx| {
            let var_ty = *ctx.var_type(id.0);
            ctx.push(var_ty);
            match var_ty {
                Type(_) => {
                    let local_pos = 8 * (ctx.var_pos(&id.0) + 1);
                    dynasm!(ctx.f.asm
                    ; push QWORD [rsp + local_pos as i32]
                    );
                }
                _ => {}
            }
            succ(ctx)
        })),
        ExprKind::Operator(op, l, r) => match op {
            OpTy::Add => {
                let left = LazyCode(Rc::new(move |ctx| {
                    let right = LazyCode(Rc::new(|ctx| {
                        let add_op = LazyCode(Rc::new(|ctx| {
                            let r = ctx.pop().unwrap();
                            let l = ctx.pop().unwrap();
                            match (l, r) {
                                (ConstFloat(n1), ConstFloat(n2)) => ctx.push(ConstFloat(n1 + n2)),
                                (ConstInt(n1), ConstInt(n2)) => ctx.push(ConstInt(n1 + n2)),
                                _ => todo!("not implemented"),
                            }
                            succ(ctx)
                        }));
                        translate_expr(&*r, add_op)(ctx);
                    }));
                    translate_expr(&*l, right)(ctx);
                }));
                left
            }
            _ => todo!("Operator not implemented"),
        },
        _ => todo!("Expression not implemented"),
    }
}
