// Crate termcolor
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// Standard imports
use std::io::Write;
use std::path::Path;
use std::process;
use std::process::Command;
use std::{env, fs};

pub fn get_vars(get_var: &str) -> String {
    let home_dir: String = env::var("HOME").expect("Failed to read HOME system variable.");
    let chroot_dir: String = [&home_dir, "/blackarch_chroot"].concat();
    let devtools_mkarchroot = String::from("/usr/bin/mkarchroot");
    let devtools_nspawn = String::from("/usr/bin/arch-nspawn");
    let blackarch_instance: String = String::from("blackarch");
    let chroot_root: String = [&chroot_dir, "/root/"].concat();
    let chroot_blackarch: String = [&chroot_dir, "/blackarch/"].concat();
    let devtools_makechrootpkg: String = String::from("/usr/bin/makechrootpkg");
    let pacman: String = String::from("/usr/bin/pacman");
    if get_var == "home_dir" {
        home_dir
    } else if get_var == "chroot_dir" {
        chroot_dir
    } else if get_var == "mkarchroot" {
        devtools_mkarchroot
    } else if get_var == "nspawn" {
        devtools_nspawn
    } else if get_var == "chroot_root" {
        chroot_root
    } else if get_var == "chroot_blackarch" {
        chroot_blackarch
    } else if get_var == "blackarch_instance" {
        blackarch_instance
    } else if get_var == "makechrootpkg" {
        devtools_makechrootpkg
    } else if get_var == "pacman" {
        pacman
    } else {
        String::from("Error returning the value.")
    }
}

pub fn coloring(color: &str) -> termcolor::StandardStream {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    if color == "green" {
        stdout
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)))
            .unwrap();
        stdout
    } else if color == "yellow" {
        stdout
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)))
            .unwrap();
        stdout
    } else if color == "red" {
        stdout
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Red)))
            .unwrap();
        stdout
    } else {
        stdout
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::White)))
            .unwrap();
        stdout
    }
}

pub fn setup_chroot() {
    let chroot_dir = get_vars("chroot_dir");
    if Path::new(&chroot_dir).exists() {
        if Path::new(&chroot_dir).is_dir() {
            writeln!(coloring("red"), "The directory {} already exists in the system, remove it or try with a different path.", &chroot_dir).unwrap();
            process::exit(1);
        } else if Path::new(&chroot_dir).is_file() {
            writeln!(
                coloring("red"),
                "The file {} already exists in the system, remove it or try a different path.",
                &chroot_dir
            )
            .unwrap();
            process::exit(1);
        }
    } else {
        writeln!(
            coloring("yellow"),
            "Creating chroot directory with name: {}",
            &chroot_dir
        )
        .unwrap();
        fs::create_dir(&chroot_dir)
            .expect("An error as ocurred while creating the chroot directoy.");
        writeln!(coloring("yellow"), "Setting up chroot environment...").unwrap();
        let devtools_mkarchroot = get_vars("mkarchroot");
        let devtools_nspawn = get_vars("nspawn");
        let chroot_root = get_vars("chroot_root");
        if Path::new(&devtools_mkarchroot).exists() {
            let up_chroot = Command::new(&devtools_mkarchroot)
                .args(&[&chroot_root, "base", "base-devel"])
                .status()
                .expect("Failed to setup chroot environment.");
            if up_chroot.success() {
                writeln!(coloring("yellow"), "Enabling multilib repos...").unwrap();
                Command::new(&devtools_nspawn).args(&[&chroot_root, "/bin/sh", "-c", "echo -e '\n[multilib]\nInclude = /etc/pacman.d/mirrorlist\n' | sudo tee -a /etc/pacman.conf > /dev/null"]).status().expect("Failed enabling multilib repos.");
                writeln!(coloring("yellow"), "Changing makepkg.conf to use zstd...").unwrap();
                Command::new(&devtools_nspawn)
                    .args(&[
                        &chroot_root,
                        "/bin/sh",
                        "-c",
                        "sed -i 's/^PKGEXT.*/PKGEXT='\\''.pkg.tar.zst'\\''/g' /etc/makepkg.conf",
                    ])
                    .status()
                    .expect("Failed to change makepkg.conf");
                writeln!(
                    coloring("yellow"),
                    "Configuring BlackArch Linux repo in the chroot environment..."
                )
                .unwrap();
                let get_strap = Command::new(&devtools_nspawn)
                    .args(&[&chroot_root, "curl", "-O", "https://blackarch.org/strap.sh"])
                    .status()
                    .expect("Failed to retrieve strap.sh from blackarch.org");
                if get_strap.success() {
                    let strap_exec = Command::new(&devtools_nspawn)
                        .args(&[&chroot_root, "sh", "strap.sh"])
                        .status()
                        .expect("Failed to exec strap.sh");
                    if strap_exec.success() {
                        Command::new(&devtools_nspawn)
                            .args(&[&chroot_root, "rm", "strap.sh"])
                            .status()
                            .expect("Error deleting strap.sh from chroot environment.");
                        sync_chroot();
                        writeln!(coloring("green"), "Chroot environment setup complete!").unwrap();
                    } else {
                        writeln!(
                            coloring("red"),
                            "Can't install strap.sh into {}!",
                            &chroot_root
                        )
                        .unwrap();
                    }
                } else {
                    writeln!(
                        coloring("red"),
                        "Failed to retrieve strap.sh from blackarch.org!"
                    )
                    .unwrap();
                }
            } else {
                writeln!(
                    coloring("red"),
                    "Failed to install base packages into chroot environment."
                )
                .unwrap();
            }
        } else {
            writeln!(
                coloring("red"),
                "Executable file {} not found, install the devtools package from repos.",
                &devtools_mkarchroot
            )
            .unwrap();
        }
    }
}

