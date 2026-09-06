use std::{
    fs::{File, OpenOptions},
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{anyhow, Result};
use clap::{
    builder::{BoolishValueParser, TypedValueParser, ValueParserFactory},
    Parser,
};

mod anonymize;
mod color_json;
mod copy;
mod examine;
mod examine_sync;
mod export;
mod history;
mod import;
mod merge;

#[derive(Parser, Debug)]
#[clap(about = "Automerge CLI")]
struct Opts {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ExportFormat {
    Json,
    Toml,
}

#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct VerifyFlag(bool);

impl VerifyFlag {
    fn load(&self, buf: &[u8]) -> Result<automerge::Automerge, automerge::AutomergeError> {
        if self.0 {
            automerge::Automerge::load(buf)
        } else {
            automerge::Automerge::load_unverified_heads(buf)
        }
    }
}

#[derive(Clone)]
struct VerifyFlagParser;
impl ValueParserFactory for VerifyFlag {
    type Parser = VerifyFlagParser;

    fn value_parser() -> Self::Parser {
        VerifyFlagParser
    }
}

impl TypedValueParser for VerifyFlagParser {
    type Value = VerifyFlag;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        BoolishValueParser::new()
            .parse_ref(cmd, arg, value)
            .map(VerifyFlag)
    }
}

impl FromStr for ExportFormat {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<ExportFormat> {
        match input {
            "json" => Ok(ExportFormat::Json),
            "toml" => Ok(ExportFormat::Toml),
            _ => Err(anyhow!("Invalid export format: {}", input)),
        }
    }
}

#[derive(Debug, Parser)]
enum Command {
    /// Output current state of an Automerge document in a specified format
    Export {
        /// Format for output: json, toml
        #[clap(long, short, default_value = "json")]
        format: ExportFormat,

        /// Path that contains Automerge changes
        changes_file: Option<PathBuf>,

        /// The file to write to. If omitted assumes stdout
        #[clap(long("out"), short('o'))]
        output_file: Option<PathBuf>,

        /// Whether to verify the head hashes of a compressed document
        #[clap(long, action = clap::ArgAction::SetFalse)]
        skip_verifying_heads: VerifyFlag,
    },

    Import {
        /// Format for input: json, toml
        #[clap(long, short, default_value = "json")]
        format: ExportFormat,

        input_file: Option<PathBuf>,

        /// Path to write Automerge changes to
        #[clap(long("out"), short('o'))]
        changes_file: Option<PathBuf>,
    },

    /// Read an automerge document and print a JSON representation of the changes in it to stdout
    Examine {
        input_file: Option<PathBuf>,

        /// Whether to verify the head hashes of a compressed document
        #[clap(long, action = clap::ArgAction::SetFalse)]
        skip_verifying_heads: VerifyFlag,
    },

    /// Read an automerge sync messaage and print a JSON representation of it
    ExamineSync { input_file: Option<PathBuf> },

    /// Replace private document data while retaining history and structural shape
    Anonymize {
        /// The Automerge document to anonymize. If omitted, reads from stdin
        input_file: Option<PathBuf>,

        /// The file to write to. If omitted, writes to stdout
        #[clap(long("out"), short('o'))]
        output_file: Option<PathBuf>,
    },

    /// Read one or more automerge documents and output a merged, compacted version of them
    Merge {
        /// The file to write to. If omitted assumes stdout
        #[clap(long("out"), short('o'))]
        output_file: Option<PathBuf>,

        /// The file(s) to compact. If empty assumes stdin
        input: Vec<PathBuf>,
    },
    /// Validate an Automerge document and preserve its exact bytes.
    Copy {
        /// The Automerge document to copy. If omitted, reads from stdin.
        input_file: Option<PathBuf>,

        /// The file to write to. If omitted, writes to stdout.
        #[clap(long("out"), short('o'))]
        output_file: Option<PathBuf>,
    },

    /// Print the document's current change heads as JSON.
    Heads { input_file: Option<PathBuf> },

    /// Print changes present in the second document but not the first as JSON.
    Diff {
        before_file: PathBuf,
        after_file: PathBuf,
    },

    /// Extract appendable changes from a base document to a target document.
    Extract {
        base_file: PathBuf,
        target_file: PathBuf,

        /// The file to write change chunks to. If omitted, writes to stdout.
        #[clap(long("out"), short('o'))]
        output_file: Option<PathBuf>,
    },

    /// Apply appendable change chunks to a base document.
    Apply {
        base_file: PathBuf,
        incremental_file: PathBuf,

        /// The file to write the resulting document to. If omitted, writes to stdout.
        #[clap(long("out"), short('o'))]
        output_file: Option<PathBuf>,
    },
}

fn open_file_or_stdin(maybe_path: Option<PathBuf>) -> Result<Box<dyn std::io::Read>> {
    if let Some(path) = maybe_path {
        Ok(Box::new(File::open(path)?))
    } else if std::io::stdin().is_terminal() {
        Err(anyhow!("Provide a file path or pipe input through stdin"))
    } else {
        Ok(Box::new(std::io::stdin()))
    }
}

struct AtomicOutput {
    destination: PathBuf,
    temporary: Option<(PathBuf, File)>,
    permissions: Option<std::fs::Permissions>,
}

static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

impl AtomicOutput {
    fn new(destination: PathBuf) -> Result<Self> {
        let permissions = validate_output_destination(&destination)?;

        let parent = destination.parent().unwrap_or_else(|| Path::new("."));
        let file_name = destination
            .file_name()
            .ok_or_else(|| anyhow!("output path has no filename: {}", destination.display()))?;
        let counter = TEMPORARY_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut temporary_path = None;
        let mut temporary_file = None;
        for attempt in 0..100 {
            let candidate = parent.join(format!(
                ".{}.automerge-{}-{}.tmp",
                file_name.to_string_lossy(),
                std::process::id(),
                counter + attempt
            ));
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(&candidate) {
                Ok(file) => {
                    temporary_path = Some(candidate);
                    temporary_file = Some(file);
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.into()),
            }
        }
        let temporary_path = temporary_path.ok_or_else(|| {
            anyhow!(
                "could not create a unique temporary output file next to {}",
                destination.display()
            )
        })?;
        let temporary_file = temporary_file.expect("temporary path and file are created together");

        Ok(Self {
            destination,
            temporary: Some((temporary_path, temporary_file)),
            permissions,
        })
    }

    fn finish(mut self) -> Result<()> {
        let (temporary_path, mut temporary_file) = self
            .temporary
            .take()
            .expect("atomic output can only be finished once");
        let result = (|| {
            temporary_file.flush()?;
            drop(temporary_file);
            if let Some(permissions) = self.permissions.take() {
                std::fs::set_permissions(&temporary_path, permissions)?;
            }
            replace_file(&temporary_path, &self.destination)
        })();
        if let Err(error) = result {
            let _ = std::fs::remove_file(&temporary_path);
            return Err(anyhow!(
                "failed to replace {}: {error}",
                self.destination.display()
            ));
        }
        Ok(())
    }
}

fn validate_output_destination(destination: &Path) -> Result<Option<std::fs::Permissions>> {
    match std::fs::symlink_metadata(destination) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(anyhow!(
                    "refusing atomic output through a symlink: {}",
                    destination.display()
                ));
            }
            if !metadata.file_type().is_file() {
                return Err(anyhow!(
                    "atomic output requires a regular file: {}",
                    destination.display()
                ));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if metadata.nlink() > 1 {
                    return Err(anyhow!(
                        "refusing atomic output for a file with multiple hard links: {}",
                        destination.display()
                    ));
                }
            }
            Ok(Some(metadata.permissions()))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

impl Write for AtomicOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.temporary
            .as_mut()
            .expect("atomic output must be writable before finish")
            .1
            .write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.temporary
            .as_mut()
            .expect("atomic output must be writable before finish")
            .1
            .flush()
    }
}

