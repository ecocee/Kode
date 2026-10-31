use crate::io::flush;

#[no_mangle]
pub extern "C" fn kode_div_zero() -> ! {
    flush();
    eprintln!("kode: runtime error: division by zero");
    std::process::exit(1);
}

#[no_mangle]
pub extern "C" fn kode_assert_fail(line: i64) -> ! {
    flush();
    eprintln!("kode: assertion failed at line {line}");
    std::process::exit(1);
}
