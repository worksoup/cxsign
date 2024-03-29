use log::{error, info, warn};
use rxing::{Point, PointU};
use std::collections::HashMap;
use std::path::PathBuf;

pub use cxsign_dir::*;
pub use cxsign_login::{des_enc, load_json, login_enc};
use cxsign_store::{DataBase, DataBaseTableTrait};
use cxsign_types::{Location, LocationTable};
pub use cxsign_utils::*;
pub fn pic_dir_or_path_to_pic_path(pic_dir: &PathBuf) -> Result<Option<PathBuf>, std::io::Error> {
    loop {
        let yes = inquire_confirm("二维码图片是否就绪？", "本程序会读取 `--pic` 参数所指定的路径下最新修改的图片。你可以趁现在获取这张图片，然后按下回车进行签到。");
        if yes {
            break;
        }
    }
    let pic_path = {
        let pic_dir = std::fs::read_dir(pic_dir)?;
        let mut all_files_in_dir = Vec::new();
        for k in pic_dir {
            let k = k?;
            let file_type = k.file_type()?;
            if file_type.is_file() && {
                let file_name = k.file_name();
                file_name.to_str().is_some_and(|file_name| {
                    file_name
                        .split('.')
                        .last()
                        .is_some_and(|file_ext| file_ext == "png" || file_ext == "jpg")
                })
            } {
                all_files_in_dir.push(k);
            }
        }

        all_files_in_dir.sort_by(|a, b| {
            b.metadata()
                .unwrap()
                .modified()
                .unwrap()
                .cmp(&a.metadata().unwrap().modified().unwrap())
        });
        all_files_in_dir.get(0).map(|d| d.path())
    };
    Ok(pic_path)
}

fn scan_result_to_enc(url: &str) -> Option<String> {
    // 在二维码图片中会有一个参数 `c`, 二维码预签到时需要。
    // 但是该参数似乎暂时可以从 `signDetail` 接口获取到。所以此处先注释掉。
    // let beg = r.find("&c=").unwrap();
    // let c = &r[beg + 3..beg + 9];
    // (c.to_owned(), enc.to_owned())
    // 有时二维码里没有参数，原因不明。
    let r = url
        .find("&enc=")
        .map(|beg| url[beg + 5..beg + 37].to_owned());
    if r.is_none() {
        warn!("{url:?}中没有找到二维码！")
    }
    r
}

pub fn pic_path_to_qrcode_result(pic_path: &str) -> Option<String> {
    let r = rxing::helpers::detect_multiple_in_file(pic_path).expect("decodes");
    scan_result_to_enc(r.get(0)?.getText())
}

fn get_rect_contains_vertex(vertex: &Vec<Point>) -> (PointU, PointU) {
    // let scale_factor = display.scale_factor();
    // println!("屏幕缩放：{scale_factor}");
    let mut x_max = vertex[0].x;
    let mut x_min = x_max;
    let mut y_max = vertex[0].y;
    let mut y_min = y_max;
    for p in vertex {
        if p.x > x_max {
            x_max = p.x
        }
        if p.y > y_max {
            y_max = p.y
        }
        if p.x < x_min {
            x_min = p.x
        }
        if p.y < y_min {
            y_min = p.y
        }
    }
    let lt = {
        let x = x_min - 10.0;
        let y = y_min - 10.0;
        Point { x, y }
    };
    let rb = {
        let x = x_max + 10.0;
        let y = y_max + 10.0;
        Point { x, y }
    };
    let wh = rb - lt;
    (PointU::from(lt), PointU::from(wh))
}

fn scan_qrcode(
    image: xcap::image::DynamicImage,
    hints: &mut rxing::DecodingHintDictionary,
) -> rxing::common::Result<Vec<rxing::RXingResult>> {
    hints
        .entry(rxing::DecodeHintType::TRY_HARDER)
        .or_insert(rxing::DecodeHintValue::TryHarder(true));
    let reader = rxing::MultiFormatReader::default();
    let mut scanner = rxing::multi::GenericMultipleBarcodeReader::new(reader);
    rxing::multi::MultipleBarcodeReader::decode_multiple_with_hints(
        &mut scanner,
        &mut rxing::BinaryBitmap::new(rxing::common::HybridBinarizer::new(
            rxing::BufferedImageLuminanceSource::new(image),
        )),
        hints,
    )
}
pub fn cut_picture(
    picture: xcap::image::RgbaImage,
    top_left: PointU,
    wh: PointU,
) -> xcap::image::DynamicImage {
    xcap::image::DynamicImage::from(picture).crop(top_left.x, top_left.y, wh.x, wh.y)
}

pub fn capture_screen_for_enc(is_refresh: bool, precise: bool) -> Option<String> {
    let screens = xcap::Monitor::all().unwrap_or_else(|e| panic!("{e:?}"));
    // 在所有屏幕中寻找。
    if !precise && is_refresh {
        if !inquire_confirm(
            "二维码图片是否就绪？",
            "本程序将在屏幕上寻找签到二维码，待二维码刷新后按下回车进行签到。",
        ) {
            return None;
        }
    }
    for screen in screens {
        // 先截取整个屏幕。
        let pic = screen.capture_image().unwrap_or_else(|e| {
            error!("{e:?}");
            panic!("{e:?}")
        });
        info!("已截屏。");
        // 如果成功识别到二维码。
        let results = scan_qrcode(xcap::image::DynamicImage::from(pic), &mut HashMap::new());
        let results = if let Ok(results) = results {
            results
        } else {
            continue;
        };
        // 在结果中寻找。
        for r in &results {
            let url = r.getText();
            // 如果符合要求的二维码。
            if !(url.contains(crate::protocol::QRCODE_PAT) && url.contains("&enc=")) {
                warn!("{url:?}不是有效的签到二维码！");
                continue;
            }
            info!("存在签到二维码。");
            return if precise && is_refresh && inquire_confirm("二维码图片是否就绪？", "本程序已在屏幕上找到签到二维码。请不要改变该二维码的位置，待二维码刷新后按下回车进行签到。") {
                // 如果是定时刷新的二维码，等待二维码刷新。
                let 二维码在屏幕上的位置 = get_rect_contains_vertex(r.getPoints());
                info!("二维码位置：{:?}", 二维码在屏幕上的位置);
                let pic = screen
                    .capture_image()
                    .unwrap_or_else(|e| panic!("{e:?}"));
                let cut_pic = cut_picture(pic, 二维码在屏幕上的位置.0, 二维码在屏幕上的位置.1);
                let r = scan_qrcode(cut_pic, &mut HashMap::new()).unwrap_or_else(|e| panic!("{e:?}"));
                scan_result_to_enc(r[0].getText())
            } else {
                // 如果不是精确截取的二维码，则不需要提示。
                scan_result_to_enc(url)
            };
        }
    }
    None
}
pub fn location_str_to_location(
    db: &DataBase,
    location_str: &Option<String>,
) -> Result<Location, String> {
    let table = LocationTable::from_ref(db);
    if let Some(ref location_str) = location_str {
        let location_str = location_str.trim();
        if let Ok(location) = location_str.parse() {
            Ok(location)
        } else if let Some(location) = table.get_location_by_alias(location_str) {
            Ok(location)
        } else if let Ok(location_id) = location_str.parse() {
            if table.has_location(location_id) {
                let (_, location) = table.get_location(location_id);
                Ok(location)
            } else {
                Err(location_str.to_owned())
            }
        } else {
            Err(location_str.to_owned())
        }
    } else {
        warn!("位置字符串不存在！");
        Err("".to_string())
    }
}
