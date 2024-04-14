import { invoke } from "@tauri-apps/api/core";
import {
  CAN_USE_CAM,
  CAN_USE_CAP,
  GET_QR_CODE_TYPE_COUNT,
  IS_DEBUG,
  OS_NAME,
} from "./constants";
export enum Page {
  home,
  login,
  courseSigns,
  sign,
  qrCodeScanner,
  // locations,
  // locationImpoter,
}
// export type HomePageData = {
//   value: string;
// };
// // export type LoginPageData = {};
// export type CourseSignsPageData = {};
// export type SignPageData = {};
// export type LivePlayerPageData = {};
// export type GlobalStateData<T extends Page = Page.home> = T extends Page.home
//   ? HomePageData
//   : T extends Page.courseSigns
//   ? CourseSignsPageData
//   : T extends Page.sign
//   ? SignPageData
//   : T extends Page.livePlayer
//   ? LivePlayerPageData
//   : null;
// // export type qrCodeScannerPageData = {};
// export class GlobalState {
//   page: Page;
//   data: GlobalStateData<typeof this.page>;
// }
export async function scanImage(
  w: number,
  h: number,
  imageBuffer: Iterable<number>
): Promise<string> {
  return await invoke<string>("scan_image", {
    w,
    h,
    imageBuffer: Array.from(imageBuffer),
  });
}
export function isDebug(): boolean {
  return IS_DEBUG;
}
export function osName(): string {
  return OS_NAME;
}
export function canUseCam(): boolean {
  return CAN_USE_CAM;
}
export function canUseCap(): boolean {
  return CAN_USE_CAP;
}
export function getQrCodeTypeCount(): number {
  return GET_QR_CODE_TYPE_COUNT;
}
