use std::process::ExitCode;

use fframes::cli::clap;
use fframes::{EncoderOptions, RenderOptions, cli};
use fframes_skia_renderer::cli::SkiaRenderers;
use layout_example::{LayoutMedia, TitleCard};

#[derive(Debug, clap::Args)]
struct Args {
    /// `16x9` (a lower third) or `9x16` (a centred card).
    #[arg(long, global = true, default_value = "16x9")]
    format: String,
}

fn main() -> ExitCode {
    let args = cli::parse::<Args>();
    match args.app.format.as_str() {
        "16x9" => run::<1920, 1080>(args),
        "9x16" => run::<1080, 1920>(args),
        other => {
            eprintln!("unknown format {other}: use 16x9 or 9x16");
            ExitCode::FAILURE
        }
    }
}

fn run<const W: usize, const H: usize>(args: cli::Cli<Args>) -> ExitCode {
    let media = LayoutMedia::new().expect("layout media");
    let video = TitleCard::<W, H>::default();
    cli::new(
        &video,
        RenderOptions {
            media: Some(&media),
            video_encoder_options: EncoderOptions {
                preferred_encoder: Some("libx264"),
                codec_params: Some(&[("crf", "20"), ("preset", "medium"), ("tune", "animation")]),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .args(args)
    .default_output(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("output/layout-{W}x{H}.mp4")),
    )
    .renderers(SkiaRenderers::default())
    .run()
}
