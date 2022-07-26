use std::process::Command;

use configparser::ini::Ini;

fn main() {
    let mut ini = Ini::new();
    ini.load("jetbrains-redir.ini")
        .expect("failed to load jetbrains-redir.ini");

    let exec = ini
        .get("jetbrains", "exec")
        .expect("jetbrains.exec is not set");

    let userprofile = ini
        .get("user", "profile")
        .unwrap_or_else(|| std::env::var("USERPROFILE").expect("$USERPROFILE is not set"));
    let appdata = ini
        .get("user", "appdata")
        .unwrap_or_else(|| std::env::var("AppData").expect("$AppData is not set"));
    let local_appdata = ini
        .get("user", "local-appdata")
        .unwrap_or_else(|| std::env::var("LocalAppData").expect("$LocalAppData is not set"));

    let userprofile = dunce::canonicalize(userprofile).expect("failed to stat userprofile");
    let appdata = dunce::canonicalize(appdata).expect("failed to stat appdata");
    let local_appdata = dunce::canonicalize(local_appdata).expect("failed to stat local_appdata");

    let override_java_user_home = ini
        .get("user", "override-java-user-home")
        .map(|s| {
            if s == "true" {
                true
            } else if s == "false" {
                false
            } else {
                panic!(
                    "override-java-user-home should be either \"true\" or \"false\"; got {:?}",
                    s,
                )
            }
        })
        .unwrap_or(false);

    if override_java_user_home && std::env::var_os("_JAVA_OPTIONS").is_some() {
        panic!("cannot override Java user.home while $_JAVA_OPTIONS has already been set");
    }

    eprintln!("USERPROFILE = {:?}", userprofile);
    eprintln!("APPDATA = {:?}", appdata);
    eprintln!("LOCAL_APPDATA = {:?}", local_appdata);

    let mut cmd = Command::new(exec);
    cmd.envs([
        ("USERPROFILE", &userprofile),
        ("AppData", &appdata),
        ("LocalAppData", &local_appdata),
    ]);
    if override_java_user_home {
        cmd.env(
            "_JAVA_OPTIONS",
            format!(
                "-Duser.home={}",
                userprofile
                    .as_os_str()
                    .to_str()
                    .expect("USERPROFILE is not valid UTF-8"),
            ),
        );
    }
    cmd.spawn().expect("failed to spawn child process");
}
