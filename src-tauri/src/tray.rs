use crate::session;
use crate::windows;
use humantime::format_duration;
use image::{ExtendedColorType, ImageBuffer, ImageEncoder, Rgb, Rgba};
use rusttype::{Font, Scale};
use std::{error::Error, sync::Arc};
use tauri::{
    image::Image as TauriImage,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, Manager,
};
use tokio::sync::Mutex;
use tokio::time::Duration;
fn generate_tray_icon(
    text_color: Rgb<u8>,
    number: i32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let width = 64;
    let height = 64;
    let mut img = ImageBuffer::new(width, height);

    // set transparent background
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }

    // load font
    let font_data = include_bytes!("../assets/fonts/Roboto-Bold.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).unwrap();

    // configure font size
    let scale = if number >= 100 {
        Scale { x: 38.0, y: 38.0 }
    } else {
        Scale { x: 60.0, y: 60.0 }
    };
    let text = format!("{}", number);

    // calculate text position
    let width_in_pixels = font
        .layout(&text, scale, rusttype::point(0.0, 0.0))
        .map(|g| g.pixel_bounding_box().unwrap().width())
        .sum::<i32>();

    let height_in_pixels = font
        .layout(&text, scale, rusttype::point(0.0, 0.0))
        .map(|g| g.pixel_bounding_box().unwrap().height())
        .max()
        .unwrap_or(0);
    let x = (width as i32 - width_in_pixels) / 2;
    let y = (height as i32 - height_in_pixels) / 2 + height_in_pixels;
    for glyph in font.layout(&text, scale, rusttype::point(x as f32, y as f32)) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, v| {
                let px = bb.min.x + gx as i32;
                let py = bb.min.y + gy as i32;
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    let alpha = (v * 255.0) as u8;
                    img.put_pixel(
                        px as u32,
                        py as u32,
                        Rgba([text_color[0], text_color[1], text_color[2], alpha]),
                    );
                }
            });
        }
    }

    // convert image to PNG bytes
    let mut buffer = Vec::new();
    image::codecs::png::PngEncoder::new(&mut buffer).write_image(
        &img,
        width,
        height,
        ExtendedColorType::Rgba8,
    )?;

    Ok(buffer)
}
pub async fn update_tray_icon(tray: &TrayIcon) {
    let state = tray
        .app_handle()
        .state::<Arc<Mutex<session::SessionState>>>();
    let mut state = state.lock().await;
    let mut tray_number = (state.system.cpuload * 10000.0).round() / 100.0;
    let mut color = Rgb([255, 255, 0]);
    let mut tooltip: String = String::from("Unknown");
    if let Some(battery) = &state.battery {
        match battery.state {
            battery::State(battery::ExternalBatteryState::Charging)
            | battery::State(battery::ExternalBatteryState::Discharging)
            | battery::State(battery::ExternalBatteryState::Empty) => {
                tray_number = (battery.energy_rate * 100.0).round() / 100.0;
            }
            _ => (),
        };
        match battery.state {
            battery::State(battery::ExternalBatteryState::Charging) => color = Rgb([0, 255, 0]),
            battery::State(battery::ExternalBatteryState::Discharging) => color = Rgb([255, 0, 0]),
            battery::State(battery::ExternalBatteryState::Empty) => color = Rgb([255, 0, 255]),
            _ => (),
        };
        tooltip = match battery.state {
            battery::State(battery::ExternalBatteryState::Charging) => format!(
                "Charging, estimated charging time {}",
                format_duration(Duration::new(battery.time_to_full_secs, 0))
            ),
            battery::State(battery::ExternalBatteryState::Discharging) => format!(
                "Discharging, estimated discharging time {}",
                format_duration(Duration::new(battery.time_to_empty_secs, 0))
            ),
            battery::State(battery::ExternalBatteryState::Empty) => {
                format!(
                    "Empty, percentage {}%",
                    battery.percentage / 100.0 * 100.0 * 100.0
                )
            }
            battery::State(battery::ExternalBatteryState::Full) => {
                format!("Full, Cpu load {}%", tray_number)
            }
            _ => "Battery Monitor".to_string(),
        };
    }
    let icon_bytes = generate_tray_icon(color, tray_number.abs() as i32).unwrap();
    let result = tray.set_icon(TauriImage::from_bytes(&icon_bytes).ok());
    if result.is_err() {
        println!("tray.set_icon is err.{:?}", result);
    }

    let result = tray.set_tooltip(Some(tooltip));
    if result.is_err() {
        println!("tray.set_tooltip err.{:?}", result);
    }
}
pub fn build(app: &App, id: &str) {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
    let admin_i =
        MenuItem::with_id(app, "admin", "Run at Administrator", true, None::<&str>).unwrap();

    let menu = Menu::with_items(app, &[&admin_i, &quit_i]).unwrap();
    TrayIconBuilder::with_id(id)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "admin" => {
                let state = app.state::<Arc<Mutex<session::SessionState>>>();
                // move state into async task
                tokio::spawn({
                    let state = Arc::clone(&state); // clone Arc to pass to async task
                    async move {
                        let state = state.lock().await; // acquire Mutex lock asynchronously
                        if !state.is_admin {
                            windows::elevate_self();
                        }
                    }
                });
            }
            _ => (),
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let app = tray.app_handle();
                windows::active_window(app, "main");
                let state = app.state::<Arc<Mutex<session::SessionState>>>();
                tokio::spawn({
                    let state = Arc::clone(&state);
                    async move {
                        let mut state = state.lock().await; // acquire Mutex lock asynchronously
                        state.is_min_tray = false; // modify state
                    }
                });
            }
            _ => (),
        })
        .build(app)
        .unwrap();
}
