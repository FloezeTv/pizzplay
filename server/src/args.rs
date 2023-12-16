use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Timeout of images in seconds
    #[arg(long, default_value_t = 10)]
    pub image_timeout: u64,

    /// Offset of images
    #[arg(long, default_value_t = 2)]
    pub image_offset: usize,

    #[arg(long, default_value = "./images.json")]
    pub image_path: String,

    /// Time to show popups for in seconds
    #[arg(long, default_value_t = 5)]
    pub popup_show: u64,

    /// Time to wait between popups in seconds
    #[arg(long, default_value_t = 1)]
    pub popup_wait: u64,
}

pub fn parse() -> Args {
    Args::parse()
}
