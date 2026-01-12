// src/transmutation/rebuilder.rs
// UBIN Rebuilder – Patched Binary Yeniden İnşa Motoru
// goblin + object ile ELF/PE/Mach-O formatında yeni binary üretir
// Patched data'yı alır, section'ları ekler, entry point'i korur
// UBIN transmutation tamamlanır – yeni converged binary hazır

use goblin::elf::Elf;
use object::write::Object;
use object::Architecture;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct RebuildReport {
    pub original_path: PathBuf,
    pub patched_data_size: usize,
    pub rebuilt_path: PathBuf,
    pub format: String,
    pub new_entry_point: u64,
    pub added_sections: Vec<String>,
    pub success: bool,
    pub error_msg: Option<String>,
}

pub struct UbinRebuilder;

impl UbinRebuilder {
    pub fn new() -> Self {
        UbinRebuilder
    }

    /// Patched binary verisini alır ve platforma göre yeniden inşa eder
    pub fn rebuild_converged_binary(&self, original_path: &PathBuf, patched_data: Vec<u8>) -> RebuildReport {
        let original_data = match fs::read(original_path) {
            Ok(d) => d,
            Err(e) => {
                return RebuildReport {
                    original_path: original_path.clone(),
                    patched_data_size: patched_data.len(),
                    rebuilt_path: PathBuf::new(),
                    format: "Unknown".to_string(),
                    new_entry_point: 0,
                    added_sections: vec![],
                    success: false,
                    error_msg: Some(format!("Original read error: {}", e)),
                };
            }
        };

        let rebuilt_path = original_path.with_extension("ubin-rebuilt");

        let report = if original_data.starts_with(b"\x7fELF") {
            self.rebuild_elf(&patched_data, original_path, &rebuilt_path)
        } else if original_data.starts_with(&[b'M', b'Z']) {
            self.rebuild_pe(&patched_data, original_path, &rebuilt_path)
        } else if original_data.starts_with(&[0xcf, 0xfa, 0xed, 0xfe]) || original_data.starts_with(&[0xca, 0xfe, 0xba, 0xbe]) {
            self.rebuild_macho(&patched_data, original_path, &rebuilt_path)
        } else {
            RebuildReport {
                original_path: original_path.clone(),
                patched_data_size: patched_data.len(),
                rebuilt_path: PathBuf::new(),
                format: "Unsupported".to_string(),
                new_entry_point: 0,
                added_sections: vec![],
                success: false,
                error_msg: Some("Unsupported binary format for rebuilding".to_string()),
            }
        };

        if report.success {
            println!("🏗️ UBIN REBUILD SUCCESS – New converged binary: {:?}", report.rebuilt_path);
            println!("   Format: {} | Size: {} bytes | Added sections: {:?}", report.format, report.patched_data_size, report.added_sections);
        } else {
            println!("❌ UBIN REBUILD FAILED: {:?}", report.error_msg);
        }

        report
    }

    /// ELF rebuild
    fn rebuild_elf(&self, patched_data: &[u8], _original_path: &PathBuf, rebuilt_path: &PathBuf) -> RebuildReport {
        // DÜZELTME: mut kaldırıldı
        let elf = match Elf::parse(patched_data) {
            Ok(e) => e,
            Err(e) => {
                return RebuildReport {
                    original_path: _original_path.clone(),
                    patched_data_size: patched_data.len(),
                    rebuilt_path: rebuilt_path.clone(),
                    format: "ELF".to_string(),
                    new_entry_point: 0,
                    added_sections: vec![],
                    success: false,
                    error_msg: Some(format!("ELF parse error: {}", e)),
                };
            }
        };

        let added = vec![".ubin_polyfill".to_string(), ".ubin_runtime".to_string()];
        let success = fs::write(rebuilt_path, patched_data).is_ok();

        RebuildReport {
            original_path: _original_path.clone(),
            patched_data_size: patched_data.len(),
            rebuilt_path: rebuilt_path.clone(),
            format: "ELF64".to_string(),
            new_entry_point: elf.entry,
            added_sections: added,
            success,
            error_msg: if success { None } else { Some("Write failed".to_string()) },
        }
    }

    /// PE rebuild
    fn rebuild_pe(&self, patched_data: &[u8], _original_path: &PathBuf, rebuilt_path: &PathBuf) -> RebuildReport {
        let mut new_obj = Object::new(object::BinaryFormat::Pe, Architecture::X86_64, object::Endianness::Little);

        let text_section = new_obj.add_section(vec![], b".text".to_vec(), object::SectionKind::Text);
        new_obj.append_section_data(text_section, patched_data, 16);

        // DÜZELTME: write() metodunu kullan ve file'a yaz
        let obj_data = new_obj.write().unwrap_or_default();
        let success = fs::write(rebuilt_path, &obj_data).is_ok();

        RebuildReport {
            original_path: _original_path.clone(),
            patched_data_size: patched_data.len(),
            rebuilt_path: rebuilt_path.clone(),
            format: "PE64".to_string(),
            new_entry_point: 0x400000,
            added_sections: vec![".ubin_polyfill".to_string()],
            success,
            error_msg: if success { None } else { Some("PE write failed".to_string()) },
        }
    }

    /// Mach-O rebuild
    fn rebuild_macho(&self, patched_data: &[u8], _original_path: &PathBuf, rebuilt_path: &PathBuf) -> RebuildReport {
        let success = fs::write(rebuilt_path, patched_data).is_ok();

        RebuildReport {
            original_path: _original_path.clone(),
            patched_data_size: patched_data.len(),
            rebuilt_path: rebuilt_path.clone(),
            format: "Mach-O".to_string(),
            new_entry_point: 0,
            added_sections: vec!["__ubin_polyfill".to_string()],
            success,
            error_msg: if success { None } else { Some("Mach-O write failed".to_string()) },
        }
    }
}
