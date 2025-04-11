#[derive(Debug, Default)]
pub struct Process<'a> {
    pub name: &'a str,
    pub threads: &'a [Thread<'a>],
    pub memories: &'a [Memory],
    pub data: &'a [u8],
}

#[derive(Debug)]
pub struct Memory {
    pub min: u64,
    pub max: Option<u64>,
}

#[derive(Debug)]
pub struct Thread<'a> {
    pub name: &'a str,
    pub frames: &'a [Frame],
}

#[derive(Debug)]
pub struct Frame {
    pub func_idx: u32,
    pub code_offset: u32,
    // TODO: locals
    // TODO: stack
}

use alloc::vec::Vec;

const HEADER: [u8; 8] = [
    0x00, 0x61, 0x73, 0x6D, // Magic
    0x01, 0x00, 0x00, 0x00, // Version
];

/// Convert the given `proc` info into a coredump and write it into `w`.
///
/// ## Example
///
///     let frame = Frame {
///         func_idx: 6,
///         code_offset: 123,
///     };
///     let thread = Thread {
///         name: "main",
///         frames: &[frame],
///     };
///     let proc = Process {
///         name: "/usr/bin/true.exe",
///         threads: &[thread],
///         memories: &[Memory { min: 0, max: None }],
///         data: &[],
///     };
///     let mut coredump = Vec::new();
///     serialize(&mut coredump, &proc);
pub fn serialize(w: &mut Vec<u8>, proc: &Process) {
    w.extend(HEADER);
    write_core_section(w, proc);
    write_corestack_sections(w, proc);
    write_memory_section(w, proc);
    write_data_section(w, proc);
}

fn write_core_section(w: &mut Vec<u8>, proc: &Process) {
    let mut data = Vec::new();
    data.push(0x0);
    write_utf8(&mut data, proc.name);
    w.push(0); // custom section ID
    encode_custom_section(w, "core", &data);
}

fn write_corestack_sections(w: &mut Vec<u8>, proc: &Process) {
    for stack in proc.threads {
        let mut data = Vec::new();
        encode_stack(&mut data, stack);
        w.push(0); // custom section ID
        encode_custom_section(w, "corestack", &data);
    }
}

fn write_memory_section(w: &mut Vec<u8>, proc: &Process) {
    let mut section: Vec<u8> = Vec::new();
    for mem in proc.memories {
        let mut flags = 0;
        if mem.max.is_some() {
            flags |= 0b0001;
        }
        section.push(flags);
        write_u64(&mut section, mem.min);
        if let Some(max) = mem.max {
            write_u64(&mut section, max);
        }
    }
    w.push(5); // memory section ID
    let count = proc.memories.len();
    encode_section(w, count as u32, &section);
}

fn write_data_section(w: &mut Vec<u8>, proc: &Process) {
    let mut section = Vec::new();
    section.push(0x00); // active
    section.push(0x41); // "i32.const" instruction
    write_u64(&mut section, 0); // i32.const value (zero)
    section.push(0x0B); // "end" instruction.
    write_u64(&mut section, proc.data.len() as u64);
    section.extend(proc.data);

    w.push(11); // data section ID
    encode_section(w, 1, &section);
}

fn encode_custom_section(w: &mut Vec<u8>, name: &str, data: &[u8]) {
    let encoded_name_len = encoding_size(u32::try_from(name.len()).unwrap());
    let total_size = encoded_name_len + name.len() + data.len();
    write_u64(w, total_size as u64);
    write_u64(w, name.len() as u64);
    w.extend_from_slice(name.as_bytes());
    w.extend(data);
}

fn encode_section(w: &mut Vec<u8>, count: u32, bytes: &[u8]) {
    let size = encoding_size(count) + bytes.len();
    write_u64(w, size as u64);
    write_u64(w, count.into());
    w.extend(bytes);
}

fn encoding_size(n: u32) -> usize {
    let mut buf = Vec::new();
    write_u64(&mut buf, n.into())
}

fn encode_stack(w: &mut Vec<u8>, stack: &Thread) {
    w.push(0x0); // version 0
    write_utf8(w, stack.name);
    write_u64(w, stack.frames.len() as u64);
    for frame in stack.frames {
        w.push(0x0); // version 0
        write_u64(w, frame.func_idx as u64);
        write_u64(w, frame.code_offset as u64);
        write_u64(w, 0); // locals vec size
        write_u64(w, 0); // stack vec size
    }
}

// Encode a UTF-8 string in wasm format.
fn write_utf8(w: &mut Vec<u8>, v: &str) {
    let bytes = v.as_bytes();
    write_u64(w, bytes.len() as u64);
    w.extend_from_slice(bytes);
}

// Encode u64 value using leb128 encoding.
fn write_u64(w: &mut Vec<u8>, mut val: u64) -> usize {
    const CONTINUATION_BIT: u8 = 1 << 7;
    let mut bytes_written = 0;
    loop {
        let byte = val & (u8::MAX as u64);
        let mut byte = byte as u8 & !CONTINUATION_BIT;
        val >>= 7;
        if val != 0 {
            byte |= CONTINUATION_BIT;
        }
        let buf = [byte];
        w.extend_from_slice(&buf);
        bytes_written += 1;
        if val == 0 {
            return bytes_written;
        }
    }
}
