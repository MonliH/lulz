mod expressions;
mod helpers;
mod statements;

use crate::ast::Block;
use crate::diagnostics::Failible;

use self::helpers::Ctx;
use statements::Exec;

pub fn run(ast: Block) -> Failible<()> {
    let mut ctx = Ctx::new();
    ast.exec(&mut ctx)
}
