use core::fmt::Display;

use owo_colors::{colors::*, OwoColorize};

use crate::dtree::DeviceTree;

pub trait KSay {
    const NAME: &'static str;

    /// Function does not take in a self parameter to avoid side effects and to allow kprinting in cases of failed initalization of a subsystem
    fn kprint(message: impl Display) {
        println!("[{}]: {}", Self::NAME.fg::<Yellow>(), message);
    }
}

pub trait Init {
    fn init(&self, dtree: &DeviceTree);
}
