use std::sync::OnceLock;

static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

fn parse_debug_flag() -> bool {
    std::env::args().any(|arg| arg == "-d" || arg == "--debug")
}

pub fn init() {
    let _ = DEBUG_ENABLED.set(parse_debug_flag());
}

pub fn debug_enabled() -> bool {
    *DEBUG_ENABLED.get_or_init(parse_debug_flag)
}
