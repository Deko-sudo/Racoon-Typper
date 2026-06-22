import { invoke } from '@tauri-apps/api/core';

export async function ping(): Promise<string> {
  return invoke<string>('ping');
}

export async function getAppInfo(): Promise<{
  version: string;
  build_profile: string;
  data_dir: string;
  config_dir: string;
  db_path: string;
  settings_path: string;
}> {
  return invoke('get_app_info');
}