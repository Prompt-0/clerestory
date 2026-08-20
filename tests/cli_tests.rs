use clap::CommandFactory;
use clerestory::cli::args::Cli;
use clerestory::cli::manpage::ManpageGenerator;
use tempfile::tempdir;

#[test]
fn test_cli_help_and_version() {
    let cmd = Cli::command();
    assert_eq!(cmd.get_name(), "clerestory");
    assert!(cmd.find_subcommand("doctor").is_some());
    assert!(cmd.find_subcommand("create").is_some());
    assert!(cmd.find_subcommand("inspect").is_some());
    assert!(cmd.find_subcommand("list").is_some());
    assert!(cmd.find_subcommand("start").is_some());
    assert!(cmd.find_subcommand("stop").is_some());
    assert!(cmd.find_subcommand("destroy").is_some());
    assert!(cmd.find_subcommand("completions").is_some());
    assert!(cmd.find_subcommand("man").is_some());
}

#[test]
fn test_manpage_generation() {
    let temp_dir = tempdir().expect("tempdir must succeed");
    ManpageGenerator::generate_to_dir(temp_dir.path()).expect("Manpage generation must succeed");

    let man_file = temp_dir.path().join("clerestory.1");
    assert!(man_file.exists(), "clerestory.1 must exist");

    let content = std::fs::read_to_string(man_file).expect("Must read man file");
    assert!(content.contains(".TH clerestory 1"));
    assert!(content.contains("Windows 11"));
}
