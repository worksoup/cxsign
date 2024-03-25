use activity::sign::Sign;
use std::collections::{hash_map::OccupiedError, HashMap};

use rxing::{Point, PointU};
use user::session::Session;

use utils::inquire_confirm;

pub async fn 获取所有签到(
    sessions: &HashMap<String, Session>,
) -> (
    HashMap<Sign, HashMap<&String, &Session>>,
    HashMap<Sign, HashMap<&String, &Session>>,
) {
    let mut 有效签到 = HashMap::new();
    let mut 其他签到 = HashMap::new();
    for session in sessions {
        let (available_sign_activities, other_sign_activities, _) =
            activity::Activity::get_all_activities(session.1).unwrap();
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

fn 从二维码扫描结果中获取签到所需参数(url: &str) -> Option<String> {
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
        eprintln!("{url:?}中没有找到二维码！")
    }
    r
}

pub fn 扫描路径中二维码并获取签到所需参数(pic_path: &str) -> Option<String> {
    let 扫描结果 = rxing::helpers::detect_multiple_in_file(pic_path).expect("decodes");
    从二维码扫描结果中获取签到所需参数(扫描结果[0].getText())
}

fn 获取包含所有顶点的矩形(vertex: &Vec<Point>) -> (PointU, PointU) {
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

fn 扫描图片中所有的二维码(
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
pub fn 裁剪图片(
    原图: xcap::image::RgbaImage,
    左上顶点: PointU,
    宽高: PointU,
) -> xcap::image::DynamicImage {
    xcap::image::DynamicImage::from(原图).crop(左上顶点.x, 左上顶点.y, 宽高.x, 宽高.y)
}

pub fn 截屏获取二维码签到所需参数(is_refresh: bool, precise: bool) -> Option<String> {
    let 所有屏幕 = xcap::Monitor::all().unwrap_or_else(|e| panic!("{e:?}"));
    // 在所有屏幕中寻找。
    if !precise && is_refresh {
        if !请求确认(
            "二维码图片是否就绪？",
            "本程序将在屏幕上寻找签到二维码，待二维码刷新后按下回车进行签到。",
        ) {
            return None;
        }
    }
    for 屏幕 in 所有屏幕 {
        // 先截取整个屏幕。
        let 所截图片 = 屏幕.capture_image().unwrap_or_else(|e| panic!("{e:?}"));
        println!("已截屏。");
        // 如果成功识别到二维码。
        let 扫描结果列表 = 扫描图片中所有的二维码(
            xcap::image::DynamicImage::from(所截图片),
            &mut HashMap::new(),
        );
        let 扫描结果列表 = if let Ok(扫描结果列表) = 扫描结果列表 {
            扫描结果列表
        } else {
            continue;
        };
        // 在结果中寻找。
        for 扫描结果 in &扫描结果列表 {
            let url = 扫描结果.getText();
            // 如果符合要求的二维码。
            if !url.contains(crate::protocol::QRCODE_PAT) && url.contains("&enc=") {
                continue;
            }
            println!("存在签到二维码。");
            return if precise && is_refresh && 请求确认("二维码图片是否就绪？", "本程序已在屏幕上找到签到二维码。请不要改变该二维码的位置，待二维码刷新后按下回车进行签到。") {
                // 如果是定时刷新的二维码，等待二维码刷新。
                let 二维码在屏幕上的位置 = 获取包含所有顶点的矩形(扫描结果.getPoints());
                println!("二维码位置：{:?}", 二维码在屏幕上的位置);
                let 所截图片 = 屏幕
                    .capture_image()
                    .unwrap_or_else(|e| panic!("{e:?}"));
                let 所截图片 = 裁剪图片(所截图片, 二维码在屏幕上的位置.0, 二维码在屏幕上的位置.1);
                let 扫描结果 = 扫描图片中所有的二维码(所截图片, &mut HashMap::new()).unwrap_or_else(|e| panic!("{e:?}"));
                从二维码扫描结果中获取签到所需参数(扫描结果[0].getText())
            } else {
                // 如果不是精确截取的二维码，则不需要提示。
                从二维码扫描结果中获取签到所需参数(url)
            };
        }
    }
    None
}
