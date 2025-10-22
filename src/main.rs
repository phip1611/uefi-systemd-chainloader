#![no_std]
#![no_main]

use {
    core::time::Duration,
    log::{
        error,
        info,
    },
    uefi::{
        CStr16,
        boot::{
            self,
            LoadImageSource,
            OpenProtocolAttributes,
            OpenProtocolParams,
        },
        cstr16,
        fs::FileSystem,
        helpers,
        proto::{
            device_path::{
                DevicePath,
                text::{
                    AllowShortcuts,
                    DisplayOnly,
                },
            },
            media::fs::SimpleFileSystem,
        },
        system,
    },
};

/// This function chainloads the first `systemd-bootx64.efi` it finds.
///
/// It iterates all disks and looks for the corresponding EFI file. The first
/// one will be loaded and started.
fn chainload() -> anyhow::Result<()> {
    const PATH: &'static CStr16 = cstr16!("EFI\\systemd\\systemd-bootx64.efi");

    // Find all handles implementing the SimpleFileSystem protocol.
    // In other words: Handles to all resources representing disks.
    let handles = boot::find_handles::<SimpleFileSystem>()?;
    info!(
        "found {} handle{} implementing the SimpleFileSystem protocol",
        handles.len(),
        if handles.len() == 1 { "" } else { "s" }
    );

    // Iterate all handles and search for a matching disk/matching file system
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

        // Device path of the handle
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

        // Stringified device path
        let dp_string = dp_protocol.to_string(DisplayOnly(true), AllowShortcuts(false))?;

        info!("Found disk: {dp_string}");

        let mut fs = FileSystem::new(sfp_protocol);
        let exists = fs.try_exists(PATH)?;

        if exists {
            info!("Found file {PATH}");
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

            // This will never return
            boot::start_image(image)?;
            unreachable!();
        }
    }
    panic!("didn't find systemd-bootx64.efi on any partition");
}

fn inner_main() -> anyhow::Result<()> {
    helpers::init()?;

    info!("Hello World from uefi-systemd-chainloader");
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
        error!("Reached end of main() function");
        core::hint::spin_loop();
    }
}
