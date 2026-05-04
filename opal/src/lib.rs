#![feature(str_from_raw_parts)]
#![feature(atomic_ptr_null)]
pub mod ast;
pub mod bytecode;
pub mod heap;
pub mod heap2;
pub mod infer;
pub mod instr;
pub mod intern;
pub mod lexer;
pub mod lower;
pub mod parser;
pub mod runtime;
pub mod scope;
pub mod token;
pub mod ty;
pub mod typed_ast;
pub mod value;
pub mod vm;
