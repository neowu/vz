use std::sync::mpsc::channel;
use std::sync::mpsc::RecvError;

use block2::StackBlock;
use clap::command;
use clap::Args;
use objc2_foundation::NSError;
use objc2_virtualization::VZMacOSRestoreImage;

use crate::util::exception::Exception;

#[derive(Args)]
#[command(
    about = "Get macOS restore image ipsw url",
    long_about = "Get macOS restore image ipsw url, download ipsw file manually, then use in create command with --ipsw"
)]
pub struct Ipsw;

impl Ipsw {
    pub async fn execute(&self) -> Result<(), Exception> {
        let (tx, rx) = channel();
        unsafe {
            let block = StackBlock::new(move |image: *mut VZMacOSRestoreImage, error: *mut NSError| {
                if !error.is_null() {
                    tx.send(Err(Exception::new((*error).localizedDescription().to_string()))).unwrap();
                } else {
                    let url = (*image).URL().absoluteString();
                    tx.send(Ok(url.unwrap().to_string())).unwrap();
                }
            });
            VZMacOSRestoreImage::fetchLatestSupportedWithCompletionHandler(&block);
        };
        let url = rx.recv()??;
        println!("{url}");
        Ok(())
    }
}

impl From<RecvError> for Exception {
    fn from(err: RecvError) -> Self {
        Exception::new(err.to_string())
    }
}
