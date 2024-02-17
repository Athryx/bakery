mod breadboard;
mod ftd_data;
pub mod prelude;

use std::{path::PathBuf, str::FromStr};

pub use breadboard::{Breadboard, SwitchOptions};

fn find_ftd_folder() -> PathBuf {
    // FIXME: this is just hardcoded for me for now
    PathBuf::from_str("/home/jack/From The Depths/Player Profiles/DeltaForce").unwrap()
}

fn find_prefabs_folder() -> PathBuf {
    find_ftd_folder().join("PrefabsVersion2")
}