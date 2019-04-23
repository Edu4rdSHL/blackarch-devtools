// Crate clap
#[macro_use]
extern crate clap;
use clap::App;
use clap::AppSettings;

// Functions module
mod functions;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    if matches.is_present("setup") {
        functions::setup_chroot();
    } else if matches.is_present("build") {
        if matches.is_present("clean") {
            functions::sync_chroot();
            functions::build_package();
        } else {
            functions::build_package();
        }
    } else if matches.is_present("update") {
        functions::update_chroot_packages();
    } else if matches.is_present("test") {
        let package = matches
            .value_of("package")
            .expect("Failed to convert in a valid String")
            .to_string();
        let executable = matches
            .value_of("executable")
            .expect("Failed to convert in a valid String")
            .to_string();
        if matches.is_present("clean") {
            functions::sync_chroot();
            functions::test_package(&package, &executable);
        } else {
            functions::test_package(&package, &executable);
        }
    } else if matches.is_present("install-missing") {
        if matches.is_present("clean") {
            functions::sync_chroot();
            let missing_deps: Vec<&str> = matches.values_of("install-missing").unwrap().collect();
            functions::build_package_with_missing_deps(&missing_deps.as_slice());
        } else {
            let missing_deps: Vec<&str> = matches.values_of("install-missing").unwrap().collect();
            functions::build_package_with_missing_deps(&missing_deps.as_slice());
        }
    } else if matches.is_present("clean") {
        functions::sync_chroot();
    }
}
