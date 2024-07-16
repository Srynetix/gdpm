use gdpm_core::{downloader::DownloadAdapter, io::IoAdapter};

pub struct Context<I: IoAdapter, D: DownloadAdapter> {
    io_adapter: I,
    download_adapter: D,
}

impl<I: IoAdapter, D: DownloadAdapter> Context<I, D> {
    pub fn new(io_adapter: I, download_adapter: D) -> Self {
        Self {
            io_adapter,
            download_adapter,
        }
    }

    pub fn io(&self) -> &I {
        &self.io_adapter
    }

    pub fn download(&self) -> &D {
        &self.download_adapter
    }
}
