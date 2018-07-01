// Copyright (c) 2018 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides support for detecting Windows SDK installations.

#![cfg(target_os = "windows")]
#![forbid(warnings)]
#![forbid(future_incompatible)]
#![deny(unused)]
#![forbid(box_pointers)]
#![forbid(missing_copy_implementations)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![forbid(trivial_casts)]
#![forbid(trivial_numeric_casts)]
#![forbid(unsafe_code)]
#![forbid(unused_import_braces)]
#![deny(unused_qualifications)]
#![forbid(unused_results)]
#![forbid(variant_size_differences)]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_cargo))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_complexity))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_correctness))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_perf))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_style))]

extern crate winreg;

#[cfg(test)]
mod tests {
    #[test]
    fn enumerate_values() {
        use std::str;
        use winreg::enums::{KEY_WOW64_32KEY, HKEY_LOCAL_MACHINE, KEY_READ};
        use winreg::RegKey;

        if let Err(e) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags(
                r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v8.1",
                KEY_READ | KEY_WOW64_32KEY,
            )
            .and_then(|winsdk81_key| {
                for value in winsdk81_key.enum_values() {
                    let v = value?;
                    println!(
                        "{}: {}",
                        v.0,
                        str::from_utf8(&v.1.bytes).expect("reg value is not valid UTF-8")
                    );
                }
                Ok(())
            }) {
            panic!("{}", e);
        }
    }
}
