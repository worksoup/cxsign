use std::collections::{hash_map::OccupiedError, HashMap};

use rxing::{Point, PointI, PointU};
use screenshots::display_info::DisplayInfo;

use crate::{activity::sign::Struct签到, session::SignSession, utils::inquire_confirm};

pub async fn get_signs(
    sessions: &HashMap<String, SignSession>,
) -> (
    HashMap<Struct签到, HashMap<&String, &SignSession>>,
    HashMap<Struct签到, HashMap<&String, &SignSession>>,
) {
    let mut 有效签到 = HashMap::new();
    let mut 其他签到 = HashMap::new();
    for session in sessions {
        let (available_sign_activities, other_sign_activities, _) =
            session.1.traverse_activities().await.unwrap();
        for sa in available_sign_activities {
            let mut map = HashMap::new();
            map.insert(session.0, session.1);
            if let Err(OccupiedError {
                mut entry,
                value: _,
            }) = 有效签到.try_insert(sa, map)
            {
                entry.get_mut().insert(session.0, session.1);
            }
        }
        for sa in other_sign_activities {
            let mut map = HashMap::new();
            map.insert(session.0, session.1);
            if let Err(OccupiedError {
                mut entry,
                value: _,
            }) = 其他签到.try_insert(sa, map)
            {
                entry.get_mut().insert(session.0, session.1);
            }
        }
    }
    (有效签到, 其他签到)
}

fn handle_qrcode_url(url: &str) -> String {
    let beg = url.find("&enc=").unwrap();
    let enc = &url[beg + 5..beg + 37];
    // 在二维码图片中会有一个参数 `c`, 二维码预签到时需要。
    // 但是该参数似乎暂时可以从 `signDetail` 接口获取到。所以此处先注释掉。
    // let beg = r.find("&c=").unwrap();
    // let c = &r[beg + 3..beg + 9];
    // (c.to_owned(), enc.to_owned())
    enc.to_owned()
}

pub fn handle_qrcode_pic_path(pic_path: &str) -> String {
    let results = rxing::helpers::detect_multiple_in_file(pic_path).expect("decodes");
    handle_qrcode_url(results[0].getText())
}

fn find_max_rect(vertex: &Vec<Point>, display_info: &DisplayInfo) -> (PointI, PointU) {
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
        Point { x, y } / display_info.scale_factor
    };
    let rb = {
        let x = x_max + 10.0;
        let y = y_max + 10.0;
        Point { x, y } / display_info.scale_factor
    };
    let wh = rb - lt;
    (PointI::from(lt), PointU::from(wh))
}

fn detect_multiple_in_image(
    image: image::RgbaImage,
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
            rxing::BufferedImageLuminanceSource::new(image::DynamicImage::ImageRgba8(image)),
        )),
        hints,
    )
}

pub fn 截屏获取二维码签到所需参数(is_refresh: bool, precise: bool) -> Option<String> {
    let screens = screenshots::Screen::all().unwrap_or_else(|e| panic!("{e:?}"));
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
        let display_info = screen.display_info;
        // 先截取整个屏幕。
        let image = screen.capture().unwrap_or_else(|e| panic!("{e:?}"));
        println!("已截屏。");
        // 如果成功识别到二维码。
        let results = detect_multiple_in_image(image, &mut HashMap::new());
        if results.is_err() {
            continue;
        }
        let results = unsafe { results.unwrap_unchecked() };
        // 在结果中寻找。
        for r in &results {
            let url = r.getText();
            // 如果符合要求的二维码。
            if !url.contains(crate::utils::query::QRCODE_PAT) && url.contains("&enc=") {
                continue;
            }
            println!("存在签到二维码。");
            if precise && is_refresh && inquire_confirm("二维码图片是否就绪？", "本程序已在屏幕上找到签到二维码。请不要改变该二维码的位置，待二维码刷新后按下回车进行签到。") {
                // 如果是定时刷新的二维码，等待二维码刷新。
                // 获取二维码在屏幕上的位置。
                let pos = find_max_rect(r.getPoints(), &display_info);
                println!("二维码位置：{pos:?}");
                let image = screen
                    .capture_area(pos.0.x, pos.0.y, pos.1.x, pos.1.y)
                    .unwrap_or_else(|e| panic!("{e:?}"));
                let results = detect_multiple_in_image(image, &mut HashMap::new()).unwrap_or_else(|e| panic!("{e:?}"));
                return Some(handle_qrcode_url(results[0].getText()));
            } else {
                // 如果不是定时刷新的二维码，则不需要提示。
                return Some(handle_qrcode_url(url));
            }
        }
    }
    None
}
