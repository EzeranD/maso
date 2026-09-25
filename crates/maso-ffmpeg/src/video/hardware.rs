// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ffmsys::AVHWDeviceType::{
    AV_HWDEVICE_TYPE_D3D11VA, AV_HWDEVICE_TYPE_VAAPI, AV_HWDEVICE_TYPE_VULKAN,
};

use crate::codec::{
    Codec,
    hw_config::{HwConfig, Methods},
    parameters::Parameters,
};

pub struct Candidate {
    pub codec: Codec,
    pub config: HwConfig,
}

impl Candidate {
    pub fn from_parameters(parameters: &Parameters) -> Vec<Self> {
        let mut candidates = Vec::new();

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        return candidates;

        let id = parameters.codec_id();

        for codec in Codec::iter().filter(|c| c.id() == id && c.is_decoder()) {
            for config in HwConfig::from_codec(codec) {
                if !config.methods().contains(Methods::HW_DEVICE_CTX) {
                    continue;
                }

                let device_type = config.device_type().as_raw();
                #[cfg(target_os = "linux")]
                if !matches!(
                    device_type,
                    AV_HWDEVICE_TYPE_VAAPI | AV_HWDEVICE_TYPE_VULKAN
                ) {
                    continue;
                }

                #[cfg(target_os = "windows")]
                if !matches!(
                    device_type,
                    AV_HWDEVICE_TYPE_D3D11VA | AV_HWDEVICE_TYPE_VULKAN
                ) {
                    continue;
                }

                candidates.push(Candidate { codec, config });
            }
        }

        candidates.sort_by_key(|opt| match opt.config.device_type().as_raw() {
            AV_HWDEVICE_TYPE_VAAPI | AV_HWDEVICE_TYPE_D3D11VA => 2,
            AV_HWDEVICE_TYPE_VULKAN => 1,
            _ => 0,
        });

        candidates
    }
}
