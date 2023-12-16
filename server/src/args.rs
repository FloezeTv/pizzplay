use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Timeout of images in seconds
    #[arg(short, long, default_value_t = 10)]
    pub image_timeout: u64,

    /// Offset of images
    #[arg(short, long, default_value_t = 2)]
    pub image_offset: usize,

	#[arg(short, long, default_value = "./images.json")]
	pub image_path: String,
}

pub fn parse() -> Args {
    Args::parse()
}
