/* ============ OpenSharkX3 — application shell ============ */
import React, { useState, useEffect, useMemo, useRef, useCallback } from 'react'
import mouseLineSvg from './assets/mouse-line.svg?raw'
import { DPI_DEFAULT, BUTTONS, USB_LED_COLOR, USB_CHARGED_COLOR, DPI_MAX } from './data.jsx'
import { makeT, tAct as _tAct, ACCENTS } from './i18n.jsx'
import { AppSidebar, AppHeader, StatusRail } from './components.jsx'
import {
  DashboardSection, DpiSection, PerformanceSection, ButtonsSection,
  MacroSection, ProfilesSection, DiagnosticsSection, SettingsSection
} from './sections.jsx'
import { api } from './api'

// ─── conversion helpers ──────────────────────────────────────────────────────
const hexToRgb = (hex) => ({
  r: parseInt(hex.slice(1, 3), 16),
  g: parseInt(hex.slice(3, 5), 16),
  b: parseInt(hex.slice(5, 7), 16),
})
const rgbToHex = ({ r, g, b }) => '#' + [r, g, b].map(x => (x ?? 0).toString(16).padStart(2, '0')).join('')
const ts = () => new Date().toLocaleTimeString(undefined, { hour12: false })

const MODE_TO_NUM = {
  off: 0x00, static: 0x10, breathing: 0x20, neon: 0x30,
  colorbreathing: 0x40, staticdpi: 0x50, breathingdpi: 0x60,
}
const NUM_TO_MODE = Object.fromEntries(Object.entries(MODE_TO_NUM).map(([k, v]) => [v, k]))

const ACTION_TO_TPL = {
  'Clique Esquerdo': 'global-left-click',
  'Clique Direito': 'global-right-click',
  'Clique Meio': 'global-middle',
  'Lateral Frente': 'global-forward',
  'Lateral Trás': 'global-backward',
  'Rolar Cima': 'global-scroll-up',
  'Rolar Baixo': 'global-scroll-down',
  'Duplo Clique': 'global-double-click',
  'Ciclar DPI': 'global-dpi-cycle',
  'DPI +': 'global-dpi-+',
  'DPI −': 'global-dpi--',
  'Desativar Botão': 'global-disable-button',
  'Copiar': 'shortcut-copy',
  'Colar': 'shortcut-paste',
  'Cortar': 'shortcut-cut',
  'Desfazer': 'shortcut-undo',
  'Refazer': 'shortcut-redo',
  'Selecionar Tudo': 'shortcut-select-all',
  'Trocar Janela': 'shortcut-swap-window',
  'Bloquear PC': 'shortcut-lock-pc',
  'Captura de Tela': 'shortcut-screen-capture',
  'Abrir Calculadora': 'browser-calculator',
  'Imprimir': 'shortcut-print',
  'Play / Pause': 'multimedia-play-pause',
  'Próxima Faixa': 'multimedia-next-track',
  'Faixa Anterior': 'multimedia-previous-track',
  'Parar Música': 'multimedia-stop-music',
  'Volume +': 'multimedia-volume-+',
  'Volume −': 'multimedia-volume--',
  'Mudo': 'multimedia-mute',
  'Player de Mídia': 'multimedia-media-player',
  'Home': 'browser-home',
  'Email': 'browser-email',
  'Buscar': 'browser-search',
}
const TPL_TO_ACTION = Object.fromEntries(Object.entries(ACTION_TO_TPL).map(([k, v]) => [v, k]))
const MOD_BITS = { Ctrl: 1, Shift: 2, Alt: 4, Super: 8 }
const HID = (() => {
  const m = {
    Enter: 0x28, Escape: 0x29, Backspace: 0x2a, Tab: 0x2b, Space: 0x2c, Minus: 0x2d, Equal: 0x2e,
    BracketLeft: 0x2f, BracketRight: 0x30, Backslash: 0x31, Semicolon: 0x33, Quote: 0x34, Backquote: 0x35,
    Comma: 0x36, Period: 0x37, Slash: 0x38, CapsLock: 0x39, Delete: 0x4c, Home: 0x4a, End: 0x4d,
    PageUp: 0x4b, PageDown: 0x4e, ArrowRight: 0x4f, ArrowLeft: 0x50, ArrowDown: 0x51, ArrowUp: 0x52,
  }
  for (let i = 0; i < 26; i++) m['Key' + String.fromCharCode(65 + i)] = 0x04 + i
  for (let i = 1; i <= 9; i++) m['Digit' + i] = 0x1e + i - 1
  m.Digit0 = 0x27
  for (let i = 1; i <= 12; i++) m['F' + i] = 0x3a + i - 1
  return m
})()
const KEY_TO_HID = (() => {
  const m = { ...HID }
  const byCode = Object.fromEntries(Object.entries(HID).map(([k, v]) => [k.replace(/^Key|^Digit/, ''), v]))
  return Object.assign(byCode, m)
})()

