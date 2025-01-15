#![forbid(unsafe_code)]
use axoverlay::{
    AnchorPoint, Axoverlay, Callback, Colorspace, Overlay, OverlayBuilder, PositionType, Settings,
    Stream,
};
use log::error;

fn render_cb(
    id: i32,
    stream: &mut Stream,
    postype: &PositionType,
    overlay_x: f32,
    overlay_y: f32,
    overlay_width: u32,
    overlay_height: u32,
) {
}

fn main() {
    acap_logging::init_logger();
    error!("Starting ACAP");
    let settings = Settings::default().set_callback(axoverlay::Callback::Render(render_cb));
    error!("Got settings");

    let axoverlay = Axoverlay::new(settings).unwrap();
    error!("initialized axoverlay");

    let overlay = axoverlay
        .create_overlay(Some(
            OverlayBuilder::new()
                .postype(PositionType::CustomNormalized)
                .x(0.0)
                .y(0.0)
                .width(144)
                .height(144)
                .colorspace(Colorspace::Arg32),
        ))
        .unwrap();
    error!("generated overlay");

    axoverlay.redraw().unwrap();

    error!("Redrew overlay");

    loop {}
}
