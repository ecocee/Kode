//! `StringBuilder` (ADR 0044): `[rc][kind = BUFFER][length][capacity][data pointer]`.
//!
//! `push` copies the bytes of a string into the buffer and `build` hands back one
//! new string, so a program that writes IR text pays one grow per push instead of
//! one new string per `+`. Because the bytes are copied, the builder holds no
//! references of its own and freeing it only frees its byte vector.

use std::mem::ManuallyDrop;

use crate::object::{alloc_object, free_object, Header, BUFFER};
use crate::string;

const SIZE: usize = 40;

struct Raw(*mut Header);

impl Raw {
    // Raw pointers, not `&mut`, for the same reason as `list.rs`: two of these
    // are used in one expression and two `&mut i64` into one object would alias.
    unsafe fn len(&self) -> *mut i64 {
        (self.0 as *mut i64).add(2)
    }
    unsafe fn cap(&self) -> *mut i64 {
        (self.0 as *mut i64).add(3)
    }
    unsafe fn ptr(&self) -> *mut *mut u8 {
        (self.0 as *mut *mut u8).add(4)
    }
    /// Run `f` on the bytes as a `Vec`, then store it back.
    unsafe fn with_vec<R>(&self, f: impl FnOnce(&mut Vec<u8>) -> R) -> R {
        let mut v = ManuallyDrop::new(Vec::from_raw_parts(
            *self.ptr(),
            *self.len() as usize,
            *self.cap() as usize,
        ));
        let r = f(&mut v);
        *self.ptr() = v.as_mut_ptr();
        *self.len() = v.len() as i64;
        *self.cap() = v.capacity() as i64;
        r
    }
}

#[no_mangle]
pub extern "C" fn kode_buffer_new() -> *mut Header {
    unsafe {
        let p = alloc_object(SIZE, BUFFER);
        let r = Raw(p);
        let v = ManuallyDrop::new(Vec::<u8>::new());
        *r.ptr() = v.as_ptr() as *mut u8;
        *r.len() = 0;
        *r.cap() = 0;
        p
    }
}

pub unsafe fn drop_buffer(p: *mut Header) {
    let r = Raw(p);
    drop(Vec::from_raw_parts(
        *r.ptr(),
        *r.len() as usize,
        *r.cap() as usize,
    ));
    free_object(p, SIZE);
}

/// # Safety
/// `p` must be a valid builder and `s` a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_buffer_push(p: *mut Header, s: *mut Header) {
    let b = string::bytes(s);
    Raw(p).with_vec(|v| v.extend_from_slice(b));
}

/// The text built so far, as a new string.
///
/// # Safety
/// `p` must be a valid builder.
#[no_mangle]
pub unsafe extern "C" fn kode_buffer_build(p: *mut Header) -> *mut Header {
    Raw(p).with_vec(|v| string::from_bytes(v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::kode_release;

    #[test]
    fn pushes_are_copied_in_order() {
        unsafe {
            let b = kode_buffer_new();
            for piece in [&b"kode"[..], b"_", b"main"] {
                let s = string::from_bytes(piece);
                kode_buffer_push(b, s);
                kode_release(s);
            }
            let text = kode_buffer_build(b);
            assert_eq!(string::bytes(text), b"kode_main");
            kode_release(text);
            kode_release(b);
        }
    }

    #[test]
    fn an_empty_builder_builds_an_empty_string() {
        unsafe {
            let b = kode_buffer_new();
            let text = kode_buffer_build(b);
            assert_eq!(string::bytes(text), b"");
            kode_release(text);
            kode_release(b);
        }
    }
}
