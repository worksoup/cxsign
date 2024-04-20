use image::{ImageBuffer, Rgba};

#[tauri::command]
pub async fn scan_image(w: u32, h: u32, image_buffer: Vec<u8>) -> Result<String, String> {
    // info!("{image_buffer:?}");
    let image: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_vec(w, h, image_buffer).ok_or("无法转换为图片！".to_string())?;

    // #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    // image.save("./1.png").map_err(|err| err.to_string())?;
    let r = cxsign_internal::utils::scan_qrcode(
        image::DynamicImage::from(image),
        &mut std::collections::HashMap::new(),
    )
    .map_err(|err| err.to_string())?;
    for r in &r {
        let url = r.getText();
        // 如果符合要求的二维码。
        if !(url.contains(cxsign_internal::protocol::QRCODE_PAT) && url.contains("&enc=")) {
            log::warn!("{url:?}不是有效的签到二维码！");
            continue;
        }
        log::info!("存在签到二维码。");
        // 如果不是精确截取的二维码，则不需要提示。
        return cxsign_internal::utils::scan_result_to_enc(url)
            .ok_or("没有 `enc` 参数！".to_string());
    }
    // 这段代码事实上应该无法执行到的。
    // Err("没有二维码！".to_string())
    unreachable!("没有二维码！这段代码事实上应该无法执行得到，如果本行代码执行，很可能说明了代码发生了一些变化。")
}
