use std::sync::mpsc::channel;

use anyhow::Result;
use block2::StackBlock;
use clap::Args;
use objc2_foundation::NSError;
use objc2_virtualization::VZMacOSRestoreImage;

use crate::util::objc::ObjcError;

#[derive(Args)]
pub struct Ipsw;

impl Ipsw {
    pub fn execute(&self) -> Result<()> {
        let (tx, rx) = channel();
        let block = StackBlock::new(move |image: *mut VZMacOSRestoreImage, err: *mut NSError| {
            if !err.is_null() {
                tx.send(Err(ObjcError::from(err))).unwrap();
            } else {
                let url = unsafe { (*image).URL().absoluteString().unwrap() };
                tx.send(Ok(url)).unwrap();
            }
        });
        unsafe {
            VZMacOSRestoreImage::fetchLatestSupportedWithCompletionHandler(&block);
        };
        let url = rx.recv()??;
        println!("{}", url);
        Ok(())
    }
}