const RX_TO_MAIN = { wheel: 'middle', fwd: 'forward', back: 'backward', sup: 'scrollUp', sdn: 'scrollDown' }
const MAIN_TO_RX = Object.fromEntries(Object.entries(RX_TO_MAIN).map(([k, v]) => [v, k]))

function bindingToMain(b) {
  if (!b || b.type === 'native') {
    return { type: 'template', template: ACTION_TO_TPL[b?.action] || 'global-left-click' }
  }
  if (b.type === 'shortcut') {
    const mods = (b.mods || []).reduce((acc, m) => acc | (MOD_BITS[m] || 0), 0)
    const hid = KEY_TO_HID[b.key] || KEY_TO_HID['Key' + (b.key || 'A')] || 0
    return { type: 'keyboard', modifiers: mods, keyCode: hid }
  }
  return { type: 'template', template: 'global-left-click' }
}

function mainCfgToReact(cfg) {
  if (!cfg) return null
  const fallbackColors = DPI_DEFAULT.map(d => d.color)
  return {
    dpi: (cfg.dpi?.values || [400, 800, 1600, 3200, 5000, 22000]).map((v, i) => ({
      dpi: Math.min(v, DPI_MAX),
      color: cfg.lighting?.stageColors?.[i] ? rgbToHex(cfg.lighting.stageColors[i]) : fallbackColors[i],
      active: i === cfg.dpi?.activeStage,
    })),
    activeStage: cfg.dpi?.activeStage ?? 2,
    angleSnap: cfg.dpi?.angleSnap ?? false,
    ripple: cfg.dpi?.rippleControl ?? true,
    liftOff: cfg.dpi?.lod2mm ?? false,
    motionSync: cfg.dpi?.motionSync ?? false,
    mode: NUM_TO_MODE[cfg.lighting?.mode] || 'staticdpi',
    color: cfg.lighting?.globalColor ? rgbToHex(cfg.lighting.globalColor) : '#38d6f5',
    speed: Math.max(1, Math.min(10, Math.round((cfg.lighting?.ledSpeed || 3) * 10 / 5))),
    polling: cfg.pollingRate || 1000,
    debounce: cfg.performance?.keyResponse || 4,
    sleepTime: cfg.power?.sleepTime ?? 10,
    deepSleepTime: cfg.power?.deepSleepTime ?? 10,
    bindings: cfg.buttons
      ? Object.fromEntries(Object.entries(cfg.buttons).map(([k, b]) => {
          const rxKey = MAIN_TO_RX[k] || k
          if (b.type === 'template') return [rxKey, { type: 'native', action: TPL_TO_ACTION[b.template] || b.template }]
          if (b.type === 'keyboard') return [rxKey, { type: 'shortcut', mods: [], key: '' }]
          return [rxKey, { type: 'native', action: 'Clique Esquerdo' }]
        }))
      : Object.fromEntries(BUTTONS.map(b => [b.id, { type: 'native', action: b.def }])),
    customMacro: cfg.customMacro ? {
      enabled: !!cfg.customMacro.enabled,
      targetButton: cfg.customMacro.targetButton ?? 4,
      mode: cfg.customMacro.mode ?? 0,
      repeat: cfg.customMacro.repeat ?? 1,
      events: cfg.customMacro.events || [],
    } : { enabled: false, targetButton: 4, mode: 0, repeat: 1, events: [] },
  }
}