pub fn update_chroot_packages() {
    writeln!(coloring("green"), "Updating the chroot environment...").unwrap();
    let devtools_nspawn = get_vars("nspawn");
    let chroot_root = get_vars("chroot_root");
    let update_packages = Command::new(&devtools_nspawn)
        .args(&[&chroot_root, "/bin/sh", "-c", "pacman --noconfirm -Syuu"])
        .status()
        .expect("Failed updating chroot environment");
    if update_packages.success() {
        sync_chroot();
        writeln!(coloring("green"), "Chroot environment updated correctly!").unwrap();
    } else {
        writeln!(
            coloring("red"),
            "An error as ocurred while updating the chroot environment."
        )
        .unwrap();
    }
}

pub fn build_package() {
    sync_chroot();
    if which::which("pkgcheck").is_ok() {
        writeln!(coloring("yellow"), "Checking PKGBUILD with pkgcheck...").unwrap();
        if Command::new("pkgcheck")
            .arg("PKGBUILD")
            .status()
            .expect("Failed to execute pkgcheck command")
            .success()
        {
            writeln!(coloring("green"), "No errors detected with pkgcheck!").unwrap();
        } else {
            writeln!(
                coloring("red"),
                "Some errors were detected, please fix them before pushing!"
            )
            .unwrap();
        }
    } else {
        writeln!(
            coloring("yellow"),
            "Consideer installing pkgcheck with pip install pkgcheck-arch --user for automatic PKGBUILD syntax error checking."
        )
        .unwrap()
    }
    let devtools_makechrootpkg = get_vars("makechrootpkg");
    let chroot_dir = get_vars("chroot_dir");
    let blackarch_instance = get_vars("blackarch_instance");
    let build_package = Command::new(&devtools_makechrootpkg)
        .args(&["-l", &blackarch_instance, "-r", &chroot_dir])
        .status()
        .expect("Failed to build the package.");
    if build_package.success() {
        writeln!(coloring("green"), "Package built sucessfully!").unwrap();
    } else {
        writeln!(coloring("red"), "Failed to build the package.").unwrap();
    }
}

pub fn build_package_with_missing_deps(missing: &[&str]) {
    sync_chroot();
    let chroot_blackarch = get_vars("chroot_blackarch");
    let devtools_nspawn = get_vars("nspawn");
    for missing in missing.iter() {
        let copy_path: String = [
            &chroot_blackarch,
            "root/",
            &missing
                .split("/")
                .last()
                .expect("Failed to get package name."),
        ]
        .concat();
        Command::new("sudo")
            .args(&["cp", &missing, &copy_path])
            .spawn()
            .expect("Failed to copy missing packages.");
    }
    writeln!(
        coloring("yellow"),
        "Installing missing packages: {:?}",
        missing
    )
    .unwrap();
    let install_missing = Command::new(&devtools_nspawn)
        .args(&[
            &chroot_blackarch,
            "/bin/sh",
            "-c",
            "pacman -U --noconfirm root/*",
        ])
        .status()
        .expect("Failed to install missing packages.");
    if install_missing.success() {
        let devtools_makechrootpkg = get_vars("makechrootpkg");
        let chroot_dir = get_vars("chroot_dir");
        let blackarch_instance = get_vars("blackarch_instance");
        let build_package = Command::new(&devtools_makechrootpkg)
            .args(&["-l", &blackarch_instance, "-r", &chroot_dir])
            .status()
            .expect("Failed to build the package.");
        if build_package.success() {
            writeln!(coloring("green"), "Package built sucessfully!").unwrap();
        } else {
            writeln!(coloring("red"), "Failed to build the package.").unwrap();
        }
        Command::new(&devtools_nspawn).args(&[&chroot_blackarch, "/bin/sh", "-c", "rm -rf root/*"]);
    }
}

