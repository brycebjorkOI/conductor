//! Rune script execution for automation steps.
//!
//! Scripts receive the previous step's output as `input` (a string) and
//! return a Rune `Object` whose key-value pairs become the step's output,
//! serialized as JSON for downstream steps.

use std::path::Path;
use std::sync::Arc;

/// Execute a Rune script file, passing `input` as the first argument.
///
/// The script's entry function should accept a single `String` parameter
/// and return an `Object` (dynamic key-value map). The returned object is
/// serialized to a JSON string.
///
/// Example `.rn` script:
/// ```rune
/// pub fn main(input) {
///     let summary = input.len().to_string();
///     #{
///         summary: summary,
///         word_count: input.split(' ').len().to_string(),
///     }
/// }
/// ```
pub async fn run_script(
    script_path: &Path,
    entry_function: &str,
    input: Option<&str>,
) -> Result<String, String> {
    let script_path = script_path.to_path_buf();
    let entry = entry_function.to_string();
    let input_owned = input.map(|s| s.to_string());

    // Run in a blocking task since Rune compilation is CPU-bound.
    tokio::task::spawn_blocking(move || execute_sync(&script_path, &entry, input_owned.as_deref()))
        .await
        .map_err(|e| format!("script task panicked: {e}"))?
}

fn execute_sync(
    script_path: &Path,
    entry_function: &str,
    input: Option<&str>,
) -> Result<String, String> {
    use rune::termcolor::{ColorChoice, StandardStream};
    use rune::{Context, Diagnostics, Source, Sources, Vm};

    if !script_path.exists() {
        return Err(format!("script not found: {}", script_path.display()));
    }

    // Build context with default modules (string, collections, io, etc.).
    let context = Context::with_default_modules()
        .map_err(|e| format!("rune context: {e}"))?;
    let runtime = Arc::new(
        context
            .runtime()
            .map_err(|e| format!("rune runtime: {e}"))?,
    );

    // Load the script source.
    let mut sources = Sources::new();
    sources
        .insert(
            Source::from_path(script_path)
                .map_err(|e| format!("load script: {e}"))?,
        )
        .map_err(|e| format!("insert source: {e}"))?;

    // Compile.
    let mut diagnostics = Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if diagnostics.has_error() {
        diagnostics
            .emit(&mut StandardStream::stderr(ColorChoice::Never), &sources)
            .ok();
        return Err(format!(
            "script compilation failed with {} error(s)",
            diagnostics.into_diagnostics().len()
        ));
    }

    let unit = result.map_err(|e| format!("compile: {e}"))?;

    // Create VM and call the entry function.
    let mut vm = Vm::new(runtime, Arc::new(unit));

    let input_val = input.unwrap_or("").to_string();
    let entry_path: Vec<&str> = entry_function.split("::").collect();

    let output = vm
        .call(&entry_path, (input_val,))
        .map_err(|e| format!("script execution error: {e}"))?;

    // Convert the return value to JSON.
    value_to_json(&output)
}

/// Convert a Rune `Value` to a JSON string via serde.
fn value_to_json(value: &rune::Value) -> Result<String, String> {
    serde_json::to_string_pretty(value)
        .map_err(|e| format!("cannot serialize script output to JSON: {e}"))
}
