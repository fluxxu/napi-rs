extern crate cfg_if;
#[cfg(windows)]
extern crate reqwest;

use std::process::Command;

use cfg_if::cfg_if;

cfg_if! {
  if #[cfg(windows)] {
    use std::env::var;
    use std::fs::{File, create_dir};
    use std::io::copy;
    use std::path::PathBuf;

    pub fn setup() {
      let node_full_version =
        String::from_utf8(Command::new("node").arg("-v").output().unwrap().stdout).unwrap();

      let dev_dir: PathBuf = [
        &var("HOMEDRIVE").expect("Get env HOMEDRIVE failed"),
        &var("HOMEPATH").expect("Get env HOMEPATH failed"),
        ".napi-rs"
      ].iter().collect();

      match create_dir(&dev_dir) {
        Ok(_) => {},
        Err(err) => {
          if err.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("create ~/.napi-rs folder failed: {}", err)
          }
        },
      }

      let node_lib_file_dir = dev_dir.join(format!("node-{}.lib", node_full_version.trim_end()));
      if !node_lib_file_dir.exists() {
        let lib_file_download_url = format!(
          "https://nodejs.org/dist/{}/win-x64/node.lib",
          node_full_version
        );
        let mut resp =
          reqwest::blocking::get(&lib_file_download_url).expect("Download node.lib file failed");
        let mut node_lib_file = File::create(&node_lib_file_dir).unwrap();
        copy(&mut resp, &mut node_lib_file).expect("Save node.lib file failed");
      }
      println!(
        "cargo:rustc-link-lib={}",
        &node_lib_file_dir.file_stem().unwrap().to_str().unwrap()
      );
      println!("cargo:rustc-link-search={}", dev_dir.to_str().unwrap());
      // Link `win_delay_load_hook.obj` for windows electron
      let node_runtime_env = "npm_config_runtime";
      println!("cargo:rerun-if-env-changed={}", node_runtime_env);
      if var(node_runtime_env).map(|s| s == "electron") == Ok(true) {
        println!("cargo:rustc-cdylib-link-arg=win_delay_load_hook.obj");
        println!("cargo:rustc-cdylib-link-arg=delayimp.lib");
        println!("cargo:rustc-cdylib-link-arg=/DELAYLOAD:node.exe");
      }
      setup_napi_feature();
    }

    fn ensure_dev_dir(subfolder: &str) -> PathBuf {
      let dev_dir: PathBuf = [
        &var("HOMEDRIVE").expect("Get env HOMEDRIVE failed"),
        &var("HOMEPATH").expect("Get env HOMEPATH failed"),
        ".napi-rs",
        subfolder
      ].iter().collect();


    }
  } else if #[cfg(target_os = "macos")] {
    /// Set up the build environment by setting Cargo configuration variables.
    pub fn setup() {
      println!("cargo:rustc-cdylib-link-arg=-Wl");
      println!("cargo:rustc-cdylib-link-arg=-undefined");
      println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
      setup_napi_feature();
    }
  } else {
    pub fn setup() {
      setup_napi_feature();
    }
  }
}

fn setup_napi_feature() {
  let napi_version = String::from_utf8(
    Command::new("node")
      .args(&["-e", "console.log(process.versions.napi)"])
      .output()
      .unwrap()
      .stdout,
  )
  .expect("Get NAPI version failed");

  let napi_version_number = napi_version.trim().parse::<u32>().unwrap();

  if napi_version_number < 2 {
    panic!("current napi version is too low");
  }

  if napi_version_number == 2 {
    println!("cargo:rustc-cfg=napi{}", napi_version_number);
  } else {
    for version in 2..(napi_version_number + 1) {
      println!("cargo:rustc-cfg=napi{}", version);
    }
  }
}
