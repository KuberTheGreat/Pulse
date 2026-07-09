//! ANSI escape-code constants for terminal colours and styles.

pub const RESET:   &str = "\x1b[0m";
pub const BOLD:    &str = "\x1b[1m";
pub const DIM:     &str = "\x1b[2m";

pub const RED:     &str = "\x1b[31m";
pub const GREEN:   &str = "\x1b[32m";
pub const YELLOW:  &str = "\x1b[33m";
pub const BLUE:    &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN:    &str = "\x1b[36m";
pub const WHITE:   &str = "\x1b[37m";

pub const BG_RED:    &str = "\x1b[41m";
pub const BG_GREEN:  &str = "\x1b[42m";
pub const BG_YELLOW: &str = "\x1b[43m";
