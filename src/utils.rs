use macroquad::input::{KeyCode, MouseButton};

// map from LWJGL key to
#[allow(clippy::too_many_lines)]
pub const fn map_key(key: i32) -> KeyCode {
    #[allow(clippy::enum_glob_use)]
    use macroquad::input::KeyCode::*;

    match key {
        32 => Space,
        39 => Apostrophe, /* ' */
        44 => Comma,      /* , */
        45 => Minus,      /* - */
        46 => Period,     /* . */
        47 => Slash,      /* / */
        48 => Key0,
        49 => Key1,
        50 => Key2,
        51 => Key3,
        52 => Key4,
        53 => Key5,
        54 => Key6,
        55 => Key7,
        56 => Key8,
        57 => Key9,
        59 => Semicolon, /* ; */
        61 => Equal,     /* = */
        65 => A,
        66 => B,
        67 => C,
        68 => D,
        69 => E,
        70 => F,
        71 => G,
        72 => H,
        73 => I,
        74 => J,
        75 => K,
        76 => L,
        77 => M,
        78 => N,
        79 => O,
        80 => P,
        81 => Q,
        82 => R,
        83 => S,
        84 => T,
        85 => U,
        86 => V,
        87 => W,
        88 => X,
        89 => Y,
        90 => Z,
        91 => LeftBracket,  /* [ */
        92 => Backslash,    /* \ */
        93 => RightBracket, /* ] */
        96 => GraveAccent,  /* ` */
        161 => World1,      /* non-US #1 */
        162 => World2,      /* non-US #2 */
        256 => Escape,
        257 => Enter,
        258 => Tab,
        259 => Backspace,
        260 => Insert,
        261 => Delete,
        262 => Right,
        263 => Left,
        264 => Down,
        265 => Up,
        266 => PageUp,
        267 => PageDown,
        268 => Home,
        269 => End,
        280 => CapsLock,
        281 => ScrollLock,
        282 => NumLock,
        283 => PrintScreen,
        284 => Pause,
        290 => F1,
        291 => F2,
        292 => F3,
        293 => F4,
        294 => F5,
        295 => F6,
        296 => F7,
        297 => F8,
        298 => F9,
        299 => F10,
        300 => F11,
        301 => F12,
        302 => F13,
        303 => F14,
        304 => F15,
        305 => F16,
        306 => F17,
        307 => F18,
        308 => F19,
        309 => F20,
        310 => F21,
        311 => F22,
        312 => F23,
        313 => F24,
        314 => F25,
        320 => Kp0,
        321 => Kp1,
        322 => Kp2,
        323 => Kp3,
        324 => Kp4,
        325 => Kp5,
        326 => Kp6,
        327 => Kp7,
        328 => Kp8,
        329 => Kp9,
        330 => KpDecimal,
        331 => KpDivide,
        332 => KpMultiply,
        333 => KpSubtract,
        334 => KpAdd,
        335 => KpEnter,
        336 => KpEqual,
        340 => LeftShift,
        341 => LeftControl,
        342 => LeftAlt,
        343 => LeftSuper,
        344 => RightShift,
        345 => RightControl,
        346 => RightAlt,
        347 => RightSuper,
        348 => Menu,
        _ => Unknown,
    }
}

pub const fn map_button(button: i32) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        _ => MouseButton::Unknown,
    }
}
