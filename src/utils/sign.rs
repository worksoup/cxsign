use std::collections::{hash_map::OccupiedError, HashMap};

use rxing::{Point, PointU};

use crate::{activity::sign::SignActivity, session::SignSession, utils::inquire_confirm};

pub async fn get_signs<'a>(
    sessions: &'a HashMap<String, SignSession>,
) -> (
    HashMap<SignActivity, Vec<&'a SignSession>>,
    HashMap<SignActivity, Vec<&'a SignSession>>,
) {
    let mut asigns = HashMap::new();
    let mut osigns = HashMap::new();
    for (_, session) in sessions {
        let (available_sign_activities, other_sign_activities, _) =
            session.traverse_activities().await.unwrap();
        for sa in available_sign_activities {
            let vec = vec![session];
            if let Err(OccupiedError {
                mut entry,
                value: _,
            }) = asigns.try_insert(sa, vec)
            {
                entry.get_mut().push(session);
            }
        }
        for sa in other_sign_activities {
            let vec = vec![session];
            if let Err(OccupiedError {
                mut entry,
                value: _,
            }) = osigns.try_insert(sa, vec)
            {
                entry.get_mut().push(session);
            }
        }
    }
    (asigns, osigns)
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
    handle_qrcode_url(&results[0].getText())
}

pub fn get_refresh_qrcode_sign_params_on_screen(is_refresh: bool) -> Option<String> {
    fn find_max_rect(vertex: &Vec<Point>) -> (PointU, PointU) {
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
        let lt = Point { x: x_min, y: y_min };
        let rb = Point {
            x: x_max + 1.0,
            y: y_max + 1.0,
        };
        (PointU::from(lt), PointU::from(rb))
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
            &hints,
        )
    }
    let screens = screenshots::Screen::all().unwrap();
    // 在所有屏幕中寻找。
    for screen in screens {
        // 先截取整个屏幕。
        let image = screen.capture().unwrap();
        // 如果成功识别到二维码。
        if let Ok(results) = detect_multiple_in_image(image, &mut HashMap::new()) {
            // 在结果中寻找。
            for r in &results {
                let url = r.getText();
                // 如果符合要求的二维码。
                if url.contains(crate::utils::query::QRCODE_PAT) && url.contains("&enc=") {
                    // 如果是定时刷新的二维码。
                    if is_refresh {
                        // 获取二维码在屏幕上的位置。
                        let pos = r.getPoints();
                        let pos = find_max_rect(pos);
                        // 等待二维码刷新。
                        if inquire_confirm("二维码图片是否就绪？","本程序已在屏幕上找到签到二维码。请不要改变该二维码的位置，待二维码刷新后按下回车进行签到。") {
                            let wh = pos.1 - pos.0;
                            let image = screen
                                .capture_area(pos.0.x as i32, pos.0.y as i32, wh.x, wh.y)
                                .unwrap();
                            image.save("/home/leart/Pictures/123.png").unwrap();
                            let results = detect_multiple_in_image(image, &mut HashMap::new()).unwrap();
                            return Some(handle_qrcode_url(&results[0].getText()));
                        }
                    } else {
                        // 如果不是定时刷新的二维码，则不需要提示。
                        return Some(handle_qrcode_url(url));
                    }
                }
            }
        }
    }
    None
}
