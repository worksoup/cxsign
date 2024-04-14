fn main() {
    let os_name = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_debug = std::env::var("PROFILE").unwrap().contains("debug");
    let can_use_cam = ["android", "ios"].contains(&os_name.as_str());
    let can_use_cap = ["windows", "macos", "linux"].contains(&os_name.as_str());
    fn b2n(b: bool) -> i32 {
        if b {
            1
        } else {
            0
        }
    }
    let get_qrcode_type_count = b2n(can_use_cam) + b2n(can_use_cap);
    std::fs::write(
        "../src/lib/commands/constants.ts",
        format!(
            r#"
export const IS_DEBUG: boolean = {is_debug};
export const OS_NAME: string = "{os_name}";
export const CAN_USE_CAM: boolean = {can_use_cam};
export const CAN_USE_CAP: boolean = {can_use_cap};
export const GET_QR_CODE_TYPE_COUNT: number = {get_qrcode_type_count};
"#
        )
        .trim(),
    )
    .unwrap();
    tauri_build::build();
}