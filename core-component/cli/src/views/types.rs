#[derive(Clone)]
pub enum BoldType {
    ULTRA,
    NONE,
    NORMAL,
    SOME,
    BITMORE,
}

#[derive(Clone)]
pub enum ITALLICType {
    ULTRA,
    NONE,
    NORMAL,
    SOME,
    BITMORE,
}

#[derive(Clone)]
pub enum Color {
    RGB(u8, u8, u8),
    HEX(String),
    RGBA(u8, u8, u8),
    TRANSPARENT,
    DEFAULT,
    RANDOM,
}

#[derive(Clone)]
pub enum AllignmentType {
    CENTER,
    LEFT,
    RIGHT,
}
