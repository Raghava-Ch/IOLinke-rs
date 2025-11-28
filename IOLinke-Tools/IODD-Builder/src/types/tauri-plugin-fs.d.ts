// Ambient module declarations for optional Tauri filesystem plugin used in persistence helpers.
declare module "@tauri-apps/plugin-fs" {
  export interface BaseDirectoryMap {
    App: unknown;
    AppConfig: unknown;
    AppData: unknown;
    AppLocalData: unknown;
    AppCache: unknown;
    Config: unknown;
    Data: unknown;
    LocalData: unknown;
    Cache: unknown;
    Log: unknown;
    Temp: unknown;
    Download: unknown;
    Desktop: unknown;
    Document: unknown;
    Picture: unknown;
    Public: unknown;
    Video: unknown;
    Audio: unknown;
    Resource: unknown;
    Executable: unknown;
    Font: unknown;
  }

  export type BaseDirectory = keyof BaseDirectoryMap;

  export const BaseDirectory: Record<BaseDirectory, BaseDirectory>;

  export function exists(path: string, options?: { dir?: BaseDirectory; baseDir?: BaseDirectory }): Promise<boolean>;
  export function readTextFile(
    path: string,
    options?: { dir?: BaseDirectory; baseDir?: BaseDirectory }
  ): Promise<string>;
  export function writeTextFile(
    path: string,
    contents: string,
    options?: { dir?: BaseDirectory; baseDir?: BaseDirectory }
  ): Promise<void>;
}
