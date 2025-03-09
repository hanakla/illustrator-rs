#![doc = include_str!("../README.md")]


mod plugin_base;
mod externs;
mod ai_plugin;


pub use illustrator_sys as ai_sys;
pub use plugin_base::define_plugin;
pub use ai_plugin::AIPlugin;
pub use plugin_base;
