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
    // Strip inline comments, early-out for comment/empty lines
    let line = line.split(';').next().unwrap_or("").trim();
    if line.is_empty() {
        return vec![];
    }

    // Split into instruction and the rest at the first whitespace boundary.
    // This ensures dashes inside instruction names (e.g. `jump-zero`) are not split.
    let (instruction, rest) = match line.split_once(|c: char| c.is_whitespace()) {
        Some((instr, rest)) => (instr, rest.trim()),
        None => (line, ""),
    };

    let (mut opcode, has_param) = get_opcode(instruction);

    // Split args on both whitespace and '-' in a single pass.
    // e.g. "7-15-1" and "7 15 1" and "7-15 1" all produce ["7","15","1"].
    let args: Vec<&str> = rest
        .split(|c: char| c.is_whitespace() || c == '-')
        .filter(|s| !s.is_empty())
        .collect();

    // Check for parameter
    if !has_param {
        if !args.is_empty() {
            panic!("Instruction {} does not take parameters", instruction);
        }
        return vec![opcode];
    }
    if args.is_empty() {
        panic!("Instruction {} requires a parameter", instruction);
    }
    if args.len() > 8 {
        panic!("Instruction {} has too many parameters", instruction);
    }
    // Set opcode's last 3 bits to the number of arg bytes
    opcode |= args.len() as u8;

    let mut bytes = vec![opcode];
    for arg in &args {
        bytes.push(arg.parse::<u8>().unwrap());
    }
    bytes
}
