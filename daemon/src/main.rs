//! lofi-daemon: AI music generation daemon using MusicGen ONNX backend.
//!
//! This binary can run in two modes:
//! - CLI mode: Standalone music generation for testing (Phase 0)
//! - Daemon mode: JSON-RPC server for Neovim integration

use std::time::Instant;

use lofi_daemon::audio::write_wav;
use lofi_daemon::cli::Cli;
use lofi_daemon::error::Result;
use lofi_daemon::generation::generate_with_progress;
use lofi_daemon::models::ensure_models;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse_args();

    if cli.is_daemon_mode() {
        run_daemon_mode()
    } else if cli.is_cli_mode() {
        run_cli_mode(&cli)
    } else {
        print_usage();
        Ok(())
    }
}

/// Runs the CLI mode for Phase 0 validation.
fn run_cli_mode(cli: &Cli) -> Result<()> {
    let prompt = cli.prompt.as_ref().expect("Prompt required in CLI mode");
    let model_dir = cli.model_directory();
    let output_path = cli.output_path();

    eprintln!("=== lofi-daemon Phase 0 CLI ===");
    eprintln!("Prompt: \"{}\"", prompt);
    eprintln!("Duration: {}s", cli.duration);
    eprintln!("Output: {}", output_path.display());
    eprintln!("Model directory: {}", model_dir.display());
    if let Some(seed) = cli.seed {
        eprintln!("Seed: {}", seed);
    }
    eprintln!();

    // Ensure models are downloaded
    eprintln!("Checking model files...");
    ensure_models(&model_dir)?;
    eprintln!();

    // Start timing
    let start_time = Instant::now();

    // Generate audio with progress callback
    let samples = generate_with_progress(
        prompt,
        cli.duration,
        cli.seed,
        &model_dir,
        |current, total| {
            // Progress is logged in the generate function
            let _ = (current, total);
        },
    )?;

    // Calculate generation time
    let generation_time = start_time.elapsed();
    let generation_time_sec = generation_time.as_secs_f32();

    eprintln!();
    eprintln!("Generation complete!");
    eprintln!("  Time: {:.2}s", generation_time_sec);
    eprintln!("  Samples: {}", samples.len());
    eprintln!(
        "  Audio duration: {:.2}s",
        samples.len() as f32 / 32000.0
    );
    eprintln!();

    // Write to WAV file
    eprintln!("Writing WAV file...");
    write_wav(&samples, &output_path, 32000)?;
    eprintln!("Saved to: {}", output_path.display());

    // Phase 0 success criteria
    eprintln!();
    eprintln!("=== Phase 0 Results ===");
    eprintln!(
        "Target: Generate {}s audio in <120s",
        cli.duration
    );
    eprintln!("Actual: Generated {:.2}s audio in {:.2}s",
        samples.len() as f32 / 32000.0,
        generation_time_sec
    );

    if generation_time_sec < 120.0 {
        eprintln!("Result: PASS ✓");
    } else {
        eprintln!("Result: FAIL ✗ (exceeded 120s time limit)");
    }

    Ok(())
}

/// Runs the daemon mode (JSON-RPC server).
///
/// This is a placeholder for Phase 4 implementation.
fn run_daemon_mode() -> Result<()> {
    eprintln!("Daemon mode not yet implemented.");
    eprintln!("Use CLI mode with --prompt for Phase 0 testing.");
    Ok(())
}

/// Prints usage information.
fn print_usage() {
    eprintln!("lofi-daemon: AI music generation using MusicGen ONNX");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  CLI mode (Phase 0 testing):");
    eprintln!("    lofi-daemon --prompt \"lofi hip hop beats\" --duration 10 --output test.wav");
    eprintln!();
    eprintln!("  Daemon mode (JSON-RPC server):");
    eprintln!("    lofi-daemon --daemon");
    eprintln!();
    eprintln!("Run 'lofi-daemon --help' for full options.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_usage_doesnt_panic() {
        print_usage();
    }
}
