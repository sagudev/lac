#[cfg(all(target_os = "linux", target_arch = "x86"))]
pub const BIN_FILE: &[u8] = include_bytes!("../bin/l32/LAC");
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const BIN_FILE: &[u8] = include_bytes!("../bin/l64/LAC");
#[cfg(all(target_os = "windows", target_arch = "x86"))]
pub const BIN_FILE: &[u8] = include_bytes!("../bin/win32/LAC.exe");
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const BIN_FILE: &[u8] = include_bytes!("../bin/win64/LAC.exe");
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub const BIN_FILE: &[u8] = include_bytes!("../bin/mac64/LAC");

#[cfg(target_os = "windows")]
pub const BIN_EXE: &str = "LAC.exe";
#[cfg(not(target_os = "windows"))]
pub const BIN_EXE: &str = "LAC";
