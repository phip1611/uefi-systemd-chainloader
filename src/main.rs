#![no_std]
#![no_main]

use log::{error, info};
use core::time::Duration;
use uefi::boot::{LoadImageSource, OpenProtocolAttributes, OpenProtocolParams};
use uefi::fs::FileSystem;
use uefi::proto::device_path::DevicePath;
use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::{CStr16, Handle, boot, cstr16, helpers, system, table};


/// This function chainloads the first systemd-bootx64.efi it finds.
///
/// It iterates all disks and looks for the corresponding EFI file. The first one is loaded.
fn chainload() -> anyhow::Result<()> {
    const PATH: &'static CStr16 = cstr16!("EFI\\systemd\\systemd-bootx64.efi");

    let handles = boot::find_handles::<SimpleFileSystem>()?;
    info!(
        "found {} handle(s) implementing the SimpleFileSystem protocol",
        handles.len()
    );
    for handle in handles.iter() {
        let sfp_protocol = unsafe {
            boot::open_protocol::<SimpleFileSystem>(
                OpenProtocolParams {
                    handle: *handle,
                    agent: boot::image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )?
        };

        let dp_protocol = unsafe {
            boot::open_protocol::<DevicePath>(
                OpenProtocolParams {
                    handle: *handle,
                    agent: boot::image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )?
        };

        let dp_string = dp_protocol.to_string(DisplayOnly(true), AllowShortcuts(false))?;

        info!("Found disk: {dp_string}");

        let mut fs = FileSystem::new(sfp_protocol);
        let exists = fs.try_exists(PATH)?;

        if exists {
            info!("Found systemd-bootx64.efi on disk: {dp_string}",);
            info!("Booting in 3 ..",);
            boot::stall(Duration::from_secs(1));
            info!("Booting in 2 ..",);
            boot::stall(Duration::from_secs(1));
            info!("Booting in 1 ..",);
            boot::stall(Duration::from_secs(1));

            let efi_bytes = fs.read(PATH)?;
            // We will never return here.
            let image = boot::load_image(
                boot::image_handle(),
                LoadImageSource::FromBuffer {
                    buffer: &efi_bytes,
                    file_path: None,
                },
            )?;

            boot::start_image(image)?;
            unreachable!();
        } else {

        }
    }
    panic!("didn't find systemd-bootx64.efi on any partition");
}

fn inner_main() -> anyhow::Result<()> {
    helpers::init()?;

    info!("Hello World from uefi_std");
    info!("UEFI revision: {}", system::uefi_revision());
    chainload()?;
    Ok(())
}

#[uefi::entry]
fn main() -> uefi::Status {
    if let Err(e) = inner_main() {
        error!("\n{e:?}");
    }
    loop {
        core::hint::spin_loop();
    }
}