function reactCfgToMain(state) {
  return {
    dpi: {
      values: state.dpi.map(d => d.dpi),
      activeStage: state.activeStage,
      angleSnap: state.angleSnap,
      rippleControl: state.ripple,
      lod2mm: state.liftOff,
      motionSync: state.motionSync,
    },
    lighting: {
      mode: MODE_TO_NUM[state.mode] ?? 0x50,
      stageColors: state.dpi.map(d => hexToRgb(d.color)),
      globalColor: hexToRgb(state.color),
      ledSpeed: Math.max(1, Math.min(5, Math.round(state.speed * 5 / 10))),
    },
    // Bluetooth and USB-C are limited to 125 Hz by the protocol. Keep the
    // local preference intact but never transmit an unsupported rate.
    pollingRate: (state.conn === 'bluetooth' || state.conn === 'usb') ? Math.min(state.polling, 125) : state.polling,
    performance: { keyResponse: Math.max(4, Math.min(50, state.debounce % 2 === 0 ? state.debounce : state.debounce + 1)) },
    power: { sleepTime: state.sleepTime, deepSleepTime: state.deepSleepTime },
    buttons: Object.fromEntries(
      Object.entries(state.bindings || {}).map(([k, b]) => [RX_TO_MAIN[k] || k, bindingToMain(b)])
    ),
    customMacro: state.customMacro || { enabled: false, targetButton: 4, mode: 0, repeat: 1, events: [] },
  }
}

function loadLS(k, def) {
  try {
    const v = localStorage.getItem(k)
    return v === null ? def : JSON.parse(v)
  } catch (_) { return def }
}

