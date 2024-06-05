use std::sync::mpsc::channel;

use block2::StackBlock;
use clap::Args;
use objc2_foundation::NSError;
use objc2_virtualization::VZMacOSRestoreImage;

use crate::util::exception::Exception;

#[derive(Args)]
pub struct Ipsw;

impl Ipsw {
    pub fn execute(&self) -> Result<(), Exception> {
        let (tx, rx) = channel();
        let block = StackBlock::new(move |image: *mut VZMacOSRestoreImage, err: *mut NSError| {
            if !err.is_null() {
                tx.send(Err(Exception::from_ns_error(err))).unwrap();
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
