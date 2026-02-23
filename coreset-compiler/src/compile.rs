pub fn get_opcode(instruction: &str) -> (u8, bool) {
    let (opcode, has_param): (u8, bool) = match instruction {
        "halt" | "hlt" => (0, false),
        "jump" => (1, true),
        "jumpz" | "jump-zero" | "jz" => (2, true),
        "jumpnz" | "jump-not-zero" | "jnz" => (3, true),
        "jumpreg" | "jump-reg" | "jr" => (4, false),
        "jumpi" | "jump-ind" | "ji" => (5, true),
        "line" | "ln" => (6, false),
        "load" => (7, true),
        "read" => (8, true),
        "write" => (9, true),
        "readi" | "read-ind" | "ri" => (10, true),
        "writei" | "write-ind" | "wi" => (11, true),
        "incr" => (12, false),
        "decr" => (13, false),
        "neg" => (14, false),
        "not" => (15, false),
        "add" => (16, true),
        "sub" => (17, true),
        "mul" => (18, true),
        "div" => (19, true),
        "and" => (20, true),
        "or" => (21, true),
        "xor" => (22, true),
        "lshift" | "lsh" => (23, false),
        "rshift" | "rsh" => (24, false),
        "andi" => (25, true),
        "ori" => (26, true),
        "xori" => (27, true),
        "comp" => (28, true),
        // TODO: error handling
        _ => panic!("Unknown instruction: {}", instruction),
    };
    (opcode << 3, has_param)
}

pub fn compile(source: &str) -> Vec<u8> {
    // Get lines into a vector
    let lines: Vec<&str> = source
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();
    // Init return vector
    let mut bytes = Vec::new();
    // Compile each line
    for line in lines {
        bytes.extend(compile_line(line));
    }
    bytes
}

pub fn compile_line(line: &str) -> Vec<u8> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Vec::new();
    }
    let (mut opcode, has_param) = get_opcode(parts[0]);
    let args = parts.len() - 1;
    // Check for parameter
    if !has_param {
        if args > 0 {
            panic!("Instruction {} does not take parameters", parts[0]);
        }
        return vec![opcode];
    }
    if args < 1 {
        panic!("Instruction {} requires a parameter", parts[0]);
    }
    if args > 8 {
        panic!("Instruction {} has too many parameters", parts[0]);
    }
    // Set opcode's last 3 bits to the number of parameters
    opcode |= args as u8;

    let mut bytes = vec![opcode];
    for part in parts.iter().skip(1) {
        bytes.push(part.parse::<u8>().unwrap());
    }
    bytes
}
