use crate::run_cli;
use crate::snap_test::{SnapshotPayload, assert_cli_snapshot};
use biome_console::BufferConsole;
use biome_fs::MemoryFileSystem;
use bpaf::Args;
use camino::Utf8Path;

// === Lint Tests ===

#[test]
fn should_lint_markdown_file() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let md_file = Utf8Path::new("test.md");
    fs.insert(
        md_file.into(),
        r#"# Heading

Some text here.

## Another heading
"#
        .as_bytes(),
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["lint", md_file.as_str()].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_lint_markdown_file",
        fs,
        console,
        result,
    ));
}

#[test]
fn should_lint_markdown_file_with_errors() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let md_file = Utf8Path::new("test.md");
    // Missing blank line before heading (noMissingBlankLineBeforeHeading)
    // Heading increment violation (noHeadingLevelSkip)
    fs.insert(
        md_file.into(),
        r#"# Title
Some text
#### Skipped heading level
"#
        .as_bytes(),
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["lint", md_file.as_str()].as_slice()),
    );

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_lint_markdown_file_with_errors",
        fs,
        console,
        result,
    ));
}

// === Format Tests ===

#[test]
fn should_format_markdown_file() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let md_file = Utf8Path::new("test.md");
    fs.insert(
        md_file.into(),
        r#"##  Heading with extra spaces

Some text.
"#
        .as_bytes(),
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["format", md_file.as_str()].as_slice()),
    );

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_format_markdown_file",
        fs,
        console,
        result,
    ));
}

#[test]
fn should_format_write_markdown_file() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let md_file = Utf8Path::new("test.md");
    // Thematic break normalization: *** -> ---
    fs.insert(
        md_file.into(),
        r#"# Heading

***
"#
        .as_bytes(),
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["format", "--write", md_file.as_str()].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_format_write_markdown_file",
        fs,
        console,
        result,
    ));
}

// === Check Tests ===

#[test]
fn should_check_markdown_file() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let md_file = Utf8Path::new("test.md");
    fs.insert(
        md_file.into(),
        r#"# Title

Some text here.

## Section
"#
        .as_bytes(),
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["check", md_file.as_str()].as_slice()),
    );

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_check_markdown_file",
        fs,
        console,
        result,
    ));
}

// === Configuration Tests ===

#[test]
fn should_lint_markdown_with_config() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let md_file = Utf8Path::new("test.md");
    fs.insert(
        md_file.into(),
        r#"# Title

Some text.
"#
        .as_bytes(),
    );

    fs.insert(
        Utf8Path::new("biome.json").into(),
        r#"{
    "linter": {
        "rules": {
            "style": {
                "noMultipleSpaceAtxHeading": "error"
            }
        }
    }
}"#
        .as_bytes(),
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["lint", md_file.as_str()].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_lint_markdown_with_config",
        fs,
        console,
        result,
    ));
}

#[test]
fn should_format_markdown_with_disabled_formatter() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let md_file = Utf8Path::new("test.md");
    fs.insert(
        md_file.into(),
        r#"***
"#
        .as_bytes(),
    );

    fs.insert(
        Utf8Path::new("biome.json").into(),
        r#"{
    "markdown": {
        "formatter": {
            "enabled": false
        }
    }
}"#
        .as_bytes(),
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["format", md_file.as_str()].as_slice()),
    );

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_format_markdown_with_disabled_formatter",
        fs,
        console,
        result,
    ));
}

// === Stdin Tests ===

#[test]
fn lint_stdin_markdown_successfully() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    console.in_buffer.push(
        "# Heading\n\nSome text here.\n".to_string(),
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["lint", "--stdin-file-path", "test.md"].as_slice()),
    );

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "lint_stdin_markdown_successfully",
        fs,
        console,
        result,
    ));
}

#[test]
fn format_stdin_markdown_successfully() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    console.in_buffer.push(
        "##  Heading with extra spaces\n\nSome text.\n".to_string(),
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["format", "--stdin-file-path", "test.md"].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "format_stdin_markdown_successfully",
        fs,
        console,
        result,
    ));
}

// === Edge Cases ===

#[test]
fn should_handle_empty_markdown_file() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let md_file = Utf8Path::new("empty.md");
    fs.insert(md_file.into(), b"");

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["lint", md_file.as_str()].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_handle_empty_markdown_file",
        fs,
        console,
        result,
    ));
}

#[test]
fn should_handle_markdown_with_crlf() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let md_file = Utf8Path::new("crlf.md");
    fs.insert(
        md_file.into(),
        b"# Heading\r\n\r\nSome text.\r\n",
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["lint", md_file.as_str()].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_handle_markdown_with_crlf",
        fs,
        console,
        result,
    ));
}

#[test]
fn should_lint_multiple_markdown_files() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let file1 = Utf8Path::new("one.md");
    let file2 = Utf8Path::new("two.md");

    fs.insert(
        file1.into(),
        b"# First\n\nContent.\n",
    );
    fs.insert(
        file2.into(),
        b"# Second\n\nMore content.\n",
    );

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["lint", file1.as_str(), file2.as_str()].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_lint_multiple_markdown_files",
        fs,
        console,
        result,
    ));
}

#[test]
fn should_format_empty_markdown_file() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let md_file = Utf8Path::new("empty.md");
    fs.insert(md_file.into(), b"");

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["format", md_file.as_str()].as_slice()),
    );

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "should_format_empty_markdown_file",
        fs,
        console,
        result,
    ));
}
