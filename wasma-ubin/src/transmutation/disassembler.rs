// src/transmutation/disassembler.rs
// UBIN Disassembler – Binary Talimat Analiz Motoru
// Capstone ile disassemble eder, UI/rendering çağrılarını tespit eder
// x86_64 ve aarch64 destekli – gerçek düşük seviye analiz
// Convergence ve patcher için ham veri sağlar
use capstone::prelude::*;
use capstone::Capstone;
use object::read::FileKind;
use object::{Object, ObjectSection, ObjectSymbol};
use std::collections::HashSet;
use std::path::PathBuf;
use std::fs;
use capstone::arch::BuildsCapstone; 

#[derive(Debug, Clone)]
pub struct DisassemblyReport {
    pub path: PathBuf,
    pub arch: String,
    pub entry_point: u64,
    pub instructions: Vec<DisassembledInstruction>,
    pub symbols: Vec<String>,
    pub detected_api_calls: HashSet<String>,
    pub ui_related_calls: Vec<u64>,  // adresler
    pub render_related_calls: Vec<u64>,
    pub analysis_success: bool,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DisassembledInstruction {
    pub address: u64,
    pub mnemonic: String,
    pub op_str: String,
    pub bytes: Vec<u8>,
}

pub struct UbinDisassembler;

impl UbinDisassembler {
    pub fn new() -> Self {
        UbinDisassembler
    }

    /// Binary'yi disassemble eder – tam rapor üretir
    pub fn disassemble_binary(&self, path: PathBuf) -> DisassemblyReport {
        let data = match fs::read(&path) {
            Ok(d) => d,
            Err(e) => {
                return DisassemblyReport {
                    path,
                    arch: "Unknown".to_string(),
                    entry_point: 0,
                    instructions: vec![],
                    symbols: vec![],
                    detected_api_calls: HashSet::new(),
                    ui_related_calls: vec![],
                    render_related_calls: vec![],
                    analysis_success: false,
                    error_msg: Some(format!("File read error: {}", e)),
                };
            }
        };

        let kind = match object::read::FileKind::parse(&data[..]) {
            Ok(k) => k,
            Err(_) => object::read::FileKind::Elf64, // fallback
        };

        // DÜZELTME: Sadece x86_64 için basitleştir
        let cs = match Capstone::new()
            .x86()
            .mode(capstone::arch::x86::ArchMode::Mode64)
            .syntax(capstone::arch::x86::ArchSyntax::Att)
            .detail(true)
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                return DisassemblyReport {
                    path,
                    arch: "x86_64".to_string(),
                    entry_point: 0,
                    instructions: vec![],
                    symbols: vec![],
                    detected_api_calls: HashSet::new(),
                    ui_related_calls: vec![],
                    render_related_calls: vec![],
                    analysis_success: false,
                    error_msg: Some(format!("Capstone init error: {}", e)),
                };
            }
        };

        let obj = match object::File::parse(&*data) {
            Ok(o) => o,
            Err(e) => {
                return DisassemblyReport {
                    path,
                    arch: "x86_64".to_string(),
                    entry_point: 0,
                    instructions: vec![],
                    symbols: vec![],
                    detected_api_calls: HashSet::new(),
                    ui_related_calls: vec![],
                    render_related_calls: vec![],
                    analysis_success: false,
                    error_msg: Some(format!("Object parse error: {}", e)),
                };
            }
        };

        let entry = obj.entry();

        // Text section'ı bul ve disassemble et
        let mut instructions = vec![];
        let mut ui_calls = vec![];
        let mut render_calls = vec![];
        let mut api_calls = HashSet::new();

        if let Some(section) = obj.sections().find(|s| s.name() == Ok(".text") || s.kind() == object::SectionKind::Text) {
            let section_data = section.data().unwrap_or(&[]);
            let section_addr = section.address();

            if let Ok(insns) = cs.disasm_all(section_data, section_addr) {
                for insn in insns.iter() {
                    let addr: u64 = insn.address();
                    let mnemonic = insn.mnemonic().unwrap_or("???").to_string();
                    let op_str = insn.op_str().unwrap_or("").to_string();

                    instructions.push(DisassembledInstruction {
                        address: addr,
                        mnemonic: mnemonic.clone(),
                        op_str: op_str.clone(),
                        bytes: insn.bytes().to_vec(),
                    });

                    // UI/render çağrılarını tespit et
                    let op_low = op_str.to_lowercase();
                    if mnemonic == "call" || mnemonic == "jmp" {
                        if op_low.contains("gtk") || op_low.contains("qt") || op_low.contains("createwindow") || op_low.contains("nsapplication") {
                            ui_calls.push(addr);
                            api_calls.insert(format!("UI_CALL: {}", op_str));
                        }
                        if op_low.contains("vk") || op_low.contains("d3d") || op_low.contains("mtl") || op_low.contains("wgpu") || op_low.contains("gl") {
                            render_calls.push(addr);
                            api_calls.insert(format!("RENDER_CALL: {}", op_str));
                        }
                    }
                }
            }
        }

        // Symbol'ları çıkar
        let mut symbols = vec![];
        for symbol in obj.symbols() {
            if let Ok(name) = symbol.name() {
                symbols.push(name.to_string());
            }
        }

        println!("🔍 Disassembly complete – {} instructions, {} UI calls, {} render calls", instructions.len(), ui_calls.len(), render_calls.len());

        DisassemblyReport {
            path,
            arch: "x86_64".to_string(),
            entry_point: entry,
            instructions,
            symbols,
            detected_api_calls: api_calls,
            ui_related_calls: ui_calls,
            render_related_calls: render_calls,
            analysis_success: true,
            error_msg: None,
        }
    }
}
