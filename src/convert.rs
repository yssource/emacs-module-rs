use std::ptr;
use std::cell::RefCell;
use std::sync::{Mutex, RwLock};
use std::ffi::CString;
use libc;

use super::{Env, Value, Result};
use super::{ToLisp, FromLisp, IntoLisp, Transfer};

impl FromLisp for i64 {
    fn from_lisp(value: &Value) -> Result<Self> {
        raw_call!(value.env, extract_integer, value.raw)
    }
}

impl FromLisp for String {
    // TODO: Optimize this.
    fn from_lisp(value: &Value) -> Result<Self> {
        let bytes = value.env.string_bytes(value)?;
        // FIX
        Ok(String::from_utf8(bytes).unwrap())
    }
}

impl ToLisp for i64 {
    fn to_lisp<'e>(&self, env: &'e Env) -> Result<Value<'e>> {
        raw_call_value!(env, make_integer, *self)
    }
}

// TODO: Make this more elegant. Can't implement it for trait bound Into<Vec<u8>>, since that would
// complain about conflicting implementations for i64.
impl ToLisp for str {
    fn to_lisp<'e>(&self, env: &'e Env) -> Result<Value<'e>> {
        let cstring = CString::new(self)?;
        let ptr = cstring.as_ptr();
        raw_call_value!(env, make_string, ptr, libc::strlen(ptr) as libc::ptrdiff_t)
    }
}

impl<'e> IntoLisp<'e> for Value<'e> {
    fn into_lisp(self, _env: &'e Env) -> Result<Value> {
        Ok(self)
    }
}

impl<'e> IntoLisp<'e> for i64 {
    fn into_lisp(self, env: &Env) -> Result<Value> {
        self.to_lisp(env)
    }
}

impl<'e> IntoLisp<'e> for String {
    fn into_lisp(self, env: &Env) -> Result<Value> {
        self.to_lisp(env)
    }
}

impl<'e> IntoLisp<'e> for () {
    fn into_lisp(self, env: &Env) -> Result<Value> {
        env.intern("nil")
    }
}

impl<'e, T: Transfer> IntoLisp<'e> for Box<T> {
    fn into_lisp(self, env: &'e Env) -> Result<Value> {
        let raw = Box::into_raw(self);
        let ptr = raw as *mut libc::c_void;
        raw_call_value!(env, make_user_ptr, Some(T::finalizer), ptr)
    }
}

enable_transfers! {
    RefCell;
    Mutex;
    RwLock;
}

fn strip_trailing_zero_bytes(bytes: &mut Vec<u8>) {
    let mut len = bytes.len();
    while len > 0 && bytes[len - 1] == 0 {
        bytes.pop(); // strip trailing 0-byte(s)
        len -= 1;
    }
}

/// Implementation details.
impl Env {
    fn string_bytes(&self, value: &Value) -> Result<Vec<u8>> {
        let mut len: isize = 0;
        let mut bytes = unsafe {
            let copy_string_contents = raw_fn!(self, copy_string_contents)?;
            let ok: bool = self.handle_exit(copy_string_contents(
                self.raw, value.raw, ptr::null_mut(), &mut len))?;
            // Technically this shouldn't happen, and the return type of copy_string_contents
            // should be void, not bool. TODO: Use a custom error type instead of panicking here.
            if !ok {
                panic!("Emacs failed to give string's length but did not raise a signal");
            }

            let mut bytes = vec![0u8; len as usize];
            let ok: bool = self.handle_exit(copy_string_contents(
                self.raw, value.raw, bytes.as_mut_ptr() as *mut i8, &mut len))?;
            // Technically this shouldn't happen, and the return type of copy_string_contents
            // should be void, not bool. TODO: Use a custom error type instead of panicking here.
            if !ok {
                panic!("Emacs failed to copy string but did not raise a signal");
            }
            bytes
        };
        strip_trailing_zero_bytes(&mut bytes);
        Ok(bytes)
    }
}
