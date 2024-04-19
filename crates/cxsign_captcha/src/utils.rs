use image::{DynamicImage, Luma};
use imageproc::map::{map_colors, map_colors2};
use imageproc::point::Point;
use log::debug;
use serde::Deserialize;

use cxsign_imageproc::{image_mean, image_sum};

use crate::hash::{encode, hash, uuid};
use crate::protocol::{check_captcha, get_captcha, get_server_time, CALLBACK_NAME};

pub fn trim_response_to_json<'a, T>(text: &'a str) -> Result<T, ureq::serde_json::Error>
where
    T: ureq::serde::de::Deserialize<'a>,
{
    let s = &text[CALLBACK_NAME.len() + 1..text.len() - 1];
    ureq::serde_json::from_str(s)
}

pub fn generate_secrets(
    captcha_id: &str,
    server_time_stamp_mills: u128,
    r#type: &str,
) -> (String, String) {
    let captcha_key = encode(hash(&(server_time_stamp_mills.to_string() + &uuid())));
    let tmp_token = encode(hash(
        &(server_time_stamp_mills.to_string() + captcha_id + r#type + &captcha_key),
    ));
    let tmp_token =
        tmp_token + "%3A" + (server_time_stamp_mills + 300000_u128).to_string().as_str();
    (captcha_key, tmp_token)
}
pub fn get_image(agent: &ureq::Agent, image_url: &str) -> Result<DynamicImage, Box<ureq::Error>> {
    let mut v = Vec::new();
    agent
        .get(&image_url)
        .call()?
        .into_reader()
        .read_to_end(&mut v)
        .unwrap();
    let img = image::io::Reader::new(std::io::Cursor::new(v))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    Ok(img)
}
pub fn solve_captcha(big_image: &DynamicImage, small_image: &DynamicImage) -> u32 {
    let small_image_alpha = cxsign_imageproc::rgb_alpha_channel(small_image);
    let rects = cxsign_imageproc::find_contour_rects::<u32>(&small_image_alpha);
    let (lt, rb) = rects[0];
    let small_image = small_image.to_luma8();
    let small_image = cxsign_imageproc::cut_picture(&DynamicImage::from(small_image), lt, rb - lt);
    let small_image = small_image.to_luma8();
    let mean = image_mean(&small_image);
    let small_image = map_colors(&small_image, |p| Luma([p[0] as f32 - mean]));
    let mut max_ncc = 0.0;
    let mut max_x = 0;
    let small_w = small_image.width();
    let big_w = big_image.width();
    let big_img = cxsign_imageproc::cut_picture(
        &big_image,
        lt,
        Point {
            x: big_w - small_w,
            y: 0,
        } + (rb - lt),
    );
    let big_img = big_img.to_luma8();
    let big_img = DynamicImage::from(big_img);
    for x in 0..big_img.width() - small_image.width() {
        let window = cxsign_imageproc::cut_picture(
            &big_img,
            Point { x, y: 0 },
            Point {
                x: small_image.width(),
                y: small_image.height(),
            },
        )
        .to_luma8();
        let window_mean = image_mean(&window);
        let window = map_colors(&window, |p| Luma([p[0] as f32 - window_mean]));
        let a = map_colors2(&window, &small_image, |w, t| Luma([w[0] * t[0]]));
        let b = map_colors(&window, |w| Luma([w[0] * w[0]]));
        let ncc = image_sum(&a) / image_sum(&b);
        if ncc > max_ncc {
            max_x = x;
            max_ncc = ncc;
        }
    }
    max_x
}
pub fn auto_solve_captcha(
    agent: &ureq::Agent,
    captcha_id: &str,
    time: u128,
) -> Result<ValidateResult, Box<ureq::Error>> {
    let (key, tmp_token) = generate_secrets(captcha_id, time, "slide");
    let r = get_captcha(&agent, captcha_id, &key, &tmp_token, time + 1)?;
    #[derive(Deserialize)]
    struct Images {
        #[serde(rename = "shadeImage")]
        shade_image_url: String,
        #[serde(rename = "cutoutImage")]
        cutout_image_url: String,
    }
    #[derive(Deserialize)]
    struct Tmp {
        token: String,
        #[serde(rename = "imageVerificationVo")]
        images: Images,
    }
    let Tmp {
        token,
        images: Images {
            shade_image_url,
            cutout_image_url,
        },
    } = trim_response_to_json(&r.into_string().unwrap()).unwrap();
    debug!("滑块图片 url：{}, {}", shade_image_url, cutout_image_url);
    let agent = ureq::Agent::new();
    let small_img = get_image(&agent, &cutout_image_url)?;
    let big_img = get_image(&agent, &shade_image_url)?;
    let max_x = solve_captcha(&big_img, &small_img);
    debug!("本地滑块结果：{max_x}");
    let r = check_captcha(&agent, captcha_id, max_x, &token, time + 2)?;
    let v: ValidateResult = trim_response_to_json(&r.into_string().unwrap()).unwrap();
    debug!("滑块结果：{v:?}");
    Ok(v)
}
pub fn captcha_solver(
    agent: &ureq::Agent,
    captcha_id: &str,
) -> Result<ValidateResult, Box<ureq::Error>> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let r = get_server_time(agent, captcha_id, time)?;
    #[derive(Deserialize)]
    struct Tmp {
        t: u128,
    }
    let Tmp { t } = trim_response_to_json(r.into_string().unwrap().as_str()).unwrap();
    auto_solve_captcha(&agent, captcha_id, t)
}
#[derive(Deserialize, Debug)]
pub struct ValidateResult {
    #[serde(rename = "extraData")]
    extra_data: Option<String>,
}
impl ValidateResult {
    pub fn get_validate_info(&self) -> Option<String> {
        #[derive(Deserialize)]
        struct Tmp {
            validate: String,
        }
        self.extra_data.as_ref().map(|s| {
            debug!("{s}");
            let Tmp { validate } = ureq::serde_json::from_str(s).unwrap();
            validate
        })
    }
}
#[cfg(test)]
mod tests {
    use crate::protocol::CAPTCHA_ID;
    use crate::utils::auto_solve_captcha;

    #[test]
    fn auto_solve_captcha_test() {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        println!("{time}");
        let agent = ureq::Agent::new();
        let r = auto_solve_captcha(&agent, CAPTCHA_ID, time).unwrap();
        println!("{:?}", r.get_validate_info());
    }
}
