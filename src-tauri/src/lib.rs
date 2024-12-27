use fast_image_resize::{images::Image as FirImage, PixelType, Resizer};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{path::Path, sync::Mutex, thread};
use tauri::{Emitter, Manager, Size};
use webp::Encoder;

#[derive(Default)]
struct AppState {
    pub cancel: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Image {
    full_path: String,
    filename: String,
    extension: String,
    path: String,
    original_size: u32,
    #[serde(skip_deserializing)]
    webp_size: u32,
    #[serde(skip_deserializing)]
    processed: bool,
}

impl Image {
    pub fn new(
        full_path: String,
        filename: String,
        extension: String,
        path: String,
        original_size: u32,
    ) -> Image {
        Image {
            full_path,
            filename,
            extension,
            path,
            original_size,
            webp_size: 0,
            processed: false,
        }
    }
}

#[derive(Copy, Clone, Debug, serde::Deserialize)]
enum Resize {
    NoResizing,
    LongerSide,
    ShorterSide,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Parameters {
    is_lossless: bool,
    quality: u32,
    resize: Resize,
    resize_to: u32,
    is_enlarging_allowed: bool,
    save_folder: String,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Success {
    full_path: String,
    size: u64,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessError {
    full_path: String,
    error: String,
}

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// struct ImagesList {
//     images: Vec<Image>,
// }

// impl ImagesList {
//     pub fn new() -> ImagesList {
//         ImagesList { images: Vec::new() }
//     }
// }

pub fn resize_dynamic_image(
    image: DynamicImage,
    new_width: u32,
    new_height: u32,
) -> anyhow::Result<DynamicImage> {
    // Convert the DynamicImage to an RGBA ImageBuffer
    let (width, height) = image.dimensions();
    let rgba_image = image.to_rgba8();

    // Create a fast_image_resize::Image from the ImageBuffer
    let src_image = FirImage::from_vec_u8(width, height, rgba_image.into_raw(), PixelType::U8x4)?;

    // Create a destination image
    let mut dst_image = FirImage::new(new_width, new_height, PixelType::U8x4);

    // Create a resizer and set the resize algorithm (e.g., Lanczos3)
    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None)?;

    // Convert the resized image back into an ImageBuffer
    let resized_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = match ImageBuffer::from_raw(
        new_width as u32,
        new_height as u32,
        dst_image.buffer().to_vec(),
    ) {
        Some(buffer) => buffer,
        None => panic!("Failed to create ImageBuffer from resized data"),
    };

    // Convert the ImageBuffer into a DynamicImage and return it
    Ok(DynamicImage::ImageRgba8(resized_buffer))
}

fn resize_if_needed(
    src_image: DynamicImage,
    resize: Resize,
    resize_to: u32,
    is_enlarging_allowed: bool,
) -> anyhow::Result<DynamicImage> {
    match resize {
        Resize::NoResizing => Ok(src_image),
        Resize::LongerSide => {
            let current_width = src_image.width();
            let current_height = src_image.height();
            if !is_enlarging_allowed && current_width < resize_to && current_height < resize_to {
                Ok(src_image)
            } else {
                let is_width_longer = current_width > current_height;
                let new_width = if is_width_longer {
                    resize_to
                } else {
                    current_width * resize_to / current_height
                };
                let new_height = if !is_width_longer {
                    resize_to
                } else {
                    current_height * resize_to / current_width
                };
                resize_dynamic_image(src_image, new_width, new_height)
            }
        }
        Resize::ShorterSide => {
            let current_width = src_image.width();
            let current_height = src_image.height();
            if !is_enlarging_allowed && current_width < resize_to && current_height < resize_to {
                Ok(src_image)
            } else {
                let is_width_longer = current_width > current_height;
                let new_width = if is_width_longer {
                    current_width * resize_to / current_height
                } else {
                    resize_to
                };
                let new_height = if !is_width_longer {
                    current_height * resize_to / current_width
                } else {
                    resize_to
                };
                resize_dynamic_image(src_image, new_width, new_height)
            }
        }
    }
}

fn process_image(
    app: &tauri::AppHandle,
    image: &Image,
    parameters: &Parameters,
) -> anyhow::Result<u64> {
    let src_image = ImageReader::open(image.full_path.as_str())
        .unwrap()
        .decode()
        .unwrap();
    let sized_image = resize_if_needed(
        src_image,
        parameters.resize,
        parameters.resize_to,
        parameters.is_enlarging_allowed,
    )
    .unwrap();
    let encoder = match Encoder::from_image(&sized_image) {
        Ok(the_encoder) => the_encoder,
        Err(_) => return Err(anyhow::anyhow!("Image can't be converted")),
    };
    let encoded = match encoder.encode_simple(parameters.is_lossless, parameters.quality as f32) {
        Ok(the_encoded) => the_encoded,
        Err(_) => return Err(anyhow::anyhow!("Image can't be converted")),
    };
    let directory_path = if parameters.save_folder != "" {
        parameters.save_folder.as_str()
    } else {
        image.path.as_str()
    };
    if !is_cancel(app) {
        let output_path = Path::new(directory_path)
            .join(image.filename.as_str())
            .with_extension("webp");
        let file_size = match std::fs::write(&output_path, &*encoded) {
            Ok(_) => get_file_size(&output_path),
            Err(_) => return Err(anyhow::anyhow!("File can't be saved")),
        };
        Ok(file_size)
    } else {
        Ok(0)
    }
}

fn get_file_size(path: &std::path::PathBuf) -> u64 {
    match std::fs::metadata(path) {
        Ok(metadata) => metadata.len() / 1024,
        Err(_) => 0,
    }
}

fn is_cancel(app: &tauri::AppHandle) -> bool {
    let state = app.state::<Mutex<AppState>>();
    let state = state.lock().unwrap();
    state.cancel
}

#[tauri::command]
fn process(app: tauri::AppHandle, images: Vec<Image>, parameters: Parameters) {
    thread::spawn(move || {
        images.par_iter().for_each(|image| {
            let is_cancel = is_cancel(&app);
            if !is_cancel {
                app.emit("progress", image.full_path.clone()).unwrap();
                match process_image(&app, image, &parameters) {
                    Ok(file_size) => {
                        if file_size > 0 {
                            app.emit(
                                "success",
                                Success {
                                    full_path: image.full_path.clone(),
                                    size: file_size,
                                },
                            )
                            .unwrap()
                        }
                    }
                    Err(error) => app
                        .emit(
                            "error",
                            ProcessError {
                                full_path: image.full_path.clone(),
                                error: error.to_string(),
                            },
                        )
                        .unwrap(),
                };
            }
        });
        set_status(app, false);
    });
}

fn set_status(app: tauri::AppHandle, is_cancel: bool) {
    let state = app.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();
    state.cancel = is_cancel;
}

#[tauri::command]
fn cancel_process(app: tauri::AppHandle) {
    thread::spawn(move || {
        set_status(app, true);
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![process, cancel_process])
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            let main_window = app.get_webview_window("main").unwrap();
            main_window.set_title("BlazingWebP")?;
            let min_size: Size = Size::Physical(tauri::PhysicalSize::new(800, 1000));
            main_window.set_min_size(min_size.into())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
