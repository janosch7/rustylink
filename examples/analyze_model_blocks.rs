//! Analyze a Simulink model and report which blocks would render as a "?"
//! placeholder (i.e. have no catalog implementation: no icon, no renderer,
//! no label, rectangular body).
//!
//! Usage:
//!   cargo run --features egui,dashboard,highlight --example analyze_model_blocks -- <file.slx>

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use std::collections::BTreeMap;

use rustylink::model::{Block, System};
use rustylink::parser::{FsSource, SimulinkParser, ZipSource};
use rustylink::simulink_libraries::metadata::extract_metadata;
use rustylink::simulink_libraries::resolve_definition;
use rustylink::simulink_libraries::types::{BlockLabelPolicy, SimulinkShape};

#[derive(Parser, Debug)]
struct Args {
    file: String,
    #[arg(short = 'L', long = "lib")]
    lib: Vec<String>,
}

fn block_label_text(block: &Block) -> Option<String> {
    let def = resolve_definition(block);
    let metadata = extract_metadata(block, def);
    match def.block_label {
        BlockLabelPolicy::None => {}
        BlockLabelPolicy::Fixed(s) => return Some(s.to_string()),
        BlockLabelPolicy::MetadataDependent(f) => {
            if let Some(s) = f(block, &metadata) {
                return Some(s);
            }
        }
    }
    def.compute_instance_label.and_then(|f| f(block))
}

/// Replicates the exact `?`-placeholder decision in render_block_interior +
/// render_block_icon (live_mode = false).
fn shows_question(block: &Block) -> bool {
    let def = resolve_definition(block);
    let cfg = rustylink::egui_app::render::get_block_type_cfg(block);
    if def.shape == SimulinkShape::FilledBlack {
        return false;
    }
    if def.static_renderer.is_some() {
        return false;
    }
    if let Some(l) = block_label_text(block)
        && !l.is_empty()
    {
        return false;
    }
    // step 5: the catalog definition's icon is authoritative.
    if def.icon.is_some() {
        return false;
    }
    // iconless non-rectangle returns empty (no "?")
    if def.shape != SimulinkShape::Rectangle {
        return false;
    }
    // rectangular & iconless: legacy config-map icon path, "?" if none.
    cfg.icon.is_none()
}

fn ident(block: &Block) -> String {
    block
        .library_block_path
        .as_deref()
        .or_else(|| block.properties.get("SourceBlock").map(|s| s.as_str()))
        .map(|s| s.replace(['\n', '\r'], " "))
        .unwrap_or_else(|| block.block_type.clone())
}

fn walk(sys: &System, missing: &mut BTreeMap<String, (String, bool)>, total: &mut usize) {
    for block in &sys.blocks {
        *total += 1;
        if shows_question(block) {
            let def = resolve_definition(block);
            let key = ident(block);
            let resolved = format!("def={}|cat={}", def.block_type, def.category);
            missing
                .entry(key)
                .or_insert((resolved, def.shape == SimulinkShape::Rectangle));
        }
        if let Some(sub) = &block.subsystem {
            walk(sub, missing, total);
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Utf8PathBuf::from(&args.file);
    let mut lib_paths: Vec<Utf8PathBuf> = Vec::new();
    if let Some(parent) = path.parent()
        && parent.as_str() != ""
    {
        lib_paths.push(parent.to_path_buf());
    }
    lib_paths.extend(args.lib.iter().map(Utf8PathBuf::from));

    let mut root_system = if path.extension() == Some("slx") {
        let file = std::fs::File::open(&path).with_context(|| format!("Open {}", path))?;
        let reader = std::io::BufReader::new(file);
        let mut parser = SimulinkParser::new("", ZipSource::new(reader)?);
        let root = Utf8PathBuf::from("simulink/systems/system_root.xml");
        parser.parse_system_file(&root)?
    } else {
        let mut parser = SimulinkParser::new(Utf8PathBuf::from("."), FsSource);
        parser.parse_system_file(&path)?
    };
    SimulinkParser::<FsSource>::resolve_library_references(&mut root_system, &lib_paths)
        .with_context(|| "Failed to resolve library references")?;

    let mut missing: BTreeMap<String, (String, bool)> = BTreeMap::new();
    let mut total = 0usize;
    walk(&root_system, &mut missing, &mut total);

    println!("Total blocks: {total}");
    println!(
        "Distinct block identities rendering as \"?\": {}",
        missing.len()
    );
    println!("================");
    for (ident, (resolved, rect)) in &missing {
        println!("{:<52} | {:<45} rect={}", ident, resolved, rect);
    }
    Ok(())
}
