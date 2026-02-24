pub fn get_opcode(instruction: &str) -> (u8, u8) {
    let (opcode, min_param): (u8, u8) = match instruction {
        "halt" | "hlt" => (0, 0),
        "jump" => (1, 1),
        "jumpz" | "jump-zero" | "jz" => (2, 1),
        "jumpnz" | "jump-not-zero" | "jnz" => (3, 1),
        "jumpreg" | "jump-reg" | "jr" => (4, 0),
        "jumpi" | "jump-ind" | "ji" => (5, 1),
        "line" | "ln" => (6, 0),
        "load" => (7, 1),
        "read" => (8, 1),
        "write" => (9, 1),
        "readi" | "read-ind" | "ri" => (10, 1),
        "writei" | "write-ind" | "wi" => (11, 1),
        "incr" => (12, 0),
        "decr" => (13, 0),
        "neg" => (14, 0),
        "not" => (15, 0),
        "add" => (16, 1),
        "sub" => (17, 1),
        "mul" => (18, 1),
        "div" => (19, 1),
        "and" => (20, 1),
        "or" => (21, 1),
        "xor" => (22, 1),
        "lshift" | "lsh" => (23, 0),
        "rshift" | "rsh" => (24, 0),
        "andi" => (25, 1),
        "ori" => (26, 1),
        "xori" => (27, 1),
        "comp" => (28, 1),
        "compi" => (29, 1),
        // TODO: error handling
        _ => panic!("Unknown instruction: {}", instruction),
    };
    (opcode << 3, min_param)
}

/// Returns the number of bytecode instructions a (trimmed, non-empty, non-comment) source line
/// compiles to. Used by both the compiler and the IDE gutter.
pub fn source_line_instruction_count(trimmed: &str) -> usize {
    // label definitions emit no instructions
    if trimmed.starts_with("label ") || trimmed == "label" {
        return 0;
    }
    // 3-arg let compiles to load + write
    if trimmed.starts_with("let ") || trimmed == "let" {
        let tokens: Vec<&str> = trimmed.splitn(4, char::is_whitespace).collect();
        if tokens.len() >= 4 {
            return 2;
        }
    }
    1
}

pub fn compile(source: &str) -> Vec<u8> {
    // ---- Pass 1: build label map (label name -> instruction index) ----
    let mut labels: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut instr_counter = 0usize;
    for raw in source.lines() {
        let trimmed = raw.trim();
        let stripped = trimmed.split(';').next().unwrap_or("").trim();
        if stripped.is_empty() {
            continue;
        }
        if stripped.starts_with("label ") {
            let name = stripped["label ".len()..].trim().to_string();
            labels.insert(name, instr_counter);
        } else {
            instr_counter += source_line_instruction_count(stripped);
        }
    }

    // ---- Pass 2: compile each line ----
    let lines: Vec<&str> = source
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();
    let mut bytes = Vec::new();
    let mut variables: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for line in lines {
        bytes.extend(compile_line_with_vars(line, &mut variables, &labels));
    }
    bytes
}

fn compile_line_with_vars(
    line: &str,
    variables: &mut std::collections::HashMap<String, String>,
    labels: &std::collections::HashMap<String, usize>,
) -> Vec<u8> {
    // Strip inline comments, early-out for comment/empty lines
    let line = line.split(';').next().unwrap_or("").trim();
    if line.is_empty() {
        return vec![];
    }

    let (instruction, rest) = match line.split_once(|c: char| c.is_whitespace()) {
        Some((instr, rest)) => (instr, rest.trim()),
        None => (line, ""),
    };

    // Label definitions compile to nothing
    if instruction == "label" {
        return vec![];
    }

    if instruction == "let" {
        // Syntax: let <name> <location> [<initial_value>]
        // Tokenise by whitespace only so that "0-1" stays intact as a location
        let tokens: Vec<&str> = rest.split_whitespace().collect();
        if tokens.len() < 2 || tokens.len() > 3 {
            panic!("'let' requires 2 or 3 arguments: name, location, [initial_value]");
        }
        let name = tokens[0].to_string();
        let location = tokens[1].to_string();
        variables.insert(name, location.clone());

        if tokens.len() == 3 {
            // Emit: load <value>  then  write <location>
            let mut result = compile_line(&format!("load {}", tokens[2]));
            result.extend(compile_line(&format!("write {}", location)));
            result
        } else {
            // Emit: write <location>  (stores current register value)
            compile_line(&format!("write {}", location))
        }
    } else {
        // Substitute variable names and label names in argument tokens
        let substituted_tokens: Vec<String> = rest
            .split_whitespace()
            .map(|token| {
                if let Some(loc) = variables.get(token) {
                    loc.clone()
                } else if let Some(idx) = labels.get(token) {
                    idx.to_string()
                } else {
                    token.to_string()
                }
            })
            .collect();
        let substituted_rest = substituted_tokens.join(" ");

        let substituted_line = if substituted_rest.is_empty() {
            instruction.to_string()
        } else {
            format!("{} {}", instruction, substituted_rest)
        };
        compile_line(&substituted_line)
    }
}

pub fn args_to_byte(args: Vec<&str>) -> Vec<u8> {
    let mut bytes = Vec::new();
    for arg in args {
        let parsed = arg.parse::<u64>().expect("Failed to parse argument as u64");
        let space: usize = match parsed {
            0..=0xff => 1,
            0x100..=0xffff => 2,
            0x10000..=0xffffff => 3,
            0x1000000..=0xffffffff => 4,
            0x100000000..=0xffffffffff => 5,
            0x10000000000..=0xffffffffffff => 6,
            0x1000000000000..=0xffffffffffffff => 7,
            _ => 8,
        };
        // Write exactly `space` bytes in big-endian order, preserving leading zeros.
        for shift in (0..space).rev() {
            bytes.push(((parsed >> (shift * 8)) & 0xff) as u8);
        }
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

    let (mut opcode, min_params) = get_opcode(instruction);

    // Split args on both whitespace and '-' in a single pass.
    // e.g. "7-15-1" and "7 15 1" and "7-15 1" all produce ["7","15","1"].
    let args: Vec<&str> = rest
        .split(|c: char| c.is_whitespace() || c == '-')
        .filter(|s| !s.is_empty())
        .collect();

    // Check for parameter
    if min_params == 0 {
        if !args.is_empty() {
            panic!("Instruction {} does not take parameters", instruction);
        }
        return vec![opcode];
    }
    if args.len() < min_params as usize {
        panic!(
            "Instruction {} requires {} parameter(s)",
            instruction, min_params
        );
    }
    if args.len() > 8 {
        panic!("Instruction {} has too many parameters", instruction);
    }
    let arg_bytes = args_to_byte(args);
    // Set opcode's last 3 bits to the number of arg bytes
    opcode |= arg_bytes.len() as u8;

    let mut bytes = vec![opcode];
    bytes.extend(arg_bytes);
    bytes
}
