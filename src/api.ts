import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'

// Single IPC surface for the UI, backed by Tauri invoke/listen.
// Every value comes from the Rust backend; nothing here invents device state.

export type WriteReceipt = {
  reportId: number
  length: number
  label: string
  mode: string
  simulated: boolean
  acknowledged: boolean
  ackReport: number | null
  verified: boolean
  verification: string
  reportHex: string
}

export type ApplyResult = {
  state: object
  writeMode: string
  reports: WriteReceipt[]
  hardwareApplied: boolean
  note: string
}

export type ConnectResult = {
  ok: boolean
  mode?: string
  error?: string
  writeMode: string
  writesEnabled: boolean
  readOnly: boolean
}

export type DeviceEntry = {
  node: string
  sysfs: string
  bus: number
  vendorId: number
  productId: number
  interface: number | null
  name: string
  isConfigInterface: boolean
  featureReports: [number, number][]
  inputReports: [number, number][]
  outputReports: [number, number][]
  openableReadOnly: boolean
  openError: string | null
}

export type DescriptorReport = {
  node: string
  vendorId: number
  productId: number
  interface: number | null
  name: string
  descriptorLen: number
  descriptorHex: string
  featureReports: [number, number][]
  inputReports: [number, number][]
  outputReports: [number, number][]
  topLevelUsages: [number, number][]
}

export type DeviceStatus = {
  connected: boolean
  mode: string | null
  batteryPercent: number | null
  asleep: boolean
  dpiStage: number | null
  eventCount: number
  lastEvent: object | null
  writeMode: string
  writesEnabled: boolean
  experimentalEnabled: boolean
  note: string
}

export type PollingMeasurement = {
  ok: boolean
  node: string | null
  durationMs: number
  samples: number
  measuredHz: number | null
  nearestStandardHz: number | null
  medianIntervalUs: number | null
  jitterUs: number | null
  note: string
}

export const api = {
  // Window controls
  minimize: () => getCurrentWindow().minimize(),
  maximize: () => getCurrentWindow().toggleMaximize(),
  close:    () => getCurrentWindow().close(),

  // Device
  connect:    (mode?: string) => invoke<ConnectResult>('connect', { mode }),
  disconnect: () => invoke<void>('disconnect'),
  battery:    () => invoke<number | null>('battery'),

  // Discovery / inspection
  discoverDevices:  () => invoke<DeviceEntry[]>('discover_devices'),
  deviceStatus:     () => invoke<DeviceStatus>('device_status'),
  descriptorDump:   (node?: string) => invoke<DescriptorReport[]>('descriptor_dump', { node }),
  measurePolling:   (productId?: number, durationMs?: number) =>
    invoke<PollingMeasurement>('measure_polling_rate', { productId, durationMs }),

  // Events — each registers the listener async internally but exposes a
  // sync unlisten so callers can clean up in a React effect return.
  onBattery: (cb: (pct: number) => void): (() => void) => {
    let unlisten: UnlistenFn | undefined
    listen<number>('mouse:battery', e => cb(e.payload)).then(fn => { unlisten = fn })
    return () => { unlisten?.() }
  },

  onDpiStage: (cb: (stage: number) => void): (() => void) => {
    let unlisten: UnlistenFn | undefined
    listen<number>('mouse:dpi_stage', e => cb(e.payload)).then(fn => { unlisten = fn })
    return () => { unlisten?.() }
  },

  onDisconnected: (cb: () => void): (() => void) => {
    let unlisten: UnlistenFn | undefined
    listen('mouse:disconnected', () => cb()).then(fn => { unlisten = fn })
    return () => { unlisten?.() }
  },

  onWriteReceipt: (cb: (r: WriteReceipt) => void): (() => void) => {
    let unlisten: UnlistenFn | undefined
    listen<WriteReceipt>('mouse:write_receipt', e => cb(e.payload)).then(fn => { unlisten = fn })
    return () => { unlisten?.() }
  },

  onTraySearch: (cb: () => void): (() => void) => {
    let unlisten: UnlistenFn | undefined
    listen('tray:search', () => cb()).then(fn => { unlisten = fn })
    return () => { unlisten?.() }
  },

  // Config
  getConfig:   ()           => invoke<object>('get_config'),
  saveConfig:  (patch: object) => invoke<object>('save_config', { patch }),
  applyConfig: (patch: object) => invoke<ApplyResult>('apply_config', { patch }),
  writeSetting: (target: 'dpi' | 'lighting' | 'prefs' | 'polling' | 'buttons', patch: object) =>
    invoke<ApplyResult>('write_setting', { target, patch }),
  resetConfig: ()           => invoke<ApplyResult>('reset_config'),

  // Autostart
  autostartStatus:  () => invoke<boolean>('autostart_status'),
  autostartEnable:  () => invoke<void>('autostart_enable'),
  autostartDisable: () => invoke<void>('autostart_disable'),

  // DPI overlay
  showDpiOverlay: (text: string) => invoke<void>('show_dpi_overlay', { text }),

  // Profiles
  profilesList:   ()                          => invoke<string[]>('profiles_list'),
  profilesSave:   (name: string, cfg?: object) =>
    invoke<string[]>('profiles_save', { name, cfg }),
  profilesLoad:   (name: string)               => invoke<object | null>('profiles_load', { name }),
  profilesDelete: (name: string)               => invoke<string[]>('profiles_delete', { name }),
}
