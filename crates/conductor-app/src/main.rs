mod app;
mod bridge;
mod runtime;
mod theme;
mod ui;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("starting conductor");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Conductor")
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 480.0])
            .with_titlebar_shown(false)
            .with_title_shown(false)
            .with_fullsize_content_view(true),
        ..Default::default()
    };

    eframe::run_native(
        "Conductor",
        native_options,
        Box::new(|cc| Ok(Box::new(app::ConductorApp::new(cc)))),
    )
}
