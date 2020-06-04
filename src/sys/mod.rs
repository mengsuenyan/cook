// super::cfg_if! {
//     if #[cfg(target_os = "windows")] {
//         mod windows;
//     } else if #[cfg(target_os = "linux")] {
//         mod linux;
//     } else {
//         compile_error!("Just only support the platform of windows or linux");
//     }
// }

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "linux")]
mod linux;