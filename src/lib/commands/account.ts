import { invoke } from "@tauri-apps/api/core";

export type AccountPair = {
  uname: string;
  name: string;
  avatar: string;
};

export async function getConfigDir(): Promise<string> {
  let dir = await invoke<string>("get_config_dir");
  return dir;
}
export async function hasAccounts(): Promise<boolean> {
  let t = false;
  await invoke<boolean>("has_accounts")
    .then((data) => {
      t = data;
    })
    .catch((error) => console.error(error));
  return t;
}
export async function refreshAccounts(unames_: Set<string>) {
  let unames = Array.from(unames_);
  await invoke<AccountPair[]>("refresh_accounts", {
    unames,
  }).catch((error) => {
    console.error(error);
  });
}
export async function deleteAccounts(unames_: Set<string>) {
  let unames = Array.from(unames_);
  await invoke<AccountPair[]>("delete_accounts", {
    unames,
  }).catch((error) => {
    console.error(error);
  });
}
export async function listAccounts(): Promise<AccountPair[]> {
  let accounts = [];
  await invoke<AccountPair[]>("list_accounts")
    .then((data) => {
      accounts = data;
    })
    .catch((error) => {
      console.error(error);
    });
  return accounts;
}
export async function loadAccounts(): Promise<void> {
  await invoke<AccountPair[]>("load_accounts").catch((error) => {
    console.error(error);
  });
}
export async function addAccount(
  uname: string,
  pwd: string
): Promise<{ isOk: boolean; errMsg: string }> {
  let t = { isOk: false, errMsg: "" };
  await invoke<boolean>("add_account", {
    uname,
    pwd,
  })
    .then(() => {
      t = { isOk: true, errMsg: "" };
    })
    .catch((error) => {
      console.error(error as string);
      let errMsg = error as string;
      errMsg = errMsg.slice(1, errMsg.length - 1);
      t = { errMsg, isOk: false };
    });
  return t;
}
export function isStringEmpty(value: string | null | undefined): boolean {
  return !value || value.trim().length === 0;
}
