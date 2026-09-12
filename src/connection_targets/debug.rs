//! Human-readable dumps of resolved targets.

use super::resolver::ConnectionTargetResolver;
use super::target::ConnectionTarget;
use super::topology::line_identity;
use crate::model::{Block, Line, System};

pub fn debug_print_block_targets(root: &System, system_path: &[String], block: &Block) {
    let resolver = ConnectionTargetResolver::new(root);
    let targets = resolver.block_targets_for_block(system_path, block);
    println!("  [Targets] block '{}'", block.name);
    print_targets(&targets);
}

pub fn debug_print_block_targets_with_resolver(
    resolver: &ConnectionTargetResolver,
    system_path: &[String],
    block: &Block,
) {
    let targets = resolver.block_targets_for_block(system_path, block);
    println!("  [Targets] block '{}'", block.name);
    print_targets(&targets);
}

pub fn debug_print_line_targets(root: &System, system_path: &[String], line: &Line) {
    let resolver = ConnectionTargetResolver::new(root);
    let targets = resolver.line_targets_for_line(system_path, line);
    println!("  [Targets] line {}", line_identity(line));
    print_targets(&targets);
}

pub fn debug_print_line_targets_with_resolver(
    resolver: &ConnectionTargetResolver,
    system_path: &[String],
    line: &Line,
) {
    let targets = resolver.line_targets_for_line(system_path, line);
    println!("  [Targets] line {}", line_identity(line));
    print_targets(&targets);
}

fn print_targets(targets: &[ConnectionTarget]) {
    if targets.is_empty() {
        println!("    (no targets)");
        return;
    }

    for target in targets {
        println!(
            "    - path='{}' origin={:?} signal={:?} signal_names={:?} resolve={:?} index={:?} signals_only={} testpoint={} block_type={:?}",
            target.path,
            target.origin,
            target.signal_name,
            target.signal_names,
            target.resolve,
            target.element_index,
            target.signals_only,
            target.testpoint,
            target.block_type
        );
    }
}
