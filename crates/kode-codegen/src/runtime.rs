//! The functions that `kode-rt` provides, and how to call them.

use std::collections::HashMap;

use cranelift_codegen::ir::{types, AbiParam};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::ObjectModule;
use kode_mir::RtFn;

pub(crate) struct Rt {
    pub funcs: HashMap<RtFn, FuncId>,
    pub retain: FuncId,
    pub release: FuncId,
    pub div_zero: FuncId,
    pub finish: FuncId,
}

const I: types::Type = types::I64;
const F: types::Type = types::F64;

/// symbol name, parameter types, result type
pub(crate) fn info(f: RtFn) -> (&'static str, Vec<types::Type>, Option<types::Type>) {
    match f {
        RtFn::PrintInt => ("kode_print_int", vec![I], None),
        RtFn::PrintFloat => ("kode_print_float", vec![F], None),
        RtFn::PrintBool => ("kode_print_bool", vec![I], None),
        RtFn::PrintStr => ("kode_print_str", vec![I], None),
        RtFn::PrintSep => ("kode_print_sep", vec![], None),
        RtFn::PrintNl => ("kode_print_nl", vec![], None),
        RtFn::StrConcat => ("kode_str_concat", vec![I, I], Some(I)),
        RtFn::StrEq => ("kode_str_eq", vec![I, I], Some(I)),
        RtFn::IntToStr => ("kode_int_to_str", vec![I], Some(I)),
        RtFn::FloatToStr => ("kode_float_to_str", vec![F], Some(I)),
        RtFn::BoolToStr => ("kode_bool_to_str", vec![I], Some(I)),
        RtFn::StrLen => ("kode_str_len", vec![I], Some(I)),
        RtFn::AssertFail => ("kode_assert_fail", vec![I], None),
    }
}

const ALL: &[RtFn] = &[
    RtFn::PrintInt,
    RtFn::PrintFloat,
    RtFn::PrintBool,
    RtFn::PrintStr,
    RtFn::PrintSep,
    RtFn::PrintNl,
    RtFn::StrConcat,
    RtFn::StrEq,
    RtFn::IntToStr,
    RtFn::FloatToStr,
    RtFn::BoolToStr,
    RtFn::StrLen,
    RtFn::AssertFail,
];

fn import(
    module: &mut ObjectModule,
    name: &str,
    params: &[types::Type],
    ret: Option<types::Type>,
) -> Result<FuncId, String> {
    let mut sig = module.make_signature();
    for p in params {
        sig.params.push(AbiParam::new(*p));
    }
    if let Some(r) = ret {
        sig.returns.push(AbiParam::new(r));
    }
    module
        .declare_function(name, Linkage::Import, &sig)
        .map_err(|e| e.to_string())
}

pub(crate) fn declare(module: &mut ObjectModule) -> Result<Rt, String> {
    let mut funcs = HashMap::new();
    for f in ALL {
        let (name, params, ret) = info(*f);
        funcs.insert(*f, import(module, name, &params, ret)?);
    }
    Ok(Rt {
        funcs,
        retain: import(module, "kode_retain", &[I], None)?,
        release: import(module, "kode_release", &[I], None)?,
        div_zero: import(module, "kode_div_zero", &[], None)?,
        finish: import(module, "kode_rt_finish", &[], None)?,
    })
}
