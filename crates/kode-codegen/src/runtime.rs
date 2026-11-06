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
    pub init: FuncId,
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
        RtFn::Repr => ("kode_repr", vec![I, I], Some(I)),
        RtFn::StrLen => ("kode_str_len", vec![I], Some(I)),
        RtFn::ListNew => ("kode_list_new", vec![I], Some(I)),
        RtFn::ListPush => ("kode_list_push", vec![I, I], None),
        RtFn::ListGet => ("kode_list_get", vec![I, I, I], Some(I)),
        RtFn::ListSet => ("kode_list_set", vec![I, I, I, I], None),
        RtFn::ListLen => ("kode_list_len", vec![I], Some(I)),
        RtFn::ListNewPacked => ("kode_list_new_packed", vec![], Some(I)),
        RtFn::ListPush8 => ("kode_list_push8", vec![I, I], None),
        RtFn::ListGet8 => ("kode_list_get8", vec![I, I, I], Some(I)),
        RtFn::ListSet8 => ("kode_list_set8", vec![I, I, I, I], None),
        RtFn::ListSlice => ("kode_list_slice", vec![I, I, I], Some(I)),
        RtFn::ListSlice8 => ("kode_list_slice8", vec![I, I, I], Some(I)),
        RtFn::MapNew => ("kode_map_new", vec![I], Some(I)),
        RtFn::MapSet => ("kode_map_set", vec![I, I, I], None),
        RtFn::MapGet => ("kode_map_get", vec![I, I, I], Some(I)),
        RtFn::MapHas => ("kode_map_has", vec![I, I], Some(I)),
        RtFn::MapLen => ("kode_map_len", vec![I], Some(I)),
        RtFn::MapKeyAt => ("kode_map_key_at", vec![I, I], Some(I)),
        RtFn::MapValAt => ("kode_map_val_at", vec![I, I], Some(I)),
        RtFn::MapKeys => ("kode_map_keys", vec![I], Some(I)),
        RtFn::ObjNew => ("kode_obj_new", vec![I, I], Some(I)),
        RtFn::ObjNewClass => ("kode_obj_new_class", vec![I, I, I], Some(I)),
        RtFn::ErrThrow => ("kode_err_throw", vec![I], None),
        RtFn::ErrPending => ("kode_err_pending", vec![], Some(I)),
        RtFn::ErrTake => ("kode_err_take", vec![], Some(I)),
        RtFn::Uncaught => ("kode_uncaught", vec![], None),
        RtFn::UnwrapFail => ("kode_unwrap_fail", vec![I], None),
        RtFn::Exit => ("kode_exit", vec![I], None),
        RtFn::ListOob => ("kode_list_oob", vec![I, I, I], None),
        RtFn::ChanNew => ("kode_chan_new", vec![], Some(I)),
        RtFn::ChanSend => ("kode_chan_send", vec![I, I, I], None),
        RtFn::ChanRecv => ("kode_chan_recv", vec![I], Some(I)),
        RtFn::ChanClose => ("kode_chan_close", vec![I], None),
    }
}

/// Runtime functions that return 0 or 1 as a 64-bit integer; the compiler turns it into a bool.
pub(crate) fn returns_bool(f: RtFn) -> bool {
    matches!(f, RtFn::StrEq | RtFn::MapHas | RtFn::ErrPending)
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
    RtFn::Repr,
    RtFn::StrLen,
    RtFn::ListNew,
    RtFn::ListPush,
    RtFn::ListGet,
    RtFn::ListSet,
    RtFn::ListLen,
    RtFn::ListNewPacked,
    RtFn::ListPush8,
    RtFn::ListGet8,
    RtFn::ListSet8,
    RtFn::ListSlice,
    RtFn::ListSlice8,
    RtFn::MapNew,
    RtFn::MapSet,
    RtFn::MapGet,
    RtFn::MapHas,
    RtFn::MapLen,
    RtFn::MapKeyAt,
    RtFn::MapValAt,
    RtFn::MapKeys,
    RtFn::ObjNew,
    RtFn::ObjNewClass,
    RtFn::ErrThrow,
    RtFn::ErrPending,
    RtFn::ErrTake,
    RtFn::Uncaught,
    RtFn::UnwrapFail,
    RtFn::Exit,
    RtFn::ListOob,
    RtFn::ChanNew,
    RtFn::ChanSend,
    RtFn::ChanRecv,
    RtFn::ChanClose,
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
        init: import(module, "kode_rt_init", &[], None)?,
    })
}