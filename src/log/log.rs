use std::fs::File;
// Structure capable of holding different types for our
pub enum LogElement {
    Char(char),
    Integer(i32),
    LongInteger(i64),
    LongLongInteger(i128),
    UnsignedInteger(u32),
    UnsignedLongInteger(u64),
    UnsignedLongLongInteger(u128),
    Float(f32),  // has single percision in rust
    Double(f64), // has double percision in rust
}

const LOG_QUEUE_SIZE: usize = 8 * 1024 * 1024;

pub struct Logger {
    file_name: String,
    file: File,
}
