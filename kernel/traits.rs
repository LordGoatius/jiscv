use core::fmt::Display;

use owo_colors::{OwoColorize, colors::*};

use crate::dtree::DeviceTree;

pub trait KSay {
    const NAME: &'static str;

    fn kprint(message: impl Display) {
        println!("[{}]: {}", Self::NAME.fg::<Yellow>(), message);
    }
}

pub trait Init {
    fn init(&self, dtree: &DeviceTree);
}
