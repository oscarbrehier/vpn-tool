// use std::os::windows::ffi::OsStrExt;
// use std::{collections::HashMap, path::PathBuf};

// use std::ffi::OsStr;
// use base64::Engine;
// use serde::Serialize;
// use sysinfo::System;
// use winapi::um::shellapi::ExtractIconW;
// use winapi::um::wingdi::{BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDIBits, GetObjectW, RGBQUAD, SelectObject};
// use winapi::um::winuser::{DestroyIcon, DrawIconEx, GetDC, GetIconInfo, ICONINFO, ReleaseDC};
// use windows::Win32::UI::WindowsAndMessaging::DI_NORMAL;

// #[derive(Debug, Serialize, Clone)]
// pub struct AppGroup {
//     pub pids: Vec<u32>,
//     pub name: String,
//     pub path: Option<PathBuf>,
//     pub icon_base64: Option<String>,
// }

// pub fn get_running_apps<F>(get_cached_icon: F) -> Result<Vec<AppGroup>, String>
// where 
//     F: Fn(&PathBuf) -> Option<String>
// {
//     let mut sys = System::new_all();
//     sys.refresh_all();

//     let mut grouped_processes: HashMap<String, AppGroup> = HashMap::new();

//     for (pid, process) in sys.processes() {
//         if let Some(path) = process.exe() {
//             let path_str = path.to_string_lossy();

//             let is_user_app = path_str.contains("Program Files")
//                 || path_str.contains("AppData")
//                 || path_str.contains("Users");

//             if is_user_app {
//                 let name = process.name().to_string_lossy().replace(".exe", "");

//                 let entry = grouped_processes.entry(name.clone()).or_insert(AppGroup {
//                     name: name,
//                     path: Some(path.to_path_buf()),
//                     icon_base64: get_cached_icon(&path.to_path_buf()),
//                     pids: Vec::new(),
//                 });

//                 entry.pids.push(pid.as_u32());
//             }
//         }
//     }

//     Ok(grouped_processes.values().cloned().collect())
// }

// pub fn get_windows_icon(path: &PathBuf) -> Option<String> {
   
//     unsafe {

//         let wide_path: Vec<u16> = OsStr::new(path.as_os_str())
//             .encode_wide()
//             .chain(std::iter::once(0))
//             .collect();

//         let hicon = ExtractIconW(std::ptr::null_mut(), wide_path.as_ptr(), 0);
//         if hicon.is_null() || hicon as usize == 1 {
//             return None;
//         }

//         let mut icon_info = ICONINFO {
//             fIcon: 0,
//             xHotspot: 0,
//             yHotspot: 0,
//             hbmMask: std::ptr::null_mut(),
//             hbmColor: std::ptr::null_mut()
//         };

//         if GetIconInfo(hicon, &mut icon_info) == 0 {
//             DestroyIcon(hicon);
//             return None;
//         }

//         let mut bmp = BITMAP {
//             bmType: 0,
//             bmWidth: 0,
//             bmHeight: 0,
//             bmWidthBytes: 0,
//             bmPlanes: 0,
//             bmBitsPixel: 0,
//             bmBits: std::ptr::null_mut()
//         };

//         GetObjectW(icon_info.hbmColor as _, std::mem::size_of::<BITMAP>() as i32, &mut bmp as *mut _ as _,);

//         let width = bmp.bmWidth as usize;
//         let height = bmp.bmHeight as usize;

//         let hdc_screen = GetDC(std::ptr::null_mut());
//         let hdc_mem = CreateCompatibleDC(hdc_screen);
//         let mut bmi = BITMAPINFO {
//             bmiHeader: BITMAPINFOHEADER {
//                 biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
//                 biWidth: width as i32,
//                 biHeight: -(height as i32),
//                 biPlanes: 1,
//                 biBitCount: 32,
//                 biCompression: BI_RGB,
//                 biSizeImage: 0,
//                 biXPelsPerMeter: 0,
//                 biYPelsPerMeter: 0,
//                 biClrUsed: 0,
//                 biClrImportant: 0,
//             },
//             bmiColors: [RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }],
//         };

//         let mut bits: *mut u8 = std::ptr::null_mut();
//         let hbm = CreateDIBSection(hdc_screen, &bmi, DIB_RGB_COLORS, &mut bits as *mut _ as _, std::ptr::null_mut(), 0);

//         let old_bm = SelectObject(hdc_mem, hbm as _);
//         DrawIconEx(hdc_mem, 0, 0, hicon, width as i32, height as i32, 0, std::ptr::null_mut(), DI_NORMAL.0);

//         let pixel_count = width * height;
//         let mut pixels = vec![0u8; pixel_count * 4];
//         GetDIBits(
//             hdc_mem,
//             hbm,
//             0,
//             height as u32,
//             pixels.as_mut_ptr() as _,
//             &mut bmi,
//             DIB_RGB_COLORS,
//         );

//         SelectObject(hdc_mem, old_bm);
//         DeleteObject(hbm as _);
//         DeleteDC(hdc_mem);
//         ReleaseDC(std::ptr::null_mut(), hdc_screen);
//         DeleteObject(icon_info.hbmColor as _);
//         DeleteObject(icon_info.hbmMask as _);
//         DestroyIcon(hicon);

//         for chunk in pixels.chunks_mut(4) {
//             chunk.swap(0, 2);
//         }

//         let mut png_bytes: Vec<u8> = Vec::new();
//         {
//             let mut encoder = png::Encoder::new(&mut png_bytes, width as u32, height as u32);
//             encoder.set_color(png::ColorType::Rgba);
//             encoder.set_depth(png::BitDepth::Eight);
//             let mut writer = encoder.write_header().ok()?;
//             writer.write_image_data(&pixels).ok()?;
//         }

//         Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&png_bytes)))

//     }

// }
