import { invoke } from '@tauri-apps/api/core';

/** Thin transport wrapper: financial rules remain in the Rust backend. */
export function run<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}