impl Drop for AtomicOutput {
    fn drop(&mut self) {
        if let Some((temporary_path, temporary_file)) = self.temporary.take() {
            drop(temporary_file);
            let _ = std::fs::remove_file(temporary_path);
        }
    }
}

#[cfg(not(windows))]
fn replace_file(temporary: &Path, destination: &Path) -> io::Result<()> {
    std::fs::rename(temporary, destination)
}

#[cfg(windows)]
fn replace_file(temporary: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING};

    let temporary: Vec<u16> = temporary.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let replaced = unsafe {
        MoveFileExW(
            temporary.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING,
        )
    };
    if replaced == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

enum CliOutput {
    File(AtomicOutput),
    Stdout(std::io::Stdout),
}

impl CliOutput {
    fn finish(mut self) -> Result<()> {
        self.flush()?;
        match self {
            Self::File(output) => output.finish(),
            Self::Stdout(_) => Ok(()),
        }
    }
}

impl Write for CliOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self {
            Self::File(output) => output.write(bytes),
            Self::Stdout(output) => output.write(bytes),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::File(output) => output.flush(),
            Self::Stdout(output) => output.flush(),
        }
    }
}

fn create_file_or_stdout(maybe_path: Option<PathBuf>) -> Result<CliOutput> {
    if let Some(path) = maybe_path {
        Ok(CliOutput::File(AtomicOutput::new(path)?))
    } else if std::io::stdout().is_terminal() {
        Err(anyhow!("Provide a file path or pipe output to stdout"))
    } else {
        Ok(CliOutput::Stdout(std::io::stdout()))
    }
}