pub fn sync_chroot() {
    let chroot_blackarch = get_vars("chroot_blackarch");
    let chroot_root = get_vars("chroot_root");
    let chroot_dir = get_vars("chroot_dir");
    writeln!(
        coloring("green"),
        "Syncing chroot copy {} with {} ...",
        &chroot_blackarch,
        &chroot_root
    )
    .unwrap();
    if Path::new(&chroot_dir).exists() {
        if Path::new(&chroot_blackarch).exists() {
            let fs_type = Command::new("stat")
                .args(&["-f", "-c", "%T", &chroot_blackarch])
                .output()
                .expect("Failed to read filesystem type");
            let fs_id_hex = Command::new("stat")
                .args(&["-c", "%i", &chroot_blackarch])
                .output()
                .expect("Failed to read filesystem ID");
            if String::from_utf8_lossy(&fs_type.stdout) == "btrfs"
                && String::from_utf8_lossy(&fs_id_hex.stdout) == "256"
            {
                Command::new("sudo")
                    .args(&["btrfs", "subvolume", "delete", &chroot_blackarch])
                    .status()
                    .expect("Failed to delete chroot copy.");
                Command::new("sudo")
                    .args(&[
                        "btrfs",
                        "subvolume",
                        "snapshot",
                        &chroot_root,
                        &chroot_blackarch,
                    ])
                    .spawn()
                    .expect("Failed to create chroot copy.");
            } else {
                let make_chroot_copy = Command::new("sudo")
                    .args(&[
                        "rsync",
                        "-a",
                        "--delete",
                        "-q",
                        "-W",
                        "-x",
                        &chroot_root,
                        &chroot_blackarch,
                    ])
                    .status()
                    .expect("Failed to create copy of root chroot environment.");
                if make_chroot_copy.success() {
                    writeln!(coloring("green"), "Chroot environment is ready!").unwrap();
                } else {
                    writeln!(
                        coloring("red"),
                        "Failed to create copy of root chroot environment."
                    )
                    .unwrap();
                }
            }
        } else {
            let create_chroot_copy = Command::new("sudo")
                .args(&["mkdir", &chroot_blackarch])
                .status()
                .expect("Failed to create working copy of chroot environment.");
            if create_chroot_copy.success() {
                let make_chroot_copy = Command::new("sudo")
                    .args(&[
                        "rsync",
                        "-a",
                        "--delete",
                        "-q",
                        "-W",
                        "-x",
                        &chroot_root,
                        &chroot_blackarch,
                    ])
                    .status()
                    .expect("Failed to create chroot copy.");
                if make_chroot_copy.success() {
                    writeln!(coloring("green"), "Chroot environment is ready!").unwrap();
                } else {
                    writeln!(
                        coloring("red"),
                        "Failed to create copy of root chroot environment."
                    )
                    .unwrap();
                }
            }
        }
    } else {
        writeln!(
            coloring("red"),
            "Chroot environment doesn't exists. Please use the -s option first."
        )
        .unwrap();
    }
}

pub fn test_package(package: &str, executable: &str) {
    let pacman = get_vars("pacman");
    let chroot_blackarch = get_vars("chroot_blackarch");
    let devtools_nspawn = get_vars("nspawn");
    let install_package = Command::new("sudo")
        .args(&[
            &pacman,
            "--root",
            &chroot_blackarch,
            "-U",
            "--noconfirm",
            &package,
        ])
        .status()
        .expect("Failed to install the package in the chroot environment.");
    if install_package.success() {
        writeln!(
            coloring("green"),
            "Package {} installed correctly! Testing it now...",
            &package
        )
        .unwrap();
        Command::new("sudo")
            .args(&[&devtools_nspawn, &chroot_blackarch, "sh", "-c", &executable])
            .status()
            .expect(
                "Something went wrong while trying to execute the binary in the chroot environment.",
            );
    } else {
        writeln!(
            coloring("red"),
            "Package {} wasn't installed in the chroot environment, please check the package name.",
            &package
        )
        .unwrap();
    }
}
