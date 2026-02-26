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

// ---------------------------------------------------------------------------
// Function definition metadata
// ---------------------------------------------------------------------------

struct FuncDef {
    /// Memory address of the `retad` binding (where the return address is stored).
    retad_cell: Option<String>,
    /// Memory address of the `callad` binding (where the call-block return address is stored).
    callad_cell: Option<String>,
    /// All `let` bindings declared in the function header (name -> address).
    bindings: std::collections::HashMap<String, String>,
    /// Instruction index of the first instruction after `label body` (set in pass 1).
    body_ip: Option<usize>,
}

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Scan `source` and return the set of all declared function names.
/// Used by the IDE gutter so it can detect call-block openers.
pub fn scan_function_names(source: &str) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();
    for raw in source.lines() {
        let stripped = raw.trim().split(';').next().unwrap_or("").trim();
        if let Some(rest) = stripped.strip_prefix("func ") {
            let name = rest.trim();
            if !name.is_empty() {
                names.insert(name.to_string());
            }
        }
    }
    names
}

/// Returns the number of bytecode instructions a source line compiles to.
/// `func_names` is the set of declared function names (needed to detect call-block
/// openers and closers correctly).
///
/// - `func <name>`          → 0  (marker only)
/// - `label <name>`         → 0  (plain label)
/// - `label body`           → 1  (hidden `ji callad` inserted before it)
/// - `<funcname>` alone     → 3  (load callad_start + write callad + jump func_entry)
/// - `call` alone           → 3  (load retip + write retad + jump func_body)
/// - `ret`                  → 1  (ji retad)
/// - `let n a v`            → 2  (load + write)
/// - `let n a`              → 1  (write)
/// - everything else        → 1
pub fn source_line_instruction_count(
    trimmed: &str,
    func_names: &std::collections::HashSet<String>,
) -> usize {
    let stripped = trimmed.split(';').next().unwrap_or("").trim();
    if stripped.is_empty() {
        return 0;
    }
    let (kw, rest) = match stripped.split_once(|c: char| c.is_whitespace()) {
        Some((k, r)) => (k, r.trim()),
        None => (stripped, ""),
    };
    match kw {
        "func" => 0,
        "label" => {
            if rest == "body" {
                1
            } else {
                0
            }
        }
        "ret" => 1,
        // call-block opener: load + write + jump
        name if rest.is_empty() && func_names.contains(name) => 3,
        // call-block closer: load + write + jump
        "call" if rest.is_empty() => 3,
        "let" => {
            let n = stripped.split_whitespace().count();
            if n >= 4 { 2 } else { 1 }
        }
        _ => 1,
    }
}

// ---------------------------------------------------------------------------
// Compiler
// ---------------------------------------------------------------------------