export default function App() {
  const [theme, setThemeSt] = useState(() => loadLS('osx11_theme', 'dark'))
  const [lang, setLangSt] = useState(() => loadLS('osx11_lang', 'tr'))
  const [accent, setAccentSt] = useState(() => loadLS('osx11_accent', 'cyan'))
  const [section, setSection] = useState('dashboard')

  const [connected, setConnected] = useState(false)
  const [connecting, setConnecting] = useState(false)
  const [toastMsg, setToastMsg] = useState(null)
  const [writeMode, setWriteMode] = useState('read-only')
  const [receipts, setReceipts] = useState([])
  const [deviceStatus, setDeviceStatus] = useState(null)
  const [battUpdatedAt, setBattUpdatedAt] = useState(null)
  const [lastMeasure, setLastMeasure] = useState(null)
  const [measuring, setMeasuring] = useState(false)

  const [state, setState] = useState(() => ({
    conn: 'disconnected', batt: 0, battKnown: false,
    dpi: DPI_DEFAULT.map(d => ({ ...d })), activeStage: 2,
    angleSnap: false, ripple: true, liftOff: false, motionSync: false,
    mode: 'staticdpi', color: '#38d6f5', speed: 6, battOverride: true,
    polling: 1000, debounce: 4, sleepTime: 10, deepSleepTime: 10,
    bindings: Object.fromEntries(BUTTONS.map(b => [b.id, { type: 'native', action: b.def }])),
    customMacro: { enabled: false, targetButton: 4, mode: 0, repeat: 1, events: [{ key: 'KeyA', delay: 20, release: false }] },
    profiles: [],
    log: [{ t: ts(), tag: 'SYS', cls: 'sys', m: `OpenSharkX3 v${__APP_VERSION__} · Tauri 2 · Linux` }],
  }))
  const set = useCallback((patch) => setState(s => ({ ...s, ...patch })), [])

  const addLog = useCallback((entry) => {
    setState(s => ({ ...s, log: [...s.log.slice(-99), entry] }))
  }, [])

  const toastTimerRef = useRef(null)
  const showToast = useCallback((msg, err = false) => {
    if (toastTimerRef.current) clearTimeout(toastTimerRef.current)
    setToastMsg({ msg, err })
    toastTimerRef.current = setTimeout(() => setToastMsg(null), 3200)
  }, [])

  const setTheme = (v) => { setThemeSt(v); localStorage.setItem('osx11_theme', JSON.stringify(v)) }
  const setLang = (v) => {
    setLangSt(v)
    localStorage.setItem('osx11_lang', JSON.stringify(v))
    document.documentElement.lang = v
  }
  const setAccent = (v) => { setAccentSt(v); localStorage.setItem('osx11_accent', JSON.stringify(v)) }

  const t = useMemo(() => makeT(lang), [lang])

  useEffect(() => { document.documentElement.setAttribute('data-theme', theme) }, [theme])
  useEffect(() => { document.documentElement.lang = lang }, [lang])
  useEffect(() => {
    const c = (ACCENTS.find(a => a.id === accent) || ACCENTS[0]).c
    const r = document.documentElement.style
    r.setProperty('--live', c)
    r.setProperty('--live-soft', `color-mix(in oklab, ${c} 15%, transparent)`)
    r.setProperty('--live-line', `color-mix(in oklab, ${c} 50%, transparent)`)
  }, [accent])

  const tRef = useRef(t)
  tRef.current = t

  // Live backend status. Battery is only what the device reported.
  useEffect(() => {
    if (!api) return
    let alive = true
    const tick = async () => {
      try {
        const s = await api.deviceStatus()
        if (!alive) return
        setDeviceStatus(s)
        if (s?.batteryPercent != null && s.batteryPercent >= 0) {
          setState(st => st.batt === s.batteryPercent && st.battKnown
            ? st
            : { ...st, batt: s.batteryPercent, battKnown: true })
        }
      } catch (_) {}
    }
    tick()
    const id = setInterval(tick, 3000)
    return () => { alive = false; clearInterval(id) }
  }, [])

  const battPct = deviceStatus?.batteryPercent != null ? deviceStatus.batteryPercent : (state.battKnown ? state.batt : null)

  useEffect(() => {
    if (battPct != null) setBattUpdatedAt(Date.now())
  }, [battPct])

  const reconnect = useCallback(async (mode) => {
    if (!api || connecting) return
    setConnecting(true)
    addLog({ t: ts(), tag: 'USB', cls: 'info', m: t('log.searching') })
    try {
      const res = await api.connect(mode)
      if (res.writeMode) setWriteMode(res.writeMode)
      if (res.ok) {
        setConnected(true)
        const connVal = res.mode === 'wireless' ? '2.4ghz' : res.mode === 'bluetooth' ? 'bluetooth' : 'usb'
        set({ conn: connVal })
        addLog({ t: ts(), tag: 'USB', cls: 'ok', m: `${t('log.connectedVia')} ${res.mode}` })
        const cfg = await api.getConfig()
        const rx = mainCfgToReact(cfg)
        if (rx) setState(s => ({ ...s, ...rx }))
        const batt = await api.battery()
        if (batt !== null && batt >= 0) {
          set({ batt, battKnown: true })
          setBattUpdatedAt(Date.now())
        }
        const names = await api.profilesList()
        if (names) setState(s => ({ ...s, profiles: names.map(n => ({ name: n, meta: '' })) }))
      } else {
        addLog({ t: ts(), tag: 'USB', cls: 'warn', m: res.error || t('toast.notFound') })
        showToast(res.error || t('toast.notFound'), true)
      }
    } catch (e) {
      addLog({ t: ts(), tag: 'USB', cls: 'err', m: e?.message ?? String(e) })
      showToast(e?.message ?? String(e), true)
    } finally {
      setConnecting(false)
    }
  }, [connecting, addLog, showToast, t])

  const reconnectRef = useRef(null)
  reconnectRef.current = reconnect

  useEffect(() => {
    if (!api) return
    const cleanups = [
      api.onBattery((pct) => {
        set({ batt: pct, battKnown: true })
        setBattUpdatedAt(Date.now())
      }),
      api.onDpiStage((stage) => {
        setState(s => {
          const dpiVal = s.dpi?.[stage]?.dpi
          if (dpiVal && api?.showDpiOverlay) api.showDpiOverlay(`DPI ${dpiVal.toLocaleString()}`)
          return { ...s, activeStage: stage }
        })
        addLog({ t: ts(), tag: 'DPI', cls: 'info', m: `${t('log.stageActive')} ${stage + 1}` })
      }),
      api.onDisconnected(() => {
        setConnected(false)
        set({ conn: 'disconnected', batt: 0, battKnown: false })
        addLog({ t: ts(), tag: 'USB', cls: 'err', m: t('log.disconnected') })
        showToast(t('toast.disconnected'), true)
      }),
      api.onTraySearch(() => { reconnectRef.current?.() }),
      api.onWriteReceipt((r) => { setReceipts(prev => [...prev.slice(-19), r]) }),
    ]
    reconnect()
    return () => { cleanups.forEach(fn => fn?.()) }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Auto-update check on startup (AppImage installs).
  useEffect(() => {
    if (!api) return
    const t = setTimeout(async () => {
      try {
        const info = await api.checkUpdate()
        if (info?.available && info.downloadUrl) {
          showToast(`Güncelleme ${info.latestVersion} indiriliyor...`)
          await api.applyUpdate(info.downloadUrl)
        }
      } catch (e) {
        console.warn('update check failed', e)
      }
    }, 8000)
    return () => clearTimeout(t)
  }, [api])

  const usbConn = state.conn === 'usb'
  const usbCharged = usbConn && state.batt >= 95
  const usbColor = usbCharged ? USB_CHARGED_COLOR : USB_LED_COLOR
  const battColor = battPct == null ? 'var(--text-faint)' : battPct < 15 ? 'var(--danger)' : battPct < 30 ? 'var(--warn)' : 'var(--good)'

  const glow = useMemo(() => {
    if (usbConn) return usbColor
    if (state.battOverride) {
      if (state.batt < 15 && state.batt > 0) return '#f43f6b'
      if (state.batt < 30 && state.batt > 0) return '#f5a524'
    }
    if (state.mode === 'staticdpi' || state.mode === 'breathingdpi') return state.dpi[state.activeStage].color
    if (state.mode === 'neon' || state.mode === 'colorbreathing') return '#ff0000'
    return state.color
  }, [usbConn, usbColor, state.battOverride, state.batt, state.mode, state.dpi, state.activeStage, state.color])
  const lit = usbConn || state.mode !== 'off'
  const mouseProps = { markup: mouseLineSvg, glow, mode: usbConn ? 'static' : state.mode, speed: state.speed, lit }

  // Explicit write only: each target sends exactly the reports it owns.
  const applyConfig = async (targets, tag) => {
    if (!api || !connected) return
    try {
      const patch = reactCfgToMain(state)
      let res = null
      for (const target of targets) res = await api.writeSetting(target, patch)
      setReceipts(res?.reports || [])
      const rx = mainCfgToReact(res?.state)
      if (rx) setState(s => ({ ...s, ...rx }))
      const toastKey = tag === 'lighting' ? 'toast.lightingApplied'
        : tag === 'buttons' ? 'toast.buttonsApplied'
        : tag === 'dpi' ? 'toast.dpiApplied' : 'toast.perfApplied'
      showToast(t(toastKey))
      addLog({ t: ts(), tag: tag === 'lighting' ? 'RGB' : tag === 'buttons' ? 'BTN' : tag === 'dpi' ? 'DPI' : 'PERF', cls: 'ok', m: t(toastKey) })
    } catch (e) {
      showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
      addLog({ t: ts(), tag: 'CFG', cls: 'err', m: e?.message ?? String(e) })
    }
  }

  const applyDpi = () => applyConfig(['dpi'], 'dpi')
  const applyBindings = () => applyConfig(['buttons'], 'buttons')
  const applyPerf = () => applyConfig(['dpi', 'prefs', 'polling'], 'perf')
  const applyLighting = () => applyConfig(['prefs'], 'lighting')

  const measurePolling = useCallback(async () => {
    if (!api) return null
    setMeasuring(true)
    try {
      const res = await api.measurePolling(undefined, 1500)
      setLastMeasure(res)
      if (res?.ok) {
        showToast(t('toast.measureOk'))
        addLog({ t: ts(), tag: 'POLL', cls: 'ok', m: `${res.measuredHz} Hz · ${res.samples} ${t('diag.samples').toLowerCase()}` })
      } else {
        showToast(t('toast.measureFail'), true)
        addLog({ t: ts(), tag: 'POLL', cls: 'warn', m: res?.note || t('toast.measureFail') })
      }
      return res
    } catch (e) {
      showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
      addLog({ t: ts(), tag: 'POLL', cls: 'err', m: e?.message ?? String(e) })
      return null
    } finally {
      setMeasuring(false)
    }
  }, [addLog, showToast, t])

  const refreshProfiles = async () => {
    if (!api) return []
    const names = await api.profilesList()
    const list = (names || []).map(n => ({ name: n, meta: '' }))
    setState(s => ({ ...s, profiles: list }))
    return list
  }

  const profileSave = async (name) => {
    if (!api || !name.trim()) return
    try {
      await api.profilesSave(name.trim(), reactCfgToMain(state))
      await refreshProfiles()
      showToast(t('pr.savedToast'))
      addLog({ t: ts(), tag: 'PRF', cls: 'ok', m: `${t('log.profileSaved')} ${name.trim()}` })
      return true
    } catch (e) {
      showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
      return false
    }
  }

  const profileLoad = async (name) => {
    if (!api) return
    try {
      const cfg = await api.profilesLoad(name)
      if (!cfg) { showToast(t('pr.invalid'), true); return }
      const rx = mainCfgToReact(cfg)
      if (rx) setState(s => ({ ...s, ...rx }))
      showToast(t('pr.loadedToast'))
      addLog({ t: ts(), tag: 'PRF', cls: 'ok', m: `${t('log.profileLoaded')} ${name}` })
    } catch (e) {
      showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
    }
  }

  const profileDelete = async (name) => {
    if (!api) return
    try {
      await api.profilesDelete(name)
      await refreshProfiles()
      showToast(t('pr.deletedToast'))
      addLog({ t: ts(), tag: 'PRF', cls: 'warn', m: `${t('log.profileDeleted')} ${name}` })
      return true
    } catch (e) {
      showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
      return false
    }
  }

  const profileRename = async (oldName, newName) => {
    if (!api || !newName.trim() || newName.trim() === oldName) return false
    try {
      const cfg = await api.profilesLoad(oldName)
      if (!cfg) throw new Error(t('pr.invalid'))
      await api.profilesSave(newName.trim(), cfg)
      await api.profilesDelete(oldName)
      await refreshProfiles()
      showToast(t('pr.renamedToast'))
      addLog({ t: ts(), tag: 'PRF', cls: 'ok', m: `${t('app.rename')}: ${oldName} → ${newName.trim()}` })
      return true
    } catch (e) {
      showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
      return false
    }
  }

  const profileDuplicate = async (name, copyName) => {
    if (!api || !copyName.trim()) return false
    try {
      const cfg = await api.profilesLoad(name)
      if (!cfg) throw new Error(t('pr.invalid'))
      await api.profilesSave(copyName.trim(), cfg)
      await refreshProfiles()
      showToast(t('pr.duplicatedToast'))
      addLog({ t: ts(), tag: 'PRF', cls: 'ok', m: `${t('app.duplicate')}: ${name} → ${copyName.trim()}` })
      return true
    } catch (e) {
      showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
      return false
    }
  }

  const ctx = {
    state, set, mouseProps, t, lang, tAct: (s) => _tAct(s, lang), goTo: setSection,
    theme, setTheme, setLang, accent, setAccent, api,
    connected, connecting, reconnect, applyDpi, applyBindings, applyPerf, applyLighting,
    profileSave, profileLoad, profileDelete, profileRename, profileDuplicate, refreshProfiles,
    writeMode: deviceStatus?.writeMode || writeMode,
    receipts, deviceStatus, addLog, showToast, battPct, battColor, battUpdatedAt,
    lastMeasure, measuring, measurePolling, usbCharged, usbColor,
    clearLog: () => set({ log: [] }),
  }

  const Section = {
    dashboard: DashboardSection,
    dpi: DpiSection,
    performance: PerformanceSection,
    buttons: ButtonsSection,
    macros: MacroSection,
    profiles: ProfilesSection,
    diagnostics: DiagnosticsSection,
    settings: SettingsSection,
  }[section] || DashboardSection

  const connLabel = state.conn === 'usb' ? 'USB-C' : state.conn === '2.4ghz' ? '2.4 GHz' : state.conn === 'bluetooth' ? 'Bluetooth' : '—'

  const win = (action) => {
    if (!api) return
    if (action === 'min') api.minimize()
    else if (action === 'max') api.maximize()
    else if (action === 'close') api.close()
  }

  return (
    <div className="app">
      {toastMsg && (
        <div className={'toast' + (toastMsg.err ? ' err' : '')} role="status">
          {toastMsg.msg}
        </div>
      )}

      <AppSidebar
        t={t}
        section={section}
        setSection={setSection}
        connected={connected}
        connLabel={connLabel}
        writeMode={ctx.writeMode}
        battPct={battPct}
      />

      <div className="workspace">
        <AppHeader
          t={t}
          section={section}
          connected={connected}
          connLabel={connLabel}
          theme={theme}
          onToggleTheme={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
          onWin={win}
        />
        <main className="content">
          <Section ctx={ctx} />
        </main>
      </div>

      <StatusRail
        t={t}
        connected={connected}
        connLabel={connLabel}
        state={state}
        deviceStatus={deviceStatus}
        battPct={battPct}
        battUpdatedAt={battUpdatedAt}
        lastMeasure={lastMeasure}
        measuring={measuring}
        measurePolling={measurePolling}
        receipts={receipts}
      />
    </div>
  )
}
