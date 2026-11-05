//! `math.*` runtime functions. They all work on floats; Python-style.

#[no_mangle]
pub extern "C" fn kode_math_sqrt(x: f64) -> f64 {
    x.sqrt()
}

#[no_mangle]
pub extern "C" fn kode_math_abs(x: f64) -> f64 {
    x.abs()
}

#[no_mangle]
pub extern "C" fn kode_math_floor(x: f64) -> f64 {
    x.floor()
}

#[no_mangle]
pub extern "C" fn kode_math_ceil(x: f64) -> f64 {
    x.ceil()
}

#[no_mangle]
pub extern "C" fn kode_math_round(x: f64) -> f64 {
    x.round()
}

#[no_mangle]
pub extern "C" fn kode_math_sin(x: f64) -> f64 {
    x.sin()
}

#[no_mangle]
pub extern "C" fn kode_math_cos(x: f64) -> f64 {
    x.cos()
}

#[no_mangle]
pub extern "C" fn kode_math_tan(x: f64) -> f64 {
    x.tan()
}

#[no_mangle]
pub extern "C" fn kode_math_log(x: f64) -> f64 {
    x.ln()
}

#[no_mangle]
pub extern "C" fn kode_math_exp(x: f64) -> f64 {
    x.exp()
}

#[no_mangle]
pub extern "C" fn kode_math_pow(a: f64, b: f64) -> f64 {
    a.powf(b)
}

#[no_mangle]
pub extern "C" fn kode_math_min(a: f64, b: f64) -> f64 {
    a.min(b)
}

#[no_mangle]
pub extern "C" fn kode_math_max(a: f64, b: f64) -> f64 {
    a.max(b)
}