pub fn compile(source: &str) -> Vec<u8> {
    // ---- Pre-pass: collect function definitions ----
    // For each `func name`, gather the `let` bindings that appear in the header
    // (before `label body`) so the compiler knows retad/callad addresses up front.
    let mut functions: std::collections::HashMap<String, FuncDef> =
        std::collections::HashMap::new();
    {
        let mut current_func: Option<String> = None;
        let mut collecting_lets = false;
        for raw in source.lines() {
            let stripped = raw.trim().split(';').next().unwrap_or("").trim();
            if stripped.is_empty() {
                continue;
            }
            let (kw, rest) = match stripped.split_once(|c: char| c.is_whitespace()) {
                Some((k, r)) => (k, r.trim()),
                None => (stripped, ""),
            };
            if kw == "func" {
                let name = rest.trim().to_string();
                functions.insert(
                    name.clone(),
                    FuncDef {
                        retad_cell: None,
                        callad_cell: None,
                        bindings: std::collections::HashMap::new(),
                        body_ip: None,
                    },
                );
                current_func = Some(name);
                collecting_lets = true;
            } else if kw == "let" && collecting_lets {
                if let Some(ref fname) = current_func {
                    let tokens: Vec<&str> = rest.split_whitespace().collect();
                    if tokens.len() >= 2 {
                        let bname = tokens[0];
                        let addr = tokens[1].to_string();
                        let def = functions.get_mut(fname).unwrap();
                        if bname == "retad" {
                            def.retad_cell = Some(addr.clone());
                        } else if bname == "callad" {
                            def.callad_cell = Some(addr.clone());
                        }
                        def.bindings.insert(bname.to_string(), addr);
                    }
                }
            } else if kw == "label" && rest == "body" {
                // `label body` ends the header
                collecting_lets = false;
            } else if kw != "label" && kw != "func" {
                collecting_lets = false;
            }
        }
    }

    let func_names: std::collections::HashSet<String> = functions.keys().cloned().collect();

    // ---- Pass 1: build label/function map (name -> instruction index) ----
    // Also resolves `body_ip` for each function.
    let mut labels: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    {
        let mut ip = 0usize;
        let mut in_call_block = false;
        let mut current_func_p1: Option<String> = None;
        for raw in source.lines() {
            let stripped = raw.trim().split(';').next().unwrap_or("").trim();
            if stripped.is_empty() {
                continue;
            }
            let (kw, rest) = match stripped.split_once(|c: char| c.is_whitespace()) {
                Some((k, r)) => (k, r.trim()),
                None => (stripped, ""),
            };
            if kw == "func" {
                let name = rest.trim().to_string();
                labels.insert(name.clone(), ip);
                current_func_p1 = Some(name);
            } else if kw == "label" {
                if rest == "body" {
                    if let Some(ref fname) = current_func_p1 {
                        // Hidden `ji callad` occupies one instruction before body
                        ip += 1;
                        functions.get_mut(fname).unwrap().body_ip = Some(ip);
                    } else {
                        panic!("'label body' used outside of a function");
                    }
                    labels.insert("body".to_string(), ip);
                } else {
                    labels.insert(rest.trim().to_string(), ip);
                }
            } else if func_names.contains(kw) && rest.is_empty() {
                if in_call_block {
                    panic!("Nested call blocks are not allowed");
                }
                in_call_block = true;
                // call-block opener emits 3 instructions
                ip += 3;
            } else if kw == "call" && rest.is_empty() {
                if !in_call_block {
                    panic!("'call' with no open function block");
                }
                // call-block closer emits 3 instructions
                ip += 3;
                in_call_block = false;
            } else {
                ip += source_line_instruction_count(stripped, &func_names);
            }
        }
        if in_call_block {
            panic!("Unclosed call block: missing 'call'");
        }
    }

    // ---- Pass 2: emit bytecode ----
    let mut bytes: Vec<u8> = Vec::new();
    let mut variables: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    // call-block state: (func_name, ip_at_opener, body_bytes, body_instr_count)
    let mut call_block: Option<(String, usize, Vec<u8>, usize)> = None;
    // which func we are compiling inside (for `ret` and `label body`)
    let mut current_func_context: Option<String> = None;
    // running instruction pointer
    let mut ip = 0usize;

    for raw in source.lines() {
        let stripped = raw.trim().split(';').next().unwrap_or("").trim();
        if stripped.is_empty() {
            continue;
        }
        let (kw, rest) = match stripped.split_once(|c: char| c.is_whitespace()) {
            Some((k, r)) => (k, r.trim()),
            None => (stripped, ""),
        };

        match kw {
            "func" => {
                current_func_context = Some(rest.trim().to_string());
                // Zero instructions emitted
            }
            "label" => {
                if rest == "body" {
                    // Must be inside a function
                    let fname = current_func_context
                        .as_deref()
                        .unwrap_or_else(|| panic!("'label body' used outside of a function"));
                    let callad = functions
                        .get(fname)
                        .and_then(|d| d.callad_cell.as_deref())
                        .unwrap_or_else(|| {
                            panic!("Function '{}' has no 'let callad' declaration", fname)
                        });
                    // Emit hidden `ji callad` — jumps back into the call block
                    let b = compile_line(&format!("ji {}", callad));
                    push_bytes(&mut bytes, &mut call_block, b, 1);
                    ip += 1;
                }
                // All labels are zero-instruction markers; addresses set in pass 1
            }
            "ret" => {
                let fname = current_func_context
                    .as_deref()
                    .unwrap_or_else(|| panic!("'ret' used outside of a function body"));
                let retad = functions
                    .get(fname)
                    .and_then(|d| d.retad_cell.as_deref())
                    .unwrap_or_else(|| {
                        panic!("Function '{}' has no 'let retad' declaration", fname)
                    });
                let b = compile_line(&format!("ji {}", retad));
                push_bytes(&mut bytes, &mut call_block, b, 1);
                ip += 1;
            }
            "let" => {
                let tokens: Vec<&str> = rest.split_whitespace().collect();
                if tokens.len() < 2 || tokens.len() > 3 {
                    panic!("'let' requires 2 or 3 arguments: name, location, [initial_value]");
                }
                let name = tokens[0].to_string();
                let location = tokens[1].to_string();
                variables.insert(name, location.clone());
                if tokens.len() == 3 {
                    let mut v = compile_line(&format!("load {}", tokens[2]));
                    v.extend(compile_line(&format!("write {}", location)));
                    push_bytes(&mut bytes, &mut call_block, v, 2);
                    ip += 2;
                } else {
                    let v = compile_line(&format!("write {}", location));
                    push_bytes(&mut bytes, &mut call_block, v, 1);
                    ip += 1;
                }
            }
            name if func_names.contains(name) && rest.is_empty() => {
                // Call-block opener
                if call_block.is_some() {
                    panic!("Nested call blocks are not allowed");
                }
                // Bring the function's bindings into variable scope
                if let Some(def) = functions.get(name) {
                    for (bname, addr) in &def.bindings {
                        variables.insert(bname.clone(), addr.clone());
                    }
                }
                // Store ip_at_opener; the 3 opener instructions will be emitted at `call`
                call_block = Some((name.to_string(), ip, Vec::new(), 0));
                // Advance ip past the 3 opener instructions so inner code sees correct ip
                ip += 3;
            }
            "call" if rest.is_empty() => {
                let (fname, ip_at_opener, body_bytes, body_count) = call_block
                    .take()
                    .unwrap_or_else(|| panic!("'call' with no open function block"));
                let func_def = functions
                    .get(&fname)
                    .unwrap_or_else(|| panic!("Unknown function '{}'", fname));
                let retad = func_def.retad_cell.as_deref().unwrap_or_else(|| {
                    panic!("Function '{}' has no 'let retad' declaration", fname)
                });
                let callad = func_def.callad_cell.as_deref().unwrap_or_else(|| {
                    panic!("Function '{}' has no 'let callad' declaration", fname)
                });
                let func_entry_ip = labels[&fname];
                let body_ip = func_def
                    .body_ip
                    .unwrap_or_else(|| panic!("Function '{}' has no 'label body'", fname));

                // call_block_body_start is the instruction right after the 3 opener instructions
                let call_block_body_start = ip_at_opener + 3;
                // return_ip is right after the 3 closer instructions
                let return_ip = ip_at_opener + 3 + body_count + 3;

                // Opener: load call_block_body_start, write callad, jump func_entry
                bytes.extend(compile_line(&format!("load {}", call_block_body_start)));
                bytes.extend(compile_line(&format!("write {}", callad)));
                bytes.extend(compile_line(&format!("jump {}", func_entry_ip)));
                // Body
                bytes.extend(body_bytes);
                // Closer: load return_ip, write retad, jump func_body
                bytes.extend(compile_line(&format!("load {}", return_ip)));
                bytes.extend(compile_line(&format!("write {}", retad)));
                bytes.extend(compile_line(&format!("jump {}", body_ip)));

                ip += 3; // closer (opener+body already accounted for)
            }
            _ => {
                // Regular instruction — substitute variables and labels
                let sub: Vec<String> = stripped
                    .split_whitespace()
                    .skip(1)
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
                let full = if sub.is_empty() {
                    kw.to_string()
                } else {
                    format!("{} {}", kw, sub.join(" "))
                };
                let b = compile_line(&full);
                push_bytes(&mut bytes, &mut call_block, b, 1);
                ip += 1;
            }
        }
    }

    if let Some((fname, _, _, _)) = call_block {
        panic!(
            "Unclosed call block for function '{}': missing 'call'",
            fname
        );
    }

    bytes
}

/// Push `b` into the call-block body buffer when inside a call block,
/// or directly into the main output otherwise.
/// `count` is the number of bytecode instructions in `b`.
fn push_bytes(
    out: &mut Vec<u8>,
    call_block: &mut Option<(String, usize, Vec<u8>, usize)>,
    b: Vec<u8>,
    count: usize,
) {
    if let Some((_, _, body, c)) = call_block {
        *c += count;
        body.extend(b);
    } else {
        out.extend(b);
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
