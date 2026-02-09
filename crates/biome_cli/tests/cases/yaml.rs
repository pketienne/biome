use crate::run_cli;
use crate::snap_test::{SnapshotPayload, assert_cli_snapshot, assert_file_contents};
use biome_console::BufferConsole;
use biome_fs::MemoryFileSystem;
use bpaf::Args;
use camino::Utf8Path;

const UNFORMATTED: &str = "key1:   value1\nkey2:  value2\nnested:\n  child:   deep\n";

const FORMATTED: &str = "key1: value1\nkey2: value2\nnested:\n  child: deep\n";

const LINT_INVALID: &str = "first: &anchor value1\nsecond: &anchor value2\n";

#[test]
fn format_yaml_files() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let file_path = Utf8Path::new("file.yaml");
    fs.insert(file_path.into(), UNFORMATTED.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["format", file_path.as_str()].as_slice()),
    );

    assert!(result.is_err(), "run_cli returned {result:?}");

    assert_file_contents(&fs, file_path, UNFORMATTED);

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "format_yaml_files",
        fs,
        console,
        result,
    ));
}

#[test]
fn format_and_write_yaml_files() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let file_path = Utf8Path::new("file.yaml");
    fs.insert(file_path.into(), UNFORMATTED.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["format", "--write", file_path.as_str()].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_file_contents(&fs, file_path, FORMATTED);

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "format_and_write_yaml_files",
        fs,
        console,
        result,
    ));
}

#[test]
fn lint_yaml_files() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let file_path = Utf8Path::new("file.yaml");
    fs.insert(file_path.into(), LINT_INVALID.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(
            [
                "lint",
                "--only=nursery/noDuplicateAnchors",
                file_path.as_str(),
            ]
            .as_slice(),
        ),
    );

    assert!(result.is_err(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "lint_yaml_files",
        fs,
        console,
        result,
    ));
}

#[test]
fn check_yaml_files() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let file_path = Utf8Path::new("file.yaml");
    fs.insert(file_path.into(), UNFORMATTED.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["check", file_path.as_str()].as_slice()),
    );

    assert!(result.is_err(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "check_yaml_files",
        fs,
        console,
        result,
    ));
}
