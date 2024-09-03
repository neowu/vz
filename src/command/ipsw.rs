use std::sync::mpsc::channel;

use block2::StackBlock;
use clap::Args;
use objc2_foundation::NSError;
use objc2_virtualization::VZMacOSRestoreImage;

#[derive(Args)]
pub struct Ipsw;

impl Ipsw {
    pub fn execute(&self) {
        let (tx, rx) = channel();
        let block = StackBlock::new(move |image: *mut VZMacOSRestoreImage, err: *mut NSError| {
            if !err.is_null() {
                panic!("failed to fetch macos image, err={}", unsafe { (*err).localizedDescription() });
            } else {
                let url = unsafe { (*image).URL().absoluteString().unwrap() };
                tx.send(url).unwrap();
            }
        });
        unsafe {
            VZMacOSRestoreImage::fetchLatestSupportedWithCompletionHandler(&block);
        };
        let url = rx.recv().unwrap();
        println!("{}", url);
    }
}
