// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Every source file carries its licence header.
//!
//! Provenance: this repository is AGPL-3.0-or-later and `publish = false`. A
//! missing SPDX line is not cosmetic — it is the file that a later extraction
//! copies somewhere unlicensed.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic IS the failure report; the production lints stand"
)]

use std::path::{Path, PathBuf};

const EXPECTED: &str = "// SPDX-License-Identifier: AGPL-3.0-or-later";

/// Trace: NFR-002-AC-1
/// `tc_020`: every `.rs` file under the repository begins with the SPDX header.
#[test]
fn tc_020_every_rust_source_declares_its_licence() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut missing = Vec::new();
    for file in rust_files(&root) {
        let text = std::fs::read_to_string(&file).unwrap();
        if !text.starts_with(EXPECTED) {
            missing.push(file.display().to_string());
        }
    }
    assert!(missing.is_empty(), "missing SPDX header: {missing:?}");
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if name == "target" || name == ".git" {
            continue;
        }
        if path.is_dir() {
            found.extend(rust_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
    found
}
