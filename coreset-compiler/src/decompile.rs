pub fn decompile(bytes: &[u8]) -> String {
    let mut source = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let opcode = bytes[i] >> 3;
        let args = (bytes[i] & 0b111) as usize;
        i += 1;
        let instruction = match opcode {
            0 => "halt",
            1 => "jump",
            2 => "jump-zero",
            3 => "jump-not-zero",
            4 => "jump-reg",
            5 => "jump-ind",
            6 => "line",
            7 => "load",
            8 => "read",
            9 => "write",
            10 => "readind",
            11 => "writeind",
            12 => "incr",
            13 => "decr",
            14 => "neg",
            15 => "not",
            16 => "add",
            17 => "sub",
            18 => "mul",
            19 => "div",
            20 => "and",
            21 => "or",
            22 => "xor",
            23 => "lshift",
            24 => "rshift",
            25 => "andi",
            26 => "ori",
            27 => "xori",
            28 => "comp",
            _ => "unknown",
        };
        source.push_str(instruction);
        for _ in 0..args {
            if i >= bytes.len() {
                break;
            }
            source.push_str(&format!(" {}", bytes[i]));
            i += 1;
        }
        source.push('\n');
    }
    source
}
