pub fn decompile(bytes: &[u8]) -> String {
    let mut source = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let opcode = bytes[i] >> 3;
        let args = (bytes[i] & 0b111) as usize;
        i += 1;
        // Arg types:
        // 0 = no args
        // 1 = data
        // 2 = address
        // 3 = instruction number
        let (instruction, arg_type) = match opcode {
            0 => ("halt", 0),
            1 => ("jump", 3),
            2 => ("jump-zero", 3),
            3 => ("jump-not-zero", 3),
            4 => ("jump-reg", 0),
            5 => ("jump-ind", 2),
            6 => ("line", 0),
            7 => ("load", 1),
            8 => ("read", 2),
            9 => ("write", 2),
            10 => ("readind", 2),
            11 => ("writeind", 2),
            12 => ("incr", 0),
            13 => ("decr", 0),
            14 => ("neg", 0),
            15 => ("not", 0),
            16 => ("add", 2),
            17 => ("sub", 2),
            18 => ("mul", 2),
            19 => ("div", 2),
            20 => ("and", 2),
            21 => ("or", 2),
            22 => ("xor", 2),
            23 => ("lshift", 0),
            24 => ("rshift", 0),
            25 => ("andi", 1),
            26 => ("ori", 1),
            27 => ("xori", 1),
            28 => ("comp", 2),
            29 => ("compi", 1),
            _ => ("unknown", 0),
        };
        source.push_str(instruction);

        let mut arg_bytes = Vec::new();
        for _ in 0..args {
            if i >= bytes.len() {
                break;
            }
            arg_bytes.push(bytes[i]);
            i += 1;
        }

        if !arg_bytes.is_empty() {
            match arg_type {
                1 | 3 => {
                    // Data or instruction number: combine bytes into a single little-endian number
                    let mut value: u64 = 0;
                    for (j, &b) in arg_bytes.iter().enumerate() {
                        value |= (b as u64) << (j * 8);
                    }
                    source.push_str(&format!(" {}", value));
                }
                2 => {
                    // Address: first byte separated by a dash from the rest, or only first byte
                    let first = arg_bytes[0];
                    if arg_bytes.len() == 1 {
                        source.push_str(&format!(" {}", first));
                    } else {
                        let mut rest: u64 = 0;
                        for (j, &b) in arg_bytes[1..].iter().enumerate() {
                            rest |= (b as u64) << (j * 8);
                        }
                        source.push_str(&format!(" {}-{}", first, rest));
                    }
                }
                _ => {
                    for b in &arg_bytes {
                        source.push_str(&format!(" {}", b));
                    }
                }
            }
        }

        source.push('\n');
    }
    source
}
