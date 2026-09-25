// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;

fn main() {
    for (name, value) in env::vars() {
        if let Some(cfg) = name.strip_prefix("DEP_FFMPEG_") {
            let cfg = cfg.to_lowercase();
            if value == "true" {
                println!("cargo::rustc-cfg={cfg}");
            }
            println!("cargo::rustc-check-cfg=cfg({cfg})");
        }
    }
}
