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

const V10_0_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v10.0";
const V8_1A_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v8.1A";
const V8_1_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v8.1";
const V8_0A_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v8.0A";
const V8_0_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v8.0";
const V7_1A_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v7.1A";
const V7_1_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v7.1";
const V7_0A_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v7.0a";
const V7_0_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v7.0";
const V6_1A_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v6.1a";
const V6_1_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v6.1";
const V6_0A_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v6.0a";
const V6_0_REG_KEY: &str = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v6.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Windows SDK versions.
///
/// Note that prior to v10.0, each Windows SDK came in two flavours; one with an `A` suffix in its
/// version number, and one without. For convenience, this crate detects both, preferring the `A`
/// suffix version if present.
pub enum SdkVersion {
    /// Any Windows SDK version.
    ///
    /// This is either one specified by environment variables or, if that is not available, the
    /// latest version found in the registry.
    Any,
    /// A Windows SDK installation specified by environment variables.
    Env,
    /// The Windows 10.0 SDK.
    V10_0,
    /// The Windows 8.1 SDK.
    V8_1,
    /// The Windows 8.0 SDK.
    V8_0,
    /// The Windows 7.1 SDK.
    V7_1,
    /// The Windows 7.0 SDK.
    V7_0,
    /// The Windows 6.1 SDK.
    V6_1,
    /// The Windows 6.0 SDK.
    V6_0,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
/// Information about a Windows SDK installation.
pub struct SdkInfo {
    installation_folder: PathBuf,
    product_name: Option<String>,
    product_version: String,
}

impl SdkInfo {
    /// Returns installation information for a Windows SDK installation.
    ///
    /// If `SdkVersion::Any` is specified, this method will first query environment variables, then
    /// search the registry for the latest Windows SDK recognised by this crate. If a specific
    /// version is specified, this method will only look for that version before giving up.
    pub fn find(version: SdkVersion) -> io::Result<Option<Self>> {
        match version {
            SdkVersion::Any => {
                use SdkVersion::*;
                let vers = [Env, V10_0, V8_1, V8_0, V7_1, V7_0, V6_1, V6_0];
                for res in vers.iter().map(|v| Self::find(*v)) {
                    match res {
                        Ok(None) => (),
                        _ => return res,
                    }
                }
                Ok(None)
            }
            SdkVersion::Env => Ok(Self::query_env()),
            SdkVersion::V10_0 => Self::query_reg(V10_0_REG_KEY),
            SdkVersion::V8_1 => Self::find_double_release((V8_1A_REG_KEY, V8_1_REG_KEY)),
            SdkVersion::V8_0 => Self::find_double_release((V8_0A_REG_KEY, V8_0_REG_KEY)),
            SdkVersion::V7_1 => Self::find_double_release((V7_1A_REG_KEY, V7_1_REG_KEY)),
            SdkVersion::V7_0 => Self::find_double_release((V7_0A_REG_KEY, V7_0_REG_KEY)),
            SdkVersion::V6_1 => Self::find_double_release((V6_1A_REG_KEY, V6_1_REG_KEY)),
            SdkVersion::V6_0 => Self::find_double_release((V6_0A_REG_KEY, V6_0_REG_KEY)),
        }
    }

    fn find_double_release(keys: (&str, &str)) -> io::Result<Option<Self>> {
        let res = Self::query_reg(keys.0);
        match res {
            Ok(None) => Self::query_reg(keys.1),
            _ => res,
        }
    }

    /// Returns installation information for a Windows SDK from environment variables, if present.
    fn query_env() -> Option<Self> {
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

    fn query_reg<P: AsRef<OsStr>>(subkey: P) -> io::Result<Option<Self>> {
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
    use {SdkInfo, SdkVersion};

    #[test]
    fn any() {
        let _ = SdkInfo::find(SdkVersion::Any)
            .expect("could not retrieve Windows SDK info from registry")
            .expect("Windows SDK is not installed");
    }

    #[test]
    fn winsdk_env() {
        let _ = SdkInfo::find(SdkVersion::Env)
            .unwrap_or_else(|_| unreachable!())
            .expect("environment does not specify a Windows SDK installation");
    }

    #[test]
    fn winsdk_10_0() {
        let _ = SdkInfo::find(SdkVersion::V10_0)
            .expect("could not retrieve Windows 10 SDK info from registry")
            .expect("Windows 10 SDK is not installed");
    }

    #[test]
    fn winsdk_8_1() {
        let _ = SdkInfo::find(SdkVersion::V8_1)
            .expect("could not retrieve Windows 8.1 SDK info from registry")
            .expect("Windows 8.1 SDK is not installed");
    }
}
