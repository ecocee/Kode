//! MIR -> native object file (Cranelift), then link with the Kode runtime.

mod function;
mod link;
mod runtime;

use cranelift_codegen::ir::{types, AbiParam, InstBuilder};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{default_libcall_names, DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use kode_mir::{MirFunc, MirProgram, StdSig, Ty};

pub use link::link;

pub(crate) fn clif_ty(t: Ty) -> types::Type {
    match t {
        Ty::Int | Ty::Ref | Ty::Unit => types::I64,
        Ty::Float => types::F64,
        Ty::Bool => types::I8,
    }
}

fn signature(module: &ObjectModule, f: &MirFunc) -> cranelift_codegen::ir::Signature {
    let mut sig = module.make_signature();
    for t in f.param_tys() {
        sig.params.push(AbiParam::new(clif_ty(t)));
    }
    if f.ret != Ty::Unit {
        sig.returns.push(AbiParam::new(clif_ty(f.ret)));
    }
    sig
}

fn declare_std(module: &mut ObjectModule, s: &StdSig) -> Result<FuncId, String> {
    let mut sig = module.make_signature();
    for p in &s.params {
        sig.params.push(AbiParam::new(if *p == Ty::Float {
            types::F64
        } else {
            types::I64
        }));
    }
    match s.ret {
        Ty::Unit => {}
        Ty::Float => sig.returns.push(AbiParam::new(types::F64)),
        _ => sig.returns.push(AbiParam::new(types::I64)),
    }
    module
        .declare_function(s.name, Linkage::Import, &sig)
        .map_err(|e| e.to_string())
}

/// Static string literals: `[refcount = -1][kind = string][length][bytes]`.
fn define_strings(module: &mut ObjectModule, strings: &[String]) -> Result<Vec<DataId>, String> {
    let mut ids = Vec::new();
    for s in strings {
        let id = module
            .declare_anonymous_data(false, false)
            .map_err(|e| e.to_string())?;
        let mut bytes = Vec::with_capacity(24 + s.len());
        bytes.extend_from_slice(&(-1i64).to_le_bytes());
        bytes.extend_from_slice(&1i64.to_le_bytes());
        bytes.extend_from_slice(&(s.len() as i64).to_le_bytes());
        bytes.extend_from_slice(s.as_bytes());
        let mut desc = DataDescription::new();
        desc.define(bytes.into_boxed_slice());
        desc.set_align(8);
        module.define_data(id, &desc).map_err(|e| e.to_string())?;
        ids.push(id);
    }
    Ok(ids)
}

/// Compile a MIR program to the bytes of a native object file.
pub fn compile_to_object(program: &MirProgram, optimize: bool) -> Result<Vec<u8>, String> {
    let mut flags = settings::builder();
    flags
        .set("opt_level", if optimize { "speed" } else { "none" })
        .map_err(|e| e.to_string())?;
    flags.set("is_pic", "true").map_err(|e| e.to_string())?;
    // No frame pointer: one more register and a shorter function entry and exit.
    flags
        .set("preserve_frame_pointers", "false")
        .map_err(|e| e.to_string())?;
    let isa = cranelift_native::builder()
        .map_err(|e| e.to_string())?
        .finish(settings::Flags::new(flags))
        .map_err(|e| e.to_string())?;
    let builder =
        ObjectBuilder::new(isa, "kode", default_libcall_names()).map_err(|e| e.to_string())?;
    let mut module = ObjectModule::new(builder);
    if module.target_config().pointer_type() != types::I64 {
        return Err("only 64-bit targets are supported".into());
    }

    let rt = runtime::declare(&mut module)?;
    let strings = define_strings(&mut module, &program.strings)?;

    // Declare every function first, so calls can refer to later ones.
    let mut ids: Vec<FuncId> = Vec::new();
    for (i, f) in program.funcs.iter().enumerate() {
        let name = if i == program.entry {
            "kode_script_main".to_string()
        } else {
            format!("kode_fn_{}", f.name)
        };
        let sig = signature(&module, f);
        ids.push(
            module
                .declare_function(&name, Linkage::Local, &sig)
                .map_err(|e| e.to_string())?,
        );
    }
    // Task thunks come after the user functions in `ids`.
    let nfuncs = program.funcs.len();
    for (i, f) in program.thunks.iter().enumerate() {
        let sig = signature(&module, f);
        ids.push(
            module
                .declare_function(&format!("kode_thunk_{i}"), Linkage::Local, &sig)
                .map_err(|e| e.to_string())?,
        );
    }
    // Closure bodies and named-function wrappers.
    let nclosures = program.closures.len();
    for (i, f) in program.closures.iter().enumerate() {
        let sig = signature(&module, f);
        ids.push(
            module
                .declare_function(&format!("kode_closure_{i}"), Linkage::Local, &sig)
                .map_err(|e| e.to_string())?,
        );
    }
    let nfnvals = program.fnvals.len();
    for (i, f) in program.fnvals.iter().enumerate() {
        let sig = signature(&module, f);
        ids.push(
            module
                .declare_function(&format!("kode_fnval_{i}"), Linkage::Local, &sig)
                .map_err(|e| e.to_string())?,
        );
    }
    // Runtime library functions called by name.
    let mut std_ids: Vec<FuncId> = Vec::new();
    for s in &program.std_sigs {
        std_ids.push(declare_std(&mut module, s)?);
    }

    let mut fb_ctx = FunctionBuilderContext::new();
    let all: Vec<&MirFunc> = program
        .funcs
        .iter()
        .chain(program.thunks.iter())
        .chain(program.closures.iter())
        .chain(program.fnvals.iter())
        .collect();
    let nthunks = program.thunks.len();
    for (i, f) in all.iter().enumerate() {
        let mut ctx = module.make_context();
        ctx.func.signature = signature(&module, f);
        {
            let mut env = function::Env {
                module: &mut module,
                rt: &rt,
                user: &ids[..nfuncs],
                thunks: &ids[nfuncs..nfuncs + nthunks],
                closures: &ids[nfuncs + nthunks..nfuncs + nthunks + nclosures],
                fnvals: &ids[nfuncs + nthunks + nclosures..nfuncs + nthunks + nclosures + nfnvals],
                strings: &strings,
                std: &std_ids,
                std_sigs: &program.std_sigs,
            };
            function::translate(f, &mut ctx.func, &mut fb_ctx, &mut env)
                .map_err(|e| format!("in `{}`: {e}", f.name))?;
        }
        module
            .define_function(ids[i], &mut ctx)
            .map_err(|e| format!("in `{}`: {e:?}", f.name))?;
        module.clear_context(&mut ctx);
    }

    // The C entry point: run the script, then let the runtime finish.
    let mut ctx = module.make_context();
    ctx.func.signature.returns.push(AbiParam::new(types::I32));
    let main_id = module
        .declare_function("main", Linkage::Export, &ctx.func.signature)
        .map_err(|e| e.to_string())?;
    {
        let mut b = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = b.create_block();
        b.switch_to_block(entry);
        let init = module.declare_func_in_func(rt.init, b.func);
        b.ins().call(init, &[]);
        let script = module.declare_func_in_func(ids[program.entry], b.func);
        b.ins().call(script, &[]);
        let fin = module.declare_func_in_func(rt.finish, b.func);
        b.ins().call(fin, &[]);
        let zero = b.ins().iconst(types::I32, 0);
        b.ins().return_(&[zero]);
        b.seal_all_blocks();
        b.finalize();
    }
    module
        .define_function(main_id, &mut ctx)
        .map_err(|e| format!("in `main`: {e:?}"))?;

    module.finish().emit().map_err(|e| e.to_string())
}
