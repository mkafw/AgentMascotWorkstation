use goblin::Object;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 证据账本条目：禁止 AI 幻觉，每一行重构逻辑必须有二进制或源码反编译证据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Evidence {
    pub source_offset: String,
    pub raw_symbol: String,
    pub deduced_role: String,
    pub confidence: f32,
}

/// 逆向解构出的重构规格书 (Reconstruction Spec)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReconstructionSpec {
    pub target_name: String,
    pub format: String,
    pub total_symbols_found: usize,
    pub core_symbols: Vec<String>,
    pub evidence_ledger: Vec<Evidence>,
    pub rust_reconstruction_advice: Vec<String>,
}

pub struct ReaEngine;

impl ReaEngine {
    /// 逆向原生二进制（ELF/PE/Mach-O），抽取重构规格
    pub fn reverse_binary(target_path: &Path) -> Result<ReconstructionSpec, Box<dyn std::error::Error>> {
        let buffer = fs::read(target_path)?;
        let mut symbols = Vec::new();
        let mut evidence_ledger = Vec::new();
        let mut advice = Vec::new();
        let format_desc;

        match Object::parse(&buffer)? {
            Object::Elf(elf) => {
                format_desc = format!("Linux ELF (Machine: {:?})", elf.header.e_machine);
                advice.push("检测到原生 ELF 二进制，建议使用 Rust 原生 `tokio` 异步运行时替代原生线程池".into());
                advice.push("动态符号表已解构，可直接映射为 Rust `trait` 契约".into());

                for sym in elf.dynsyms.iter() {
                    if let Some(name) = elf.dynstrtab.get_at(sym.st_name) {
                        if !name.is_empty() && !name.starts_with('_') {
                            symbols.push(name.to_string());
                            if evidence_ledger.len() < 15 {
                                evidence_ledger.push(Evidence {
                                    source_offset: format!("0x{:x}", sym.st_value),
                                    raw_symbol: name.to_string(),
                                    deduced_role: "核心对外 API 导出或系统调用点".into(),
                                    confidence: 0.98,
                                });
                            }
                        }
                    }
                }
            }
            Object::PE(pe) => {
                format_desc = "Windows PE 可执行程序".to_string();
                advice.push("检测到 PE 格式，建议使用 Rust `windows-rs` 或跨平台无依赖纯 safe Rust 重构".into());
                for export in pe.exports {
                    if let Some(name) = export.name {
                        symbols.push(name.to_string());
                    }
                }
            }
            _ => {
                format_desc = "脚本或自定义压缩归档 (JavaScript/Electron/Text)".to_string();
                advice.push("建议解析 AST 语法树，使用 Rust `serde_json` 和模式匹配直接还原逻辑".into());
            }
        }

        let total_symbols_found = symbols.len();
        let core_symbols = symbols.into_iter().take(20).collect();

        Ok(ReconstructionSpec {
            target_name: target_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            format: format_desc,
            total_symbols_found,
            core_symbols,
            evidence_ledger,
            rust_reconstruction_advice: advice,
        })
    }
}