fn write_file_if_changed(path: &Path, bytes: &[u8]) -> Result<()> {
    validate_output_destination(path)?;
    match std::fs::read(path) {
        Ok(existing) if existing == bytes => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let mut output = AtomicOutput::new(path.to_path_buf())?;
    output.write_all(bytes)?;
    output.finish()
}

fn write_output(output_file: Option<PathBuf>, bytes: &[u8]) -> Result<()> {
    if let Some(output_file) = output_file {
        write_file_if_changed(&output_file, bytes)
    } else {
        let mut output = create_file_or_stdout(None)?;
        output.write_all(bytes)?;
        output.finish()
    }
}

fn paths_refer_to_same_file(input: &Path, output: &Path) -> Result<bool> {
    if input == output || !output.exists() {
        return Ok(input == output);
    }

    let input_metadata = std::fs::metadata(input)?;
    let output_metadata = std::fs::metadata(output)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if input_metadata.dev() == output_metadata.dev()
            && input_metadata.ino() == output_metadata.ino()
        {
            return Ok(true);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if input_metadata.volume_serial_number() == output_metadata.volume_serial_number()
            && input_metadata.file_index() == output_metadata.file_index()
        {
            return Ok(true);
        }
    }
    Ok(input.canonicalize()? == output.canonicalize()?)
}

fn ensure_paths_differ(input: Option<&Path>, output: Option<&Path>) -> Result<()> {
    if let (Some(input), Some(output)) = (input, output) {
        if paths_refer_to_same_file(input, output)? {
            return Err(anyhow!("input and output paths must differ"));
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    let opts = Opts::parse();
    match opts.cmd {
        Command::Export {
            changes_file,
            format,
            output_file,
            skip_verifying_heads,
        } => {
            ensure_paths_differ(changes_file.as_deref(), output_file.as_deref())?;
            let mut output = create_file_or_stdout(output_file)?;
            match format {
                ExportFormat::Json => {
                    let mut in_buffer = open_file_or_stdin(changes_file)?;
                    export::export_json(
                        &mut in_buffer,
                        &mut output,
                        skip_verifying_heads,
                        std::io::stdout().is_terminal(),
                    )?;
                    output.finish()
                }
                ExportFormat::Toml => unimplemented!(),
            }
        }
        Command::Import {
            format,
            input_file,
            changes_file,
        } => match format {
            ExportFormat::Json => {
                ensure_paths_differ(input_file.as_deref(), changes_file.as_deref())?;
                let mut out_buffer = create_file_or_stdout(changes_file)?;
                let mut in_buffer = open_file_or_stdin(input_file)?;
                import::import_json(&mut in_buffer, &mut out_buffer)?;
                out_buffer.finish()
            }
            ExportFormat::Toml => unimplemented!(),
        },
        Command::Examine {
            input_file,
            skip_verifying_heads,
        } => {
            let in_buffer = open_file_or_stdin(input_file)?;
            let out_buffer = std::io::stdout();
            match examine::examine(
                in_buffer,
                out_buffer,
                skip_verifying_heads,
                std::io::stdout().is_terminal(),
            ) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                }
            }
            Ok(())
        }
        Command::ExamineSync { input_file } => {
            let in_buffer = open_file_or_stdin(input_file)?;
            let out_buffer = std::io::stdout();
            match examine_sync::examine_sync(in_buffer, out_buffer, std::io::stdout().is_terminal())
            {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                }
            }
            Ok(())
        }
        Command::Anonymize {
            input_file,
            output_file,
        } => {
            ensure_paths_differ(input_file.as_deref(), output_file.as_deref())?;

            let input = open_file_or_stdin(input_file)?;
            // Do not truncate an existing output until the input has loaded and anonymization has
            // succeeded.
            let anonymized = anonymize::anonymize(input)?;
            let mut output = create_file_or_stdout(output_file)?;
            output.write_all(&anonymized.bytes)?;
            output.finish()?;
            eprintln!(
                "anonymized {} change(s), {} operation(s), and {} actor(s)",
                anonymized.change_count, anonymized.operation_count, anonymized.actor_count
            );
            eprintln!(
                "review the output before publishing; document structure and lengths are retained"
            );
            Ok(())
        }
        Command::Merge { input, output_file } => {
            match merge::merge(input.into()) {
                Ok(merged) => write_output(output_file, &merged)?,
                Err(e) => return Err(e.into()),
            }
            Ok(())
        }
        Command::Copy {
            input_file,
            output_file,
        } => {
            let bytes = copy::copy(input_file)?;
            write_output(output_file, &bytes)?;
            Ok(())
        }
        Command::Heads { input_file } => history::print_heads(input_file),
        Command::Diff {
            before_file,
            after_file,
        } => history::print_diff(&before_file, &after_file),
        Command::Extract {
            base_file,
            target_file,
            output_file,
        } => {
            ensure_paths_differ(Some(&base_file), output_file.as_deref())?;
            ensure_paths_differ(Some(&target_file), output_file.as_deref())?;
            let bytes = history::extract(&base_file, &target_file)?;
            write_output(output_file, &bytes)
        }
        Command::Apply {
            base_file,
            incremental_file,
            output_file,
        } => {
            ensure_paths_differ(Some(&base_file), output_file.as_deref())?;
            ensure_paths_differ(Some(&incremental_file), output_file.as_deref())?;
            let bytes = history::apply(&base_file, &incremental_file)?;
            write_output(output_file, &bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_an_atomic_output_removes_its_temporary_file() {
        let base = std::env::temp_dir().join(format!(
            "automerge-cli-atomic-drop-{}-{}",
            std::process::id(),
            TEMPORARY_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&base).unwrap();
        let destination = base.join("output.automerge");
        {
            let mut output = AtomicOutput::new(destination).unwrap();
            output.write_all(b"uncommitted").unwrap();
            assert_eq!(std::fs::read_dir(&base).unwrap().count(), 1);
        }
        assert_eq!(std::fs::read_dir(&base).unwrap().count(), 0);
        std::fs::remove_dir(&base).unwrap();
    }
}
