#![no_std]
#![no_main]

extern crate alloc;

use {
    alloc::vec,
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
            BootPolicy,
            device_path::{
                DevicePath,
                build::{
                    self,
                    DevicePathBuilder,
                },
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

        // Device path of the disk with the FAT file system
        let dp_disk = unsafe {
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
        info!("Found disk with device path: {}", dp_disk.to_string(DisplayOnly(true), AllowShortcuts(false))?);

        let mut fs = FileSystem::new(sfp_protocol);
        let exists = fs.try_exists(PATH)?;

        if !exists {
            info!("Didn't find file {PATH}");
        } else {
            info!("Found file {PATH}");

            let mut db_builder_buf = vec![];
            let dp_file: &DevicePath = DevicePathBuilder::with_vec(&mut db_builder_buf)
                .push(&build::media::FilePath { path_name: PATH })?
                .finalize()?;
            let dp_full = dp_disk.append_path(dp_file)?;

            info!(
                "File's full device path: {}",
                dp_full.to_string(DisplayOnly(true), AllowShortcuts(false))?
            );

            info!("Booting in 3 ..",);
            boot::stall(Duration::from_secs(1));
            info!("Booting in 2 ..",);
            boot::stall(Duration::from_secs(1));
            info!("Booting in 1 ..",);
            boot::stall(Duration::from_secs(1));

            // We will never return here.
            let image = boot::load_image(
                boot::image_handle(),
                LoadImageSource::FromDevicePath {
                    device_path: &dp_full,
                    boot_policy: BootPolicy::ExactMatch,
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
