//! Maps with string keys, in insertion order:
//! `[rc][kind = MAP | flags][pointer to MapInner]`.

use std::collections::HashMap;

use crate::error::fatal;
use crate::list::{kode_list_new, kode_list_push};
use crate::object::{
    alloc_object, free_object, kode_release, kode_retain, Header, ITEMS_ARE_REFS, MAP,
};
use crate::string::bytes;

struct MapInner {
    keys: Vec<*mut Header>,
    vals: Vec<u64>,
    index: HashMap<Vec<u8>, usize>,
}

unsafe fn inner<'a>(p: *const Header) -> &'a mut MapInner {
    &mut **((p as *const *mut MapInner).add(2))
}

/// Key/value pairs in insertion order. The pointers are borrowed: they are valid
/// while the map is alive and no item is removed.
pub(crate) unsafe fn entries(p: *const Header) -> Vec<(*mut Header, u64)> {
    let m = inner(p);
    m.keys.iter().copied().zip(m.vals.iter().copied()).collect()
}

unsafe fn is_refs(p: *const Header) -> bool {
    (*p).kind & ITEMS_ARE_REFS != 0
}

#[no_mangle]
pub extern "C" fn kode_map_new(items_are_refs: i64) -> *mut Header {
    unsafe {
        let kind = MAP
            | if items_are_refs != 0 {
                ITEMS_ARE_REFS
            } else {
                0
            };
        let p = alloc_object(24, kind);
        let b = Box::new(MapInner {
            keys: Vec::new(),
            vals: Vec::new(),
            index: HashMap::new(),
        });
        *((p as *mut *mut MapInner).add(2)) = Box::into_raw(b);
        p
    }
}

pub unsafe fn drop_map(p: *mut Header) {
    let refs = is_refs(p);
    let b = Box::from_raw(*((p as *mut *mut MapInner).add(2)));
    for k in &b.keys {
        kode_release(*k);
    }
    if refs {
        for v in &b.vals {
            kode_release(*v as *mut Header);
        }
    }
    drop(b);
    free_object(p, 24);
}

/// Insert or replace. The map retains the key and takes ownership of `raw`
/// if its values are heap values.
///
/// # Safety
/// `p` must be a valid map and `key` a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_map_set(p: *mut Header, key: *mut Header, raw: i64) {
    let m = inner(p);
    let kb = bytes(key).to_vec();
    match m.index.get(&kb) {
        Some(&i) => {
            let old = m.vals[i];
            m.vals[i] = raw as u64;
            if is_refs(p) {
                kode_release(old as *mut Header);
            }
        }
        None => {
            kode_retain(key);
            m.index.insert(kb, m.keys.len());
            m.keys.push(key);
            m.vals.push(raw as u64);
        }
    }
}

/// # Safety
/// `p` must be a valid map and `key` a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_map_get(p: *mut Header, key: *mut Header, line: i64) -> i64 {
    let m = inner(p);
    match m.index.get(bytes(key)) {
        Some(&i) => m.vals[i] as i64,
        None => fatal(&format!(
            "key \"{}\" is not in the map at line {line}",
            String::from_utf8_lossy(bytes(key))
        )),
    }
}

/// # Safety
/// `p` must be a valid map and `key` a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_map_has(p: *mut Header, key: *mut Header) -> i64 {
    inner(p).index.contains_key(bytes(key)) as i64
}

/// # Safety
/// `p` must be a valid map.
#[no_mangle]
pub unsafe extern "C" fn kode_map_len(p: *mut Header) -> i64 {
    inner(p).keys.len() as i64
}

/// # Safety
/// `p` must be a valid map and `i` in range.
#[no_mangle]
pub unsafe extern "C" fn kode_map_key_at(p: *mut Header, i: i64) -> *mut Header {
    inner(p).keys[i as usize]
}

/// # Safety
/// `p` must be a valid map and `i` in range.
#[no_mangle]
pub unsafe extern "C" fn kode_map_val_at(p: *mut Header, i: i64) -> i64 {
    inner(p).vals[i as usize] as i64
}

/// A new list holding the keys.
///
/// # Safety
/// `p` must be a valid map.
#[no_mangle]
pub unsafe extern "C" fn kode_map_keys(p: *mut Header) -> *mut Header {
    let list = kode_list_new(1);
    for k in &inner(p).keys {
        kode_retain(*k);
        kode_list_push(list, *k as i64);
    }
    list
}

/// A new list holding the values, in insertion order. Heap values are retained.
///
/// # Safety
/// `p` must be a valid map.
#[no_mangle]
pub unsafe extern "C" fn kode_map_values(p: *mut Header) -> *mut Header {
    let refs = is_refs(p);
    let list = kode_list_new(refs as i64);
    for v in &inner(p).vals {
        if refs {
            kode_retain(*v as *mut Header);
        }
        kode_list_push(list, *v as i64);
    }
    list
}

/// Remove a key (and release its value). Missing keys are ignored.
///
/// # Safety
/// `p` must be a valid map and `key` a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_map_remove(p: *mut Header, key: *mut Header) {
    let m = inner(p);
    let Some(i) = m.index.remove(bytes(key)) else {
        return;
    };
    let k = m.keys.remove(i);
    let v = m.vals.remove(i);
    kode_release(k);
    if is_refs(p) {
        kode_release(v as *mut Header);
    }
    for pos in m.index.values_mut() {
        if *pos > i {
            *pos -= 1;
        }
    }
}
