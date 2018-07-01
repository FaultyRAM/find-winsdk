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
#![cfg_attr(feature = "cargo-clippy", deny(clippy_correctness))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_perf))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_style))]

#[macro_use]
extern crate serde_derive;
extern crate winreg;

use std::env;
use std::ffi::OsStr;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use winreg::enums::{KEY_WOW64_32KEY, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE};
use winreg::RegKey;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
/// Information about a Windows SDK installation.
pub struct SdkInfo {
    installation_folder: PathBuf,
    product_name: Option<String>,
    product_version: String,
}

impl SdkInfo {
    /// Returns installation information for a Windows SDK installation, trying multiple locations
    /// until one is found.
    ///
    /// This method looks for installations in the following order before giving up:
    ///
    /// 1. Environment-defined installation;
    /// 2. Windows 10 SDK;
    /// 3. Windows 8.1 SDK.
    pub fn any() -> io::Result<Option<Self>> {
        Ok(Self::winsdk_env())
            .and_then(|opt| {
                if opt.is_none() {
                    Self::winsdk_10()
                } else {
                    Ok(opt)
                }
            })
            .and_then(|opt| {
                if opt.is_none() {
                    Self::winsdk_8_1()
                } else {
                    Ok(opt)
                }
            })
    }

    /// Returns installation information for a Windows SDK from environment variables, if present.
    pub fn winsdk_env() -> Option<Self> {
        env::var_os("WindowsSdkDir")
            .and_then(|install_dir| {
                env::var_os("WindowsSdkVersion").map(|version| (install_dir, version))
            })
            .map(|(install_dir, version)| {
                let ver = version
                    .into_string()
                    .map(|s| {
                        s.split(r".0\")
                            .next()
                            .expect("`str::split` failed")
                            .to_owned()
                    })
                    .expect("`WindowsSdkVersion` was not valid UTF-8");
                Self {
                    installation_folder: Path::new(&install_dir).to_owned(),
                    product_name: None,
                    product_version: ver,
                }
            })
    }

    /// Returns installation information for the Windows 10 SDK, if present.
    ///
    /// This queries the registry for installation information, and may fail if an error occurs
    /// while accessing the registry.
    pub fn winsdk_10() -> io::Result<Option<Self>> {
        Self::get_info(r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v10.0")
    }

    /// Returns installation information for the Windows 8.1 SDK, if present.
    ///
    /// This queries the registry for installation information, and may fail if an error occurs
    /// while accessing the registry.
    pub fn winsdk_8_1() -> io::Result<Option<Self>> {
        Self::get_info(r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v8.1")
    }

    fn get_info<P: AsRef<OsStr>>(subkey: P) -> io::Result<Option<Self>> {
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags(subkey, KEY_QUERY_VALUE | KEY_WOW64_32KEY)
            .map(|key| {
                // If deserialization fails, the key might not have been deleted correctly.
                if let Ok(info) = key.decode() {
                    Some(info)
                } else {
                    None
                }
            })
            .or_else(|e| {
                if e.kind() == ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(e)
                }
            })
    }

    /// Returns the filesystem path to where a Windows SDK instance is installed.
    pub fn installation_folder(&self) -> &Path {
        &self.installation_folder
    }

    /// Returns the human-readable name of a Windows SDK instance.
    pub fn product_name(&self) -> Option<&str> {
        self.product_name.as_ref().map(|s| s.as_ref())
    }

    /// Returns the version number of a Windows SDK instance.
    pub fn product_version(&self) -> &str {
        &self.product_version
    }
}

#[cfg(test)]
mod tests {
    use SdkInfo;

    #[test]
    fn any() {
        let _ = SdkInfo::any()
            .expect("could not retrieve Windows SDK info from registry")
            .expect("Windows SDK is not installed");
    }

    #[test]
    fn winsdk_env() {
        let _ =
            SdkInfo::winsdk_env().expect("environment does not specify a Windows SDK installation");
    }

    #[test]
    fn winsdk_10() {
        let _ = SdkInfo::winsdk_10()
            .expect("could not retrieve Windows 10 SDK info from registry")
            .expect("Windows 10 SDK is not installed");
    }

    #[test]
    fn winsdk_8_1() {
        let _ = SdkInfo::winsdk_8_1()
            .expect("could not retrieve Windows 8.1 SDK info from registry")
            .expect("Windows 8.1 SDK is not installed");
    }
}
