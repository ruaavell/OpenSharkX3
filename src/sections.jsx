/* ============ redesigned sections ============ */
import React, { useState, useEffect, useRef } from 'react'
import {
  Activity, ArrowDown, ArrowRight, ArrowUp, BatteryMedium, Bell, Bluetooth,
  Check, ChevronDown, ChevronRight, CircleAlert, Copy, Cpu, Crosshair,
  Download, ExternalLink, FileJson, FlaskConical, FolderPlus, Gauge, GitBranch,
  HardDrive, Info, Keyboard, Languages, ListVideo, Lock, Monitor, MousePointer2,
  Palette, Pencil, Play, Plus, RefreshCw, RotateCcw, Save, ShieldCheck,
  Stethoscope, Terminal, Trash2, Unlock, Upload, Usb, Wifi, X, Zap,
} from 'lucide-react'
import { BUTTONS, NATIVE_ACTIONS, SHORTCUT_PRESETS, MODES, SWATCHES, DPI_DEFAULT, DPI_MAX, USB_LED_COLOR, KEY_TO_HID } from './data.jsx'
import { LANGS, ACCENTS, tModeName, tModeDesc } from './i18n.jsx'
import { MouseStage } from './mouse.jsx'
import {
  Panel, Badge, ProvenanceBadge, receiptTier, Toggle, Seg, SensorToggle,
  Tooltip, BatteryCard, PollingRateSelector, DpiStageCard, ActivityLog,
  EmptyState, ConfirmDialog, formatTime,
} from './components.jsx'
import mousePng from './assets/mouse.png'
import tuxUrl from './assets/tux.png'

const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v))
const num = (v, fallback = 0) => { const n = Number(v); return Number.isFinite(n) ? n : fallback }
const timeNow = () => new Date().toLocaleTimeString(undefined, { hour12: false })

/* ===================== DASHBOARD ===================== */
export function DashboardSection({ ctx }) {
  const { state, t, connected, connecting, reconnect, deviceStatus, battPct, battUpdatedAt, lastMeasure, measuring, measurePolling, usbCharged } = ctx
  const usb = state.conn === 'usb'
  const bt = state.conn === 'bluetooth'
  const connLabel = !connected ? t('app.disconnected') : usb ? t('con.cable') : bt ? t('con.bluetooth') : t('con.dongle')
  const stage = state.dpi[state.activeStage]
  const effectivePolling = (usb || bt) ? 125 : state.polling

  return (
    <div className="page dashboard-page">
      <div className="hero-grid">
        <section className="hero-card">
          <div className="hero-copy">
            <div className="hero-eyebrow"><span className="status-dot" />{connected ? t('dash.live') : t('dash.knownDevice')}</div>
            <h2>{t('dash.heroTitle')}</h2>
            <p>{t('dash.heroSub')}</p>
            <div className="hero-tags">
              <Badge tone={connected ? 'good' : 'neutral'} icon={connected ? Check : CircleAlert}>
                {connected ? t('app.connected') : t('app.disconnected')}
              </Badge>
              <Badge tone="info">{connLabel}</Badge>
              <Badge tone="neutral">1d57:fa60</Badge>
            </div>
            <div className="hero-actions">
              <button type="button" className="btn primary" onClick={() => reconnect()} disabled={connecting}>
                <RefreshCw size={15} />{connecting ? t('con.searching') : connected ? t('con.reconnect') : t('con.findmouse')}
              </button>
              <button type="button" className="btn ghost" onClick={() => ctx.goTo?.('diagnostics')}>
                <Stethoscope size={15} />{t('nav.diagnostics')}
              </button>
            </div>
          </div>
          <div className="hero-visual">
            <div className="hero-halo" />
            <img src={mousePng} alt="Attack Shark X3" className="hero-mouse" draggable="false" />
            <div className="hero-led" style={{ background: usb ? USB_LED_COLOR : stage?.color }} />
            <div className="hero-led-label"><span />{connected ? `${connLabel} · ${stage?.dpi?.toLocaleString()} DPI` : t('app.disconnected')}</div>
          </div>
        </section>

        <BatteryCard pct={battPct} updatedAt={battUpdatedAt} usb={usb} t={t} />
      </div>

      <div className="stat-strip">
        <div className="metric-card">
          <span className="metric-icon"><Gauge size={17} /></span>
          <div>
            <span className="metric-label">{t('dash.polling')}</span>
            <strong className="metric-value">{effectivePolling}<small>Hz</small></strong>
            <span className="metric-sub">{lastMeasure?.measuredHz != null ? `${t('dash.measured')}: ${lastMeasure.measuredHz} Hz` : t('dash.notMeasured')}</span>
          </div>
          <button type="button" className="icon-btn metric-action" onClick={measurePolling} disabled={measuring} title={t('app.measure')}>
            <Play size={14} />
          </button>
        </div>
        <div className="metric-card">
          <span className="metric-icon"><Crosshair size={17} /></span>
          <div>
            <span className="metric-label">{t('dash.activeStage')}</span>
            <strong className="metric-value">{stage?.dpi?.toLocaleString()}<small>DPI</small></strong>
            <span className="metric-sub">{t('dpi.stage')} {state.activeStage + 1} / {state.dpi.length}</span>
          </div>
        </div>
        <div className="metric-card">
          <span className="metric-icon"><Wifi size={17} /></span>
          <div>
            <span className="metric-label">{t('dash.transport')}</span>
            <strong className="metric-value metric-text">{connected ? connLabel : '—'}</strong>
            <span className="metric-sub">{deviceStatus?.eventCount != null ? `${deviceStatus.eventCount} ${t('con.events').toLowerCase()}` : t('app.unknown')}</span>
          </div>
        </div>
        <div className="metric-card">
          <span className="metric-icon"><ShieldCheck size={17} /></span>
          <div>
            <span className="metric-label">{t('dash.writeMode')}</span>
            <strong className="metric-value metric-text">{ctx.writeMode === 'hardware' ? t('app.hardware') : t('app.readonly')}</strong>
            <span className="metric-sub">{t('pr.noHardware')}</span>
          </div>
        </div>
      </div>

      <div className="dashboard-grid">
        <Panel title={t('dash.overview')} subtitle={t('dash.interfaceNote')} actions={<Badge tone="neutral">{t('dash.knownDevice')}</Badge>}>
          <div className="detail-list">
            <div><span>{t('con.vidpid')}</span><strong>1d57:fa60</strong></div>
            <div><span>{t('con.transport')}</span><strong>{connected ? connLabel : '—'}</strong></div>
            <div><span>{t('con.configInterface')}</span><strong>02 <small>· {t('dash.live')}</small></strong></div>
            <div><span>{t('con.interface')}</span><strong>01 <small>· input</small></strong></div>
            <div><span>{t('con.events')}</span><strong>{deviceStatus?.eventCount ?? '—'}</strong></div>
          </div>
          <div className="inline-note">
            <Info size={14} />{t('dash.interfaceNote')}
          </div>
        </Panel>

        <Panel title={t('dash.dpiOverview')} subtitle={`${t('dash.activeStage')}: ${t('dpi.stage')} ${state.activeStage + 1}`}>
          <div className="dpi-mini-grid">
            {state.dpi.map((d, i) => (
              <button key={i} type="button" className={'dpi-mini' + (i === state.activeStage ? ' on' : '')} style={{ '--c': usb ? 'var(--accent)' : d.color }} onClick={() => ctx.goTo?.('dpi')}>
                <span>{String(i + 1).padStart(2, '0')}</span>
                <strong>{d.dpi.toLocaleString()}</strong>
                <small>DPI</small>
              </button>
            ))}
          </div>
          <button type="button" className="btn subtle full" onClick={() => ctx.goTo?.('dpi')}>
            <Pencil size={14} />{t('dash.quickDpi')}<ArrowRight size={14} />
          </button>
        </Panel>

        <Panel title={t('dash.recent')} actions={<Badge tone="neutral">{state.log.length}</Badge>}>
          <ActivityLog entries={state.log} t={t} limit={7} />
        </Panel>
      </div>

      <div className="quick-row">
        <button type="button" className="quick-card" onClick={() => ctx.goTo?.('performance')}>
          <span className="quick-icon"><Gauge size={18} /></span>
          <span><strong>{t('dash.quickPerformance')}</strong><small>{t('perf.polling')} · {t('perf.debounce')}</small></span>
          <ArrowRight size={16} />
        </button>
        <button type="button" className="quick-card" onClick={() => ctx.goTo?.('buttons')}>
          <span className="quick-icon"><MousePointer2 size={18} /></span>
          <span><strong>{t('dash.quickButtons')}</strong><small>{t('bt.count')} · {t('bt.hint')}</small></span>
          <ArrowRight size={16} />
        </button>
        <button type="button" className="quick-card" onClick={() => ctx.goTo?.('profiles')}>
          <span className="quick-icon"><FileJson size={18} /></span>
          <span><strong>{t('nav.profiles')}</strong><small>{t('pr.savecur')}</small></span>
          <ArrowRight size={16} />
        </button>
      </div>
    </div>
  )
}

/* ===================== DPI ===================== */
export function DpiSection({ ctx }) {
  const { state, t, connected, usbCharged } = ctx
  const [confirmReset, setConfirmReset] = useState(false)
  const usb = state.conn === 'usb'
  const sel = state.activeStage
  const stage = state.dpi[sel]
  const ledColor = usb ? USB_LED_COLOR : stage.color
  const presets = [400, 800, 1600, 3200, 6400, 12000, 22000, 26000]

  const patchStage = (p) => {
    const next = state.dpi.map((d, i) => i === sel ? { ...d, ...p } : d)
    ctx.set({ dpi: next })
  }
  const setDpi = (v) => patchStage({ dpi: clamp(num(v, stage.dpi), 50, DPI_MAX) })
  const resetLocal = () => {
    ctx.set({ dpi: DPI_DEFAULT.map(d => ({ ...d })), activeStage: 0, angleSnap: false, ripple: true })
    setConfirmReset(false)
    ctx.showToast(t('dpi.localDefaults'))
  }

  return (
    <div className="page dpi-page">
      <div className="dpi-layout">
        <Panel title={t('dpi.stages')} subtitle={`${state.dpi.length} · ${t('dpi.slots')}`} className="dpi-stage-panel">
          <div className="dpi-stage-list">
            {state.dpi.map((d, i) => (
              <DpiStageCard key={i} stage={d} index={i} active={i === sel} onSelect={(n) => ctx.set({ activeStage: n })} t={t} usb={usb} />
            ))}
          </div>
          <div className="inline-note">
            <Info size={14} />{t('dpi.addNotSupported')} {t('dpi.enableNotSupported')}
          </div>
        </Panel>

        <div className="dpi-editor-col">
          <Panel
            title={`${t('dpi.adjust')} · ${t('dpi.stage')} ${sel + 1}`}
            badge={<><Badge tone="warn">{t('app.experimental')}</Badge><ProvenanceBadge tier="local" t={t} /></>}
            actions={<span className="stage-swatch" style={{ background: ledColor, boxShadow: `0 0 14px ${ledColor}55` }} />}
          >
            <div className="dpi-value-row">
              <div>
                <span className="field-label">{t('dpi.resolution')}</span>
                <div className="dpi-big-input">
                  <input
                    type="number"
                    min="50"
                    max={DPI_MAX}
                    step="50"
                    value={stage.dpi}
                    onChange={e => setDpi(e.target.value)}
                    aria-label={t('dpi.resolution')}
                  />
                  <span>DPI</span>
                </div>
              </div>
              <div className="dpi-stepper">
                <button type="button" className="step-btn" onClick={() => setDpi(stage.dpi - 50)} aria-label="-50 DPI">−</button>
                <button type="button" className="step-btn" onClick={() => setDpi(stage.dpi + 50)} aria-label="+50 DPI">+</button>
              </div>
            </div>

            <input
              type="range"
              min="50"
              max={DPI_MAX}
              step="50"
              value={stage.dpi}
              onChange={e => setDpi(e.target.value)}
              aria-label={t('dpi.resolution')}
            />
            <div className="range-ends"><span>50</span><span>{DPI_MAX.toLocaleString()}</span></div>

            <div className="field-block">
              <span className="field-label">{t('dpi.presets')}</span>
              <div className="preset-chips">
                {presets.map(p => (
                  <button type="button" key={p} className={'chip' + (stage.dpi === p ? ' on' : '')} onClick={() => setDpi(p)}>{p.toLocaleString()}</button>
                ))}
              </div>
            </div>

            <div className="field-block">
              <div className="field-label-row">
                <span className="field-label">{t('dpi.color')}</span>
                <Badge tone="warn">{t('app.experimental')}</Badge>
              </div>
              <div className={'swatches' + (usb ? ' disabled' : '')}>
                {SWATCHES.map(c => (
                  <button
                    type="button"
                    key={c}
                    className={'swatch' + (stage.color === c ? ' on' : '')}
                    style={{ background: c }}
                    disabled={usb}
                    aria-label={c}
                    onClick={() => patchStage({ color: c })}
                  />
                ))}
                <label className="swatch custom" title={t('lt.picker')}>
                  <Palette size={13} />
                  <input type="color" value={stage.color} disabled={usb} onChange={e => patchStage({ color: e.target.value })} />
                </label>
              </div>
              <p className="field-hint">{usb ? t('dpi.ledlock') : t('dpi.colorExperimental')}</p>
            </div>

            <div className="panel-cta">
              <button type="button" className="btn primary" onClick={ctx.applyDpi} disabled={!connected || !ctx.api}>
                <Zap size={15} />{t('dpi.apply')}
              </button>
              <button type="button" className="btn ghost" onClick={() => setConfirmReset(true)}>
                <RotateCcw size={14} />{t('dpi.localDefaults')}
              </button>
            </div>
            <p className="field-hint center">{connected ? t('dpi.appliedHint') : t('con.notConnectedHint')}</p>
          </Panel>

          <Panel title={t('dpi.motion')} subtitle={t('perf.experimental')}>
            <SensorToggle
              title={t('dpi.anglesnap')}
              description={t('dpi.anglesnap.desc')}
              on={state.angleSnap}
              onChange={() => ctx.set({ angleSnap: !state.angleSnap })}
              experimental
              t={t}
            />
            <SensorToggle
              title={t('dpi.ripple')}
              description={t('dpi.ripple.desc')}
              on={state.ripple}
              onChange={() => ctx.set({ ripple: !state.ripple })}
              experimental
              t={t}
            />
          </Panel>
        </div>

        <div className="dpi-preview-col">
          <div className="mouse-frame dpi-mouse-frame">
            <div className="frame-glow" style={{ background: ledColor }} />
            <MouseStage {...ctx.mouseProps} glow={ledColor} mode="staticdpi" lit />
            <span className="frame-caption">{t('dpi.stage')} {sel + 1} · {stage.dpi.toLocaleString()} DPI</span>
          </div>
          <div className="dpi-summary">
            <span>{t('dpi.active')}</span>
            <strong style={{ color: ledColor }}>{stage.dpi.toLocaleString()}</strong>
            <small>DPI · {t('dpi.stage')} {sel + 1}</small>
          </div>
        </div>
      </div>

      <ConfirmDialog
        open={confirmReset}
        title={t('dpi.localDefaults')}
        message={t('dpi.localDefaultsConfirm')}
        confirmLabel={t('app.reset')}
        cancelLabel={t('app.cancel')}
        onConfirm={resetLocal}
        onCancel={() => setConfirmReset(false)}
      />
    </div>
  )
}

/* ===================== PERFORMANCE ===================== */
export function PerformanceSection({ ctx }) {
  const { state, t, connected, lastMeasure, measuring, measurePolling } = ctx
  const bt = state.conn === 'bluetooth'
  const usb = state.conn === 'usb'
  const disabledRates = bt || usb ? [250, 500, 1000] : []
  const displayPolling = bt || usb ? 125 : state.polling
  const note = bt ? t('perf.bluetoothNote') : usb ? t('perf.usbNote') : t('perf.polling.desc')

  return (
    <div className="page performance-page">
      <div className="performance-layout">
        <div className="perf-main">
          <Panel title={t('perf.polling')} subtitle={t('perf.polling.desc')} actions={<Badge tone="info">{displayPolling} Hz</Badge>}>
            <PollingRateSelector value={displayPolling} onChange={(v) => ctx.set({ polling: v })} disabledRates={disabledRates} disabled={!connected} t={t} />
            <div className="polling-compare">
              <div>
                <span>{t('perf.selected')}</span>
                <strong>{displayPolling} <small>Hz</small></strong>
                <em>{displayPolling === 1000 ? '1.0 ms' : (1000 / displayPolling).toFixed(1) + ' ms'} interval</em>
              </div>
              <div>
                <span>{t('perf.measured')}</span>
                <strong>{lastMeasure?.measuredHz != null ? lastMeasure.measuredHz : '—'} <small>Hz</small></strong>
                <em>{lastMeasure ? `${lastMeasure.samples} ${t('diag.samples').toLowerCase()} · ${lastMeasure.medianIntervalUs ?? '—'} µs` : t('perf.noMeasurement')}</em>
              </div>
              <button type="button" className="btn" onClick={measurePolling} disabled={measuring || !ctx.api}>
                <Play size={14} />{measuring ? t('app.measuring') : t('perf.measure')}
              </button>
            </div>
            <p className="field-hint">{note} {t('perf.measureHint')}</p>
          </Panel>

          <Panel title={t('perf.sensor')} subtitle={t('perf.experimental')}>
            <SensorToggle
              title={t('dpi.anglesnap')}
              description={t('dpi.anglesnap.desc')}
              on={state.angleSnap}
              onChange={() => ctx.set({ angleSnap: !state.angleSnap })}
              experimental t={t}
            />
            <SensorToggle
              title={t('dpi.ripple')}
              description={t('dpi.ripple.desc')}
              on={state.ripple}
              onChange={() => ctx.set({ ripple: !state.ripple })}
              experimental t={t}
            />
            <SensorToggle
              title={t('perf.liftOff')}
              description={t('perf.liftOff.desc')}
              on={state.liftOff}
              onChange={() => ctx.set({ liftOff: !state.liftOff })}
              experimental
              t={t}
            />
            <SensorToggle
              title={t('perf.motionSync')}
              description={t('perf.motionSync.desc')}
              on={state.motionSync}
              onChange={() => ctx.set({ motionSync: !state.motionSync })}
              experimental
              t={t}
            />
          </Panel>

          <Panel title={t('perf.debounce')} subtitle={t('perf.debounce.desc')} badge={<Badge tone="warn">{t('app.experimental')}</Badge>}>
            <div className="slider-head">
              <span>{t('perf.debounce')}</span>
              <strong>{state.debounce} <small>ms</small></strong>
            </div>
            <input
              type="range"
              min="4"
              max="50"
              step="2"
              value={state.debounce}
              onChange={e => ctx.set({ debounce: clamp(num(e.target.value, 4), 4, 50) })}
              aria-label={t('perf.debounce')}
            />
            <div className="range-ends"><span>4 ms · {t('lt.fast')}</span><span>50 ms · {t('lt.slow')}</span></div>
          </Panel>

          <div className="apply-card">
            <div>
              <strong>{t('perf.apply')}</strong>
              <p>{t('perf.applyHint')}</p>
            </div>
            <button type="button" className="btn primary" onClick={ctx.applyPerf} disabled={!connected || !ctx.api}>
              <Zap size={15} />{t('app.apply')}
            </button>
          </div>
        </div>

        <div className="perf-side">
          <div className="mouse-frame perf-mouse-frame">
            <div className="frame-glow" style={{ background: ctx.usbColor }} />
            <MouseStage {...ctx.mouseProps} sm />
            <span className="frame-caption">{t('dash.heroTitle')}</span>
          </div>
          <div className="perf-stat-grid">
            <div className="stat-card"><span>{t('perf.selected')}</span><strong>{displayPolling}<small>Hz</small></strong></div>
            <div className="stat-card"><span>{t('perf.measured')}</span><strong>{lastMeasure?.measuredHz ?? '—'}<small>Hz</small></strong></div>
            <div className="stat-card"><span>{t('diag.jitter')}</span><strong>{lastMeasure?.jitterUs ?? '—'}<small>µs</small></strong></div>
            <div className="stat-card"><span>{t('diag.samples')}</span><strong>{lastMeasure?.samples ?? '—'}</strong></div>
          </div>
          <p className="side-note"><FlaskConical size={14} />{t('perf.experimental')}</p>
        </div>
      </div>
    </div>
  )
}

/* ===================== KEY CAPTURE ===================== */
function KeyCapture({ binding, onSet, t }) {
  const [cap, setCap] = useState(false)
  const [live, setLive] = useState(null)
  useEffect(() => {
    if (!cap) return
    const handler = (e) => {
      e.preventDefault(); e.stopPropagation()
      const mods = []
      if (e.ctrlKey) mods.push('Ctrl')
      if (e.altKey) mods.push('Alt')
      if (e.shiftKey) mods.push('Shift')
      if (e.metaKey) mods.push('Super')
      const k = e.key
      const isMod = ['Control', 'Alt', 'Shift', 'Meta', 'OS'].includes(k)
      const keyName = isMod ? '' : (k === ' ' ? 'Space' : k.length === 1 ? k.toUpperCase() : k)
      setLive({ mods, key: keyName })
      if (!isMod) {
        onSet({ mods, key: keyName })
        setCap(false)
        setTimeout(() => setLive(null), 300)
      }
    }
    window.addEventListener('keydown', handler, true)
    return () => window.removeEventListener('keydown', handler, true)
  }, [cap, onSet])
  const cur = (cap && live) || (binding?.type === 'shortcut' ? { mods: binding.mods, key: binding.key } : null)
  const txt = cur && (cur.mods.length || cur.key) ? [...cur.mods, cur.key].filter(Boolean).join(' + ') : (cap ? t('bt.capturing') : t('bt.noshortcut'))
  return (
    <button type="button" className={'keycap' + (cap ? ' on' : '')} onClick={() => setCap(c => !c)}>
      <span className="keycap-value">{txt}</span>
      <span className="keycap-label">{cap ? t('bt.cancel') : t('bt.capture')}</span>
    </button>
  )
}

/* ===================== BUTTONS ===================== */
const HOTSPOT_IDS = ['left', 'right', 'wheel', 'dpi', 'fwd', 'back']

export function ButtonsSection({ ctx }) {
  const { state, t, connected } = ctx
  const [sel, setSel] = useState('left')
  const [tab, setTab] = useState(state.bindings.left?.type || 'native')
  const bd = state.bindings[sel] || { type: 'native', action: 'Clique Esquerdo' }
  const selBtn = BUTTONS.find(b => b.id === sel) || BUTTONS[0]
  const setBind = (val) => ctx.set({ bindings: { ...state.bindings, [sel]: val } })
  const pick = (id) => {
    setSel(id)
    setTab((state.bindings[id] || { type: 'native' }).type || 'native')
  }
  const dispBind = (b) => {
    if (!b) return '—'
    if (b.type === 'shortcut') return [...(b.mods || []), b.key].filter(Boolean).join(' + ') || '—'
    return ctx.tAct(b.action)
  }
  const typeTag = (ty) => ty === 'shortcut' ? t('bt.shortcut') : t('bt.native')
  const leftBroken = state.bindings.left?.type === 'native' && state.bindings.left?.action === 'Desativar Botão'
  const nativeGroups = NATIVE_ACTIONS.map(g => ({
    ...g,
    items: sel === 'left' ? g.items.filter(a => a !== 'Desativar Botão') : g.items,
  }))

  return (
    <div className="page buttons-page">
      <div className="buttons-layout">
        <Panel title={t('bt.map')} subtitle={t('bt.hint')} className="button-list-panel">
          <div className="button-list">
            {BUTTONS.map(b => {
              const bb = state.bindings[b.id]
              return (
                <button type="button" key={b.id} className={'button-row' + (sel === b.id ? ' sel' : '')} onClick={() => pick(b.id)}>
                  <span className="button-row-num">{String(BUTTONS.indexOf(b) + 1).padStart(2, '0')}</span>
                  <span className="button-row-copy">
                    <strong>{t('btn.' + b.id)}</strong>
                    <small>{typeTag(bb?.type)}</small>
                  </span>
                  <span className="button-row-action">{dispBind(bb)}</span>
                  <ChevronRight size={14} />
                </button>
              )
            })}
          </div>
          <div className="inline-note"><Info size={14} />{t('bt.mapNote')}</div>
        </Panel>

        <div className="button-mouse-column">
          <div className="button-mouse">
            <div className="mouse-halo" />
            <img src={mousePng} alt="Attack Shark X3" draggable="false" />
            {BUTTONS.filter(b => HOTSPOT_IDS.includes(b.id)).map(b => (
              <button
                type="button"
                key={b.id}
                className={'hotspot' + (sel === b.id ? ' sel' : '')}
                style={{ left: b.x + '%', top: b.y + '%' }}
                onClick={() => pick(b.id)}
                aria-label={t('btn.' + b.id)}
                title={t('btn.' + b.id)}
              >
                <span>{t('btn.' + b.id)}</span>
              </button>
            ))}
          </div>
          <p className="mouse-caption">{t('bt.hint')}</p>
        </div>

        <div className="button-editor">
          <Panel
            title={`${t('bt.selected')} · ${t('btn.' + sel)}`}
            badge={<ProvenanceBadge tier="local" t={t} />}
          >
            <div className="current-binding">
              <span>{t('bt.current')}</span>
              <strong>{dispBind(bd)}</strong>
              <button type="button" className="btn sm ghost" onClick={() => setBind({ type: 'native', action: selBtn.def })}>
                <RotateCcw size={13} />{t('bt.default')}
              </button>
            </div>

            <Seg
              live
              options={[{ v: 'native', l: t('bt.native') }, { v: 'shortcut', l: t('bt.shortcut') }]}
              value={tab}
              onChange={setTab}
            />

            {tab === 'native' && (
              <div className="action-scroll">
                {nativeGroups.map(grp => (
                  <div className="action-group" key={grp.group}>
                    <span className="field-label">{t('bt.group.' + grp.group)}</span>
                    <div className="action-grid">
                      {grp.items.map(a => (
                        <button
                          type="button"
                          key={a}
                          className={'action-chip' + (bd.type === 'native' && bd.action === a ? ' on' : '')}
                          onClick={() => setBind({ type: 'native', action: a })}
                        >
                          {ctx.tAct(a)}
                        </button>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            )}

            {tab === 'shortcut' && (
              <div className="shortcut-panel">
                <KeyCapture binding={bd} onSet={sc => setBind({ type: 'shortcut', ...sc })} t={t} />
                <span className="field-label">{t('bt.presets')}</span>
                <div className="action-grid">
                  {SHORTCUT_PRESETS.map((p, i) => {
                    const lbl = [...p.mods, p.key].join(' + ')
                    const on = bd.type === 'shortcut' && [...(bd.mods || []), bd.key].join(' + ') === lbl
                    return <button type="button" key={i} className={'action-chip' + (on ? ' on' : '')} onClick={() => setBind({ type: 'shortcut', ...p })}>{lbl}</button>
                  })}
                </div>
              </div>
            )}

            {leftBroken && <div className="warning-note"><CircleAlert size={15} />{t('bt.leftProtection')}</div>}
            {sel === 'left' && <p className="field-hint">{t('bt.leftProtection')}</p>}
            <div className="panel-cta">
              <button type="button" className="btn primary" onClick={ctx.applyBindings} disabled={!connected || leftBroken || !ctx.api}>
                <Zap size={15} />{t('bt.apply')}
              </button>
            </div>
            <p className="field-hint center">{t('bt.localHint')}</p>
          </Panel>
        </div>
      </div>
    </div>
  )
}

/* ===================== LIGHTING ===================== */
export function LightingSection({ ctx }) {
  const { state, t, connected } = ctx
  const usb = state.conn === 'usb'
  const needsColor = ['static', 'breathing'].includes(state.mode)
  const hasSpeed = ['breathing', 'neon', 'colorbreathing', 'breathingdpi'].includes(state.mode)

  return (
    <div className="page lighting-page">
      <div className="lighting-layout">
        <Panel title={t('lt.modes')} subtitle={t('lt.modes.d')} actions={<Badge tone="warn">{t('app.experimental')}</Badge>}>
          <div className={'mode-grid' + (usb ? ' disabled' : '')}>
            {MODES.map(m => (
              <button
                type="button"
                key={m.id}
                className={'mode-card' + (state.mode === m.id ? ' on' : '')}
                disabled={usb}
                onClick={() => ctx.set({ mode: m.id })}
              >
                <span className="mode-dot" />
                <strong>{tModeName(m.id, ctx.lang)}</strong>
                <small>{tModeDesc(m.id, ctx.lang)}</small>
              </button>
            ))}
          </div>
          <div className="inline-note"><Info size={14} />{t('lt.experimental')}</div>
        </Panel>

        <div className="lighting-preview-col">
          <div className="mouse-frame lighting-mouse-frame">
            <div className="frame-glow" style={{ background: usb ? ctx.usbColor : ctx.mouseProps.glow }} />
            <MouseStage {...ctx.mouseProps} glow={usb ? ctx.usbColor : ctx.mouseProps.glow} mode={usb ? 'static' : ctx.mouseProps.mode} />
            <span className="frame-caption">{usb ? 'USB-C · STATIC' : tModeName(state.mode, ctx.lang).toUpperCase() + ' · ' + t('lt.preview')}</span>
          </div>
          {usb ? (
            <div className="warning-note"><Lock size={15} />{t('lt.usblock')}</div>
          ) : (
            <button type="button" className="btn primary wide" onClick={ctx.applyLighting} disabled={!connected || !ctx.api}>
              <Zap size={15} />{t('lt.apply')}
            </button>
          )}
        </div>

        <div className="lighting-controls">
          {needsColor && (
            <Panel title={t('lt.global')} actions={<span className="stage-swatch" style={{ background: state.color }} />}>
              <div className={'swatches' + (usb ? ' disabled' : '')}>
                {SWATCHES.map(c => (
                  <button type="button" key={c} className={'swatch' + (state.color === c ? ' on' : '')} style={{ background: c }} disabled={usb} onClick={() => ctx.set({ color: c })} aria-label={c} />
                ))}
                <label className="swatch custom">
                  <Palette size={13} />
                  <input type="color" value={state.color} disabled={usb} onChange={e => ctx.set({ color: e.target.value })} />
                </label>
              </div>
              <div className="color-value">{state.color.toUpperCase()}</div>
            </Panel>
          )}
          {hasSpeed && (
            <Panel title={t('lt.speed')}>
              <div className="slider-head"><span>{t('lt.effect')}</span><strong>{state.speed}×</strong></div>
              <input type="range" min="1" max="10" step="1" value={state.speed} onChange={e => ctx.set({ speed: num(e.target.value, 6) })} aria-label={t('lt.speed')} />
              <div className="range-ends"><span>{t('lt.slow')}</span><span>{t('lt.fast')}</span></div>
            </Panel>
          )}
          <Panel title={t('lt.battoverride')}>
            <div className="sensor-row">
              <div className="sensor-copy">
                <div className="sensor-title"><span>{t('lt.battoverride')}</span></div>
                <p>{t('lt.battoverride.d')}</p>
              </div>
              <Toggle on={state.battOverride} onClick={() => ctx.set({ battOverride: !state.battOverride })} label={t('lt.battoverride')} />
            </div>
          </Panel>
          <p className="side-note"><Info size={14} />{t('lt.note')}</p>
        </div>
      </div>
    </div>
  )
}

/* ===================== MACROS ===================== */
const MACRO_KEY = 'osx11_macro_library'

function readMacroLibrary() {
  try {
    const raw = localStorage.getItem(MACRO_KEY)
    if (!raw) return null
    const parsed = JSON.parse(raw)
    if (!Array.isArray(parsed) || !parsed.length) return null
    return parsed.map((m, i) => ({
      ...m,
      id: m.id || `macro-${i + 1}`,
      name: m.name || `Macro ${i + 1}`,
      events: Array.isArray(m.events) ? m.events : [],
    }))
  } catch (_) { return null }
}

export function MacroSection({ ctx }) {
  const { t } = ctx
  const [recording, setRecording] = useState(false)
  const recordStartRef = useRef(0)
  const lastEventRef = useRef(0)
  const [library, setLibrary] = useState(() => {
    const saved = readMacroLibrary()
    if (saved) return saved
    return [{
      id: 'macro-1', name: 'Macro 1',
      enabled: ctx.state.customMacro?.enabled || false,
      targetButton: ctx.state.customMacro?.targetButton ?? 4,
      mode: ctx.state.customMacro?.mode ?? 0,
      repeat: ctx.state.customMacro?.repeat ?? 1,
      events: ctx.state.customMacro?.events?.length ? ctx.state.customMacro.events : [{ key: 4, delay: 20, release: false }],
    }]
  })
  const [activeId, setActiveId] = useState(() => library[0]?.id)
  const [confirmDelete, setConfirmDelete] = useState(null)
  const [saving, setSaving] = useState(false)
  const active = library.find(m => m.id === activeId) || null

  useEffect(() => {
    if (!recording) return
    recordStartRef.current = Date.now()
    lastEventRef.current = recordStartRef.current
    const handler = (e) => {
      const target = e.target
      if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT')) return
      const code = KEY_TO_HID?.[e.code]
      if (code == null) return
      e.preventDefault()
      const now = Date.now()
      const delay = Math.max(0, Math.min(10000, now - lastEventRef.current))
      lastEventRef.current = now
      const release = e.type === 'keyup'
      // ensure an active macro exists
      let targetMacro = active
      if (!targetMacro) {
        const id = 'macro-' + Date.now()
        const m = { id, name: `${t('mac.new')} ${library.length + 1}`, enabled: false, targetButton: 4, mode: 0, repeat: 1, events: [] }
        setLibrary(lib => [...lib, m])
        setActiveId(id)
        targetMacro = m
      }
      setLibrary(lib => lib.map(m => {
        if (m.id !== (activeId || targetMacro?.id)) return m
        return { ...m, events: [...(m.events || []), { key: code, delay, release }] }
      }))
    }
    window.addEventListener('keydown', handler, true)
    window.addEventListener('keyup', handler, true)
    return () => {
      window.removeEventListener('keydown', handler, true)
      window.removeEventListener('keyup', handler, true)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [recording])

  useEffect(() => {
    try { localStorage.setItem(MACRO_KEY, JSON.stringify(library)) } catch (_) {}
  }, [library])

  useEffect(() => {
    if (!active) return
    ctx.set({
      customMacro: {
        enabled: !!active.enabled,
        targetButton: active.targetButton ?? 4,
        mode: active.mode ?? 0,
        repeat: active.repeat ?? 1,
        events: active.events || [],
      },
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeId, library])

  const updateActive = (p) => {
    if (!active) return
    setLibrary(lib => lib.map(m => m.id === active.id ? { ...m, ...p } : m))
  }
  const patchEvent = (i, p) => updateActive({ events: active.events.map((e, j) => j === i ? { ...e, ...p } : e) })
  const addEvent = (kind) => {
    if (!active) return
    const ev = kind === 'delay'
      ? { key: 0, delay: 50, release: true }
      : kind === 'release'
        ? { key: active.events[active.events.length - 1]?.key ?? 4, delay: 10, release: true }
        : { key: 4, delay: 20, release: false }
    updateActive({ events: [...active.events, ev] })
  }
  const moveEvent = (i, dir) => {
    const j = i + dir
    if (!active || j < 0 || j >= active.events.length) return
    const next = [...active.events]
    const [row] = next.splice(i, 1)
    next.splice(j, 0, row)
    updateActive({ events: next })
  }
  const addMacro = () => {
    const id = 'macro-' + Date.now()
    const m = {
      id, name: `${t('mac.new')} ${library.length + 1}`,
      enabled: false, targetButton: 4, mode: 0, repeat: 1,
      events: [{ key: 4, delay: 20, release: false }],
    }
    setLibrary(lib => [...lib, m])
    setActiveId(id)
  }
  const deleteMacro = (id) => {
    const next = library.filter(m => m.id !== id)
    setLibrary(next)
    if (activeId === id) setActiveId(next[0]?.id || null)
    setConfirmDelete(null)
  }
  const saveLocal = async () => {
    if (!ctx.api || !active) return
    setSaving(true)
    try {
      await ctx.api.saveConfig({ customMacro: {
        enabled: !!active.enabled,
        targetButton: active.targetButton ?? 4,
        mode: active.mode ?? 0,
        repeat: active.repeat ?? 1,
        events: active.events || [],
      } })
      ctx.showToast(t('mac.saved'))
      ctx.addLog({ t: timeNow(), tag: 'MAC', cls: 'ok', m: `${active.name} · ${t('app.localOnly')}` })
    } catch (e) {
      ctx.showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
    } finally { setSaving(false) }
  }

  const targetOptions = [
    { v: 0, l: t('btn.left') }, { v: 1, l: t('btn.right') }, { v: 2, l: t('btn.wheel') },
    { v: 3, l: t('btn.fwd') }, { v: 4, l: t('btn.back') },
  ]

  return (
    <div className="page macros-page">
      <div className="macros-layout">
        <Panel
          title={t('mac.library')}
          actions={
            <>
              <button type="button" className={'btn sm' + (recording ? ' danger-solid' : ' primary')} onClick={() => setRecording(r => !r)}>
                {recording ? <X size={14} /> : <Activity size={14} />}{recording ? t('app.cancel') : t('mac.record')}
              </button>
              <button type="button" className="btn sm" onClick={addMacro}><Plus size={14} />{t('app.add')}</button>
            </>
          }
          className="macro-list-panel"
        >
          <div className="macro-list">
            {library.map(m => (
              <button type="button" key={m.id} className={'macro-row' + (m.id === activeId ? ' on' : '')} onClick={() => setActiveId(m.id)}>
                <span className="macro-dot" />
                <span className="macro-row-copy">
                  <strong>{m.name}</strong>
                  <small>{m.events?.length || 0} {t('mac.events').toLowerCase()} · {t('app.localOnly')}</small>
                </span>
                <ChevronRight size={14} />
              </button>
            ))}
            {!library.length && <EmptyState icon={ListVideo} title={t('mac.addFirst')} />}
          </div>
          <div className="macro-list-foot">
            <button type="button" className="btn subtle full" onClick={addMacro}><FolderPlus size={14} />{t('mac.new')}</button>
          </div>
        </Panel>

        {active ? (
          <div className="macro-editor">
            <Panel
              title={t('mac.timeline')}
              badge={<Badge tone="warn" icon={Lock}>{t('app.localOnly')}</Badge>}
              actions={
                <button type="button" className="btn sm ghost danger" onClick={() => setConfirmDelete(active.id)}>
                  <Trash2 size={14} />{t('app.delete')}
                </button>
              }
            >
              <div className="macro-name-row">
                <input
                  className="text-input"
                  value={active.name}
                  onChange={e => updateActive({ name: e.target.value })}
                  aria-label={t('app.name')}
                />
                <div className="macro-enabled">
                  <span>{t('mac.enableLocal')}</span>
                  <Toggle on={!!active.enabled} onClick={() => updateActive({ enabled: !active.enabled })} label={t('mac.enableLocal')} />
                </div>
              </div>

              <div className="timeline-toolbar">
                <button type="button" className="btn sm" onClick={() => addEvent('key')}><Keyboard size={14} />{t('mac.addKey')}</button>
                <button type="button" className="btn sm" onClick={() => addEvent('delay')}><Gauge size={14} />{t('mac.addDelay')}</button>
                <button type="button" className="btn sm" onClick={() => addEvent('release')}><Unlock size={14} />{t('mac.addRelease')}</button>
              </div>

              <div className="macro-timeline">
                {(!active.events || !active.events.length) && (
                  <EmptyState icon={Keyboard} title={t('mac.noEvents')} />
                )}
                {(active.events || []).map((ev, i) => {
                  const isDelay = num(ev.key) === 0
                  return (
                    <div className="timeline-row" key={i}>
                      <span className="timeline-index">{String(i + 1).padStart(2, '0')}</span>
                      <span className={'timeline-kind' + (isDelay ? ' delay' : ev.release ? ' release' : '')}>
                        {isDelay ? <Gauge size={13} /> : ev.release ? <Unlock size={13} /> : <Keyboard size={13} />}
                        {isDelay ? t('mac.delay') : ev.release ? t('mac.release') : t('mac.addKey')}
                      </span>
                      {!isDelay && (
                        <label className="timeline-field">
                          <span>{t('mac.keyCode')}</span>
                          <input
                            type="number" min="0" max="255"
                            value={num(ev.key)}
                            onChange={e => patchEvent(i, { key: clamp(num(e.target.value), 0, 255) })}
                          />
                        </label>
                      )}
                      <label className="timeline-field">
                        <span>{t('mac.delay')}</span>
                        <input
                          type="number" min="0" max="10000" step="10"
                          value={num(ev.delay)}
                          onChange={e => patchEvent(i, { delay: clamp(num(e.target.value), 0, 10000) })}
                        />
                      </label>
                      <div className="timeline-actions">
                        <button type="button" className="icon-btn" onClick={() => moveEvent(i, -1)} disabled={i === 0} aria-label={t('mac.moveUp')}><ArrowUp size={13} /></button>
                        <button type="button" className="icon-btn" onClick={() => moveEvent(i, 1)} disabled={i === active.events.length - 1} aria-label={t('mac.moveDown')}><ArrowDown size={13} /></button>
                        <button type="button" className="icon-btn danger" onClick={() => updateActive({ events: active.events.filter((_, j) => j !== i) })} aria-label={t('app.delete')}><X size={13} /></button>
                      </div>
                    </div>
                  )
                })}
              </div>

              <div className="macro-advanced">
                <span className="field-label">{t('app.actions')}</span>
                <div className="macro-advanced-grid">
                  <label><span>{t('mac.target')}</span>
                    <select value={active.targetButton ?? 4} onChange={e => updateActive({ targetButton: num(e.target.value, 4) })}>
                      {targetOptions.map(o => <option key={o.v} value={o.v}>{o.l}</option>)}
                    </select>
                  </label>
                  <label><span>{t('mac.repeat')}</span>
                    <input type="number" min="1" max="255" value={active.repeat ?? 1} onChange={e => updateActive({ repeat: clamp(num(e.target.value, 1), 1, 255) })} />
                  </label>
                </div>
              </div>

              <div className="panel-cta">
                <button type="button" className="btn primary" onClick={saveLocal} disabled={saving || !ctx.api}>
                  <Save size={15} />{saving ? t('app.save') + '…' : t('mac.save')}
                </button>
                <button type="button" className="btn ghost" disabled title={t('mac.uploadUnavailable')}>
                  <Upload size={15} />{t('mac.uploadUnavailable')}
                </button>
              </div>
              <div className="warning-note"><Lock size={15} />{t('mac.localOnlyDesc')}</div>
            </Panel>
          </div>
        ) : (
          <Panel title={t('mac.timeline')}>
            <EmptyState icon={ListVideo} title={t('mac.selectHint')} action={<button type="button" className="btn primary" onClick={addMacro}><Plus size={14} />{t('mac.new')}</button>} />
          </Panel>
        )}

        <div className="macro-preview-col">
          <div className="mouse-frame macro-mouse-frame">
            <div className="frame-glow" style={{ background: 'var(--accent)' }} />
            <MouseStage {...ctx.mouseProps} sm />
            <span className="frame-caption">{t('mac.localOnly')}</span>
          </div>
          <div className="macro-note-card">
            <Lock size={16} />
            <div><strong>{t('mac.localOnly')}</strong><p>{t('mac.localOnlyDesc')}</p></div>
          </div>
        </div>
      </div>

      <ConfirmDialog
        open={!!confirmDelete}
        title={t('app.delete')}
        message={t('mac.deleteConfirm')}
        confirmLabel={t('app.delete')}
        cancelLabel={t('app.cancel')}
        onConfirm={() => deleteMacro(confirmDelete)}
        onCancel={() => setConfirmDelete(null)}
        danger
      />
    </div>
  )
}

/* ===================== PROFILES ===================== */
export function ProfilesSection({ ctx }) {
  const { state, t, connected } = ctx
  const [newName, setNewName] = useState('')
  const [editing, setEditing] = useState(null)
  const [renameValue, setRenameValue] = useState('')
  const [activeName, setActiveName] = useState(null)
  const [confirm, setConfirm] = useState(null)
  const [pendingImport, setPendingImport] = useState(null)
  const fileRef = useRef(null)
  const busyRef = useRef(false)

  const load = async (name) => {
    await ctx.profileLoad(name)
    setActiveName(name)
  }
  const requestSave = () => {
    const name = newName.trim()
    if (!name) return
    if (state.profiles.some(p => p.name === name)) {
      setConfirm({ type: 'overwrite', name })
    } else {
      ctx.profileSave(name).then(ok => { if (ok) setNewName('') })
    }
  }
  const doExport = async () => {
    if (!ctx.api || busyRef.current) return
    busyRef.current = true
    try {
      const profiles = {}
      for (const p of state.profiles) {
        const cfg = await ctx.api.profilesLoad(p.name)
        if (cfg) profiles[p.name] = cfg
      }
      const payload = { version: 1, app: 'OpenSharkX3', exportedAt: new Date().toISOString(), profiles }
      const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `opensharkx3-profiles-${new Date().toISOString().slice(0, 10)}.json`
      document.body.appendChild(a)
      a.click()
      a.remove()
      setTimeout(() => URL.revokeObjectURL(url), 1000)
      ctx.showToast(t('pr.exported'))
      ctx.addLog({ t: timeNow(), tag: 'PRF', cls: 'ok', m: t('pr.exported') })
    } catch (e) {
      ctx.showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
    } finally { busyRef.current = false }
  }
  const importFile = async (file) => {
    if (!file) return
    try {
      const text = await file.text()
      const parsed = JSON.parse(text)
      const profiles = parsed?.profiles && typeof parsed.profiles === 'object' ? parsed.profiles : null
      if (!profiles || !Object.keys(profiles).length) throw new Error(t('pr.invalid'))
      const names = Object.keys(profiles)
      const clash = names.some(n => state.profiles.some(p => p.name === n))
      if (clash) setPendingImport(profiles)
      else await applyImport(profiles)
    } catch (e) {
      ctx.showToast(t('pr.invalid') + ' ' + (e?.message ?? String(e)), true)
    } finally {
      if (fileRef.current) fileRef.current.value = ''
    }
  }
  const applyImport = async (profiles) => {
    if (!ctx.api) return
    try {
      for (const [name, cfg] of Object.entries(profiles)) await ctx.api.profilesSave(name, cfg)
      await ctx.refreshProfiles()
      setPendingImport(null)
      ctx.showToast(t('pr.imported'))
      ctx.addLog({ t: timeNow(), tag: 'PRF', cls: 'ok', m: t('pr.imported') })
    } catch (e) {
      ctx.showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
    }
  }
  const duplicate = (name) => {
    let copy = `${name} copy`
    let n = 2
    while (state.profiles.some(p => p.name === copy)) copy = `${name} copy ${n++}`
    ctx.profileDuplicate(name, copy)
  }
  const startRename = (name) => { setEditing(name); setRenameValue(name) }
  const finishRename = async (oldName) => {
    const next = renameValue.trim()
    if (!next || next === oldName) { setEditing(null); return }
    if (state.profiles.some(p => p.name === next)) {
      ctx.showToast(t('pr.overwriteConfirm'), true)
      return
    }
    const ok = await ctx.profileRename(oldName, next)
    if (ok) { setEditing(null); if (activeName === oldName) setActiveName(next) }
  }

  return (
    <div className="page profiles-page">
      <div className="profiles-layout">
        <Panel
          title={t('pr.profiles')}
          subtitle={`${state.profiles.length} ${t('pr.saved')}`}
          actions={
            <div className="profile-actions">
              <button type="button" className="btn sm" onClick={() => fileRef.current?.click()}><Upload size={14} />{t('app.import')}</button>
              <button type="button" className="btn sm" onClick={doExport} disabled={!state.profiles.length}><Download size={14} />{t('app.export')}</button>
              <input ref={fileRef} type="file" accept="application/json,.json" hidden onChange={e => importFile(e.target.files?.[0])} />
            </div>
          }
          className="profile-list-panel"
        >
          <div className="save-current">
            <input
              className="text-input"
              placeholder={t('pr.nameplaceh')}
              value={newName}
              onChange={e => setNewName(e.target.value)}
              onKeyDown={e => { if (e.key === 'Enter') requestSave() }}
            />
            <button type="button" className="btn primary" onClick={requestSave} disabled={!newName.trim()}>
              <Save size={15} />{t('pr.savecur')}
            </button>
          </div>

          <div className="profile-list">
            {state.profiles.map(p => (
              <div key={p.name} className={'profile-card' + (activeName === p.name ? ' on' : '')}>
                <span className="profile-card-icon"><FileJson size={17} /></span>
                {editing === p.name ? (
                  <div className="profile-rename">
                    <input
                      className="text-input"
                      autoFocus
                      value={renameValue}
                      onChange={e => setRenameValue(e.target.value)}
                      onKeyDown={e => { if (e.key === 'Enter') finishRename(p.name); if (e.key === 'Escape') setEditing(null) }}
                    />
                    <button type="button" className="icon-btn" onClick={() => finishRename(p.name)} aria-label={t('app.save')}><Check size={14} /></button>
                    <button type="button" className="icon-btn" onClick={() => setEditing(null)} aria-label={t('app.cancel')}><X size={14} /></button>
                  </div>
                ) : (
                  <>
                    <div className="profile-card-copy">
                      <strong>{p.name}</strong>
                      <small>{activeName === p.name ? t('pr.active') : t('app.localOnly')}</small>
                    </div>
                    <div className="profile-card-actions">
                      <button type="button" className="btn sm" onClick={() => load(p.name)} disabled={!ctx.api}><Play size={13} />{t('app.load')}</button>
                      <button type="button" className="icon-btn" title={t('app.rename')} aria-label={t('app.rename')} onClick={() => startRename(p.name)}><Pencil size={14} /></button>
                      <button type="button" className="icon-btn" title={t('app.duplicate')} aria-label={t('app.duplicate')} onClick={() => duplicate(p.name)}><Copy size={14} /></button>
                      <button type="button" className="icon-btn danger" title={t('app.delete')} aria-label={t('app.delete')} onClick={() => setConfirm({ type: 'delete', name: p.name })}><Trash2 size={14} /></button>
                    </div>
                  </>
                )}
              </div>
            ))}
            {!state.profiles.length && (
              <EmptyState icon={FolderPlus} title={t('pr.none')} message={t('pr.savecur')} />
            )}
          </div>
          <div className="inline-note"><Info size={14} />{t('pr.noHardware')}</div>
        </Panel>

        <div className="profiles-side">
          <Panel title={t('pr.persist')}>
            <div className="detail-list">
              <div><span>{t('pr.statefile')}</span><strong className="mono">~/.config/opensharkx3</strong></div>
              <div><span>{t('pr.ondisk')}</span><strong>{state.profiles.length}</strong></div>
              <div><span>{t('app.actions')}</span><strong>{t('app.localOnly')}</strong></div>
            </div>
            <div className="inline-note"><ShieldCheck size={14} />{t('pr.noHardware')}</div>
          </Panel>
          <div className="profile-tip-card">
            <FileJson size={17} />
            <div><strong>{t('app.export')} / {t('app.import')}</strong><p>{t('pr.exported')} {t('pr.imported')}</p></div>
          </div>
        </div>
      </div>

      <ConfirmDialog
        open={!!confirm}
        title={confirm?.type === 'delete' ? t('app.delete') : t('app.save')}
        message={confirm?.type === 'delete' ? t('pr.deleteConfirm') : t('pr.overwriteConfirm')}
        confirmLabel={confirm?.type === 'delete' ? t('app.delete') : t('app.save')}
        cancelLabel={t('app.cancel')}
        danger={confirm?.type === 'delete'}
        onConfirm={() => {
          if (confirm?.type === 'delete') {
            ctx.profileDelete(confirm.name)
            if (activeName === confirm.name) setActiveName(null)
          } else if (confirm?.type === 'overwrite') {
            ctx.profileSave(confirm.name).then(ok => { if (ok) setNewName('') })
          }
          setConfirm(null)
        }}
        onCancel={() => setConfirm(null)}
      />

      <ConfirmDialog
        open={!!pendingImport}
        title={t('app.import')}
        message={t('pr.overwriteConfirm')}
        confirmLabel={t('app.import')}
        cancelLabel={t('app.cancel')}
        onConfirm={() => applyImport(pendingImport)}
        onCancel={() => setPendingImport(null)}
      />
    </div>
  )
}

/* ===================== DIAGNOSTICS ===================== */
export function DiagnosticsSection({ ctx }) {
  const { t, deviceStatus, state } = ctx
  const [devices, setDevices] = useState([])
  const [dump, setDump] = useState([])
  const [selNode, setSelNode] = useState(null)
  const [busy, setBusy] = useState(false)
  const [err, setErr] = useState(null)
  const [measure, setMeasure] = useState(null)

  const refresh = async () => {
    if (!ctx.api) return
    setBusy(true); setErr(null)
    try {
      const list = await ctx.api.discoverDevices()
      setDevices(list)
      const nodes = await ctx.api.descriptorDump().catch(() => [])
      setDump(nodes)
      if (!selNode && nodes.length) setSelNode(nodes.find(n => n.interface === 2)?.node || nodes[0].node)
    } catch (e) { setErr(e?.message ?? String(e)) }
    finally { setBusy(false) }
  }
  useEffect(() => { refresh() /* eslint-disable-next-line react-hooks/exhaustive-deps */ }, [])

  const doMeasure = async () => {
    const res = await ctx.measurePolling()
    if (res) setMeasure(res)
  }
  const selectedDevice = devices.find(d => d.node === selNode)
  const selectedDump = dump.find(d => d.node === selNode)
  const errors = state.log.filter(l => l.cls === 'err' || l.cls === 'warn').slice(-6).reverse()

  const reportText = () => JSON.stringify({
    app: 'OpenSharkX3',
    generatedAt: new Date().toISOString(),
    deviceStatus,
    selectedNode: selNode,
    device: selectedDevice || null,
    descriptor: selectedDump || null,
    measurement: measure,
    receipts: ctx.receipts,
    errors,
  }, null, 2)

  const copyReport = async () => {
    try {
      await navigator.clipboard.writeText(reportText())
      ctx.showToast(t('app.copied'))
    } catch (_) { ctx.showToast(t('toast.copyFailed'), true) }
  }
  const exportReport = () => {
    try {
      const blob = new Blob([reportText()], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `opensharkx3-diagnostics-${new Date().toISOString().slice(0, 19).replace(/[:T]/g, '-')}.json`
      document.body.appendChild(a)
      a.click()
      a.remove()
      setTimeout(() => URL.revokeObjectURL(url), 1000)
      ctx.showToast(t('toast.exported'))
    } catch (e) {
      ctx.showToast(t('toast.error') + ': ' + (e?.message ?? String(e)), true)
    }
  }

  return (
    <div className="page diagnostics-page">
      <div className="diagnostics-grid">
        <Panel
          title={t('diag.nodes')}
          subtitle={t('diag.note')}
          actions={<button type="button" className="btn sm" onClick={refresh} disabled={busy}><RefreshCw size={14} className={busy ? 'spin' : ''} />{t('diag.rescan')}</button>}
        >
          {err && <div className="warning-note"><CircleAlert size={15} />{err}</div>}
          {!devices.length && <EmptyState icon={Usb} title={t('diag.noNodes')} />}
          <div className="node-list">
            {devices.map(d => (
              <button
                type="button"
                key={d.node}
                className={'node-row' + (d.node === selNode ? ' on' : '')}
                onClick={() => setSelNode(d.node)}
              >
                <span className="node-dot" />
                <span className="node-copy">
                  <strong>{d.node}</strong>
                  <small>if {d.interface ?? '?'} · {d.vendorId.toString(16)}:{d.productId.toString(16)}</small>
                </span>
                <span className="node-badges">
                  <Badge tone="neutral">F{d.featureReports.length}</Badge>
                  <Badge tone="neutral">I{d.inputReports.length}</Badge>
                  {!d.openableReadOnly && <Badge tone="bad">{t('app.disabled')}</Badge>}
                </span>
              </button>
            ))}
          </div>
          <div className="detail-list compact">
            <div><span>{t('diag.writeMode')}</span><strong>{deviceStatus?.writeMode ?? '—'}</strong></div>
            <div><span>{t('diag.experimental')}</span><strong>{deviceStatus?.experimentalEnabled ? t('app.enabled') : t('app.disabled')}</strong></div>
          </div>
        </Panel>

        <div className="diagnostics-main">
          <Panel
            title={t('diag.deviceInfo')}
            actions={
              <div className="diag-actions">
                <button type="button" className="btn sm" onClick={copyReport} title={t('diag.copy')}><Copy size={13} />{t('diag.copy')}</button>
                <button type="button" className="btn sm" onClick={exportReport}><Download size={13} />{t('diag.export')}</button>
              </div>
            }
          >
            {selectedDevice ? (
              <>
                <div className="detail-grid">
                  <div><span>{t('diag.deviceInfo')}</span><strong>{selectedDevice.name || 'Attack Shark X3'}</strong></div>
                  <div><span>{t('con.vidpid')}</span><strong className="mono">{selectedDevice.vendorId.toString(16).padStart(4, '0')}:{selectedDevice.productId.toString(16).padStart(4, '0')}</strong></div>
                  <div><span>{t('con.interface')}</span><strong>{selectedDevice.interface ?? '—'}</strong></div>
                  <div><span>{t('con.hidraw')}</span><strong className="mono">{selectedDevice.node}</strong></div>
                  <div><span>{t('diag.featureReports')}</span><strong>{selectedDevice.featureReports.length}</strong></div>
                  <div><span>{t('diag.inputReports')}</span><strong>{selectedDevice.inputReports.length}</strong></div>
                </div>
                <div className="report-tables">
                  <div>
                    <span className="field-label">{t('diag.featureReports')}</span>
                    <div className="report-chips">
                      {selectedDevice.featureReports.map(([id, len]) => <span key={id} className="report-chip">0x{id.toString(16).padStart(2, '0')} <small>{len}B</small></span>)}
                    </div>
                  </div>
                  <div>
                    <span className="field-label">{t('diag.inputReports')}</span>
                    <div className="report-chips">
                      {selectedDevice.inputReports.map(([id, len]) => <span key={id} className="report-chip">0x{id.toString(16).padStart(2, '0')} <small>{len}B</small></span>)}
                    </div>
                  </div>
                </div>
                {selectedDump && (
                  <details className="raw-details">
                    <summary><ChevronRight size={14} />{t('diag.raw')}<span>{selectedDump.descriptorLen} B</span></summary>
                    <pre className="hex-view mono">{selectedDump.descriptorHex.replace(/(.{64})/g, '$1\n')}</pre>
                  </details>
                )}
              </>
            ) : <EmptyState icon={Stethoscope} title={t('diag.noNodes')} />}
          </Panel>

          <div className="diagnostics-two">
            <Panel title={t('diag.measure')} actions={<button type="button" className="btn sm primary" onClick={doMeasure} disabled={!ctx.api || ctx.measuring}>{ctx.measuring ? t('app.measuring') : t('app.measure')}</button>}>
              {measure ? (
                <div className="measure-grid">
                  <div><span>{t('dash.measured')}</span><strong>{measure.measuredHz ?? '—'}<small>Hz</small></strong></div>
                  <div><span>{t('diag.nearest')}</span><strong>{measure.nearestStandardHz ?? '—'}<small>Hz</small></strong></div>
                  <div><span>{t('diag.samples')}</span><strong>{measure.samples}</strong></div>
                  <div><span>{t('diag.median')}</span><strong>{measure.medianIntervalUs ?? '—'}<small>µs</small></strong></div>
                  <div><span>{t('diag.jitter')}</span><strong>{measure.jitterUs ?? '—'}<small>µs</small></strong></div>
                </div>
              ) : <p className="empty-inline">{t('diag.noMeasurement')} {t('diag.moveHint')}</p>}
            </Panel>

            <Panel title={t('diag.errors')} actions={<button type="button" className="btn sm ghost" onClick={ctx.clearLog}><Trash2 size={13} />{t('diag.clear')}</button>}>
              {errors.length ? (
                <div className="error-list">
                  {errors.map((l, i) => <div key={i} className="error-row"><span>{l.t}</span><strong>{l.tag}</strong><p>{l.m}</p></div>)}
                </div>
              ) : <p className="empty-inline">{t('diag.noErrors')}</p>}
            </Panel>
          </div>

          <Panel title={t('diag.receipts')} actions={<Badge tone="neutral">{ctx.receipts.length}</Badge>}>
            {!ctx.receipts.length && <p className="empty-inline">{t('diag.noReceipts')}</p>}
            <div className="receipt-list">
              {ctx.receipts.slice(-8).reverse().map((r, i) => (
                <div className="receipt-row" key={i}>
                  <span className="mono">0x{r.reportId.toString(16).padStart(2, '0')}</span>
                  <span className="receipt-label">{r.label}</span>
                  <span className="receipt-len">{r.length} B</span>
                  <ProvenanceBadge tier={receiptTier(r)} t={t} />
                </div>
              ))}
            </div>
          </Panel>
        </div>
      </div>
    </div>
  )
}

/* ===================== SETTINGS ===================== */
export function SettingsSection({ ctx }) {
  const { t, lang, setLang, accent, setAccent, theme, setTheme, api } = ctx
  const [confirmReset, setConfirmReset] = useState(false)
  const [autostart, setAutostart] = useState(false)
  useEffect(() => {
    if (!api) return
    api.autostartStatus?.().then(v => setAutostart(!!v)).catch(() => {})
  }, [api])
  const toggleAutostart = async () => {
    if (!api) return
    try {
      if (autostart) { await api.autostartDisable(); setAutostart(false) }
      else { await api.autostartEnable(); setAutostart(true) }
      ctx.showToast?.(t('app.saved'))
    } catch (e) { ctx.showToast?.(t('toast.error') + ': ' + (e?.message ?? String(e)), true) }
  }
  const reset = () => {
    setLang('tr'); setAccent('cyan'); setTheme('dark')
    setConfirmReset(false)
  }

  return (
    <div className="page settings-page">
      <div className="settings-layout">
        <div className="settings-main">
          <Panel title={t('st.language')} subtitle={t('st.language.d')} actions={<Languages size={17} />}>
            <div className="language-grid">
              {LANGS.map(l => (
                <button type="button" key={l.id} className={'language-card' + (lang === l.id ? ' on' : '')} onClick={() => setLang(l.id)}>
                  <span className="language-tag">{l.tag}</span>
                  <span className="language-name">{l.label}</span>
                  {lang === l.id && <Check size={15} className="language-check" />}
                </button>
              ))}
            </div>
          </Panel>

          <Panel title={t('st.appearance')} subtitle={t('st.accent.d')} actions={<Palette size={17} />}>
            <div className="setting-row">
              <div><strong>{t('st.theme')}</strong><p>{theme === 'dark' ? t('st.dark') : t('st.light')}</p></div>
              <Seg live options={[{ v: 'dark', l: t('st.dark') }, { v: 'light', l: t('st.light') }]} value={theme} onChange={setTheme} />
            </div>
            <div className="setting-divider" />
            <div>
              <strong className="setting-label">{t('st.accent')}</strong>
              <div className="accent-grid">
                {ACCENTS.map(a => (
                  <button type="button" key={a.id} className={'accent-card' + (accent === a.id ? ' on' : '')} onClick={() => setAccent(a.id)}>
                    <span className="accent-dot" style={{ background: a.c, boxShadow: `0 0 10px ${a.c}66` }} />
                    <span>{a.name[lang] || a.name.en}</span>
                    {accent === a.id && <Check size={14} />}
                  </button>
                ))}
              </div>
            </div>
          </Panel>

          <Panel title={t('st.behavior')} subtitle={t('st.startup.d')}>
            <div className="setting-row">
              <div><strong>{t('st.startup')}</strong><p>{t('st.startup.d')}</p></div>
              <Toggle on={autostart} onClick={toggleAutostart} label={t('st.startup')} />
            </div>
            <div className="setting-divider" />
            <div className="setting-row disabled">
              <div><strong>{t('st.notifications')}</strong><p>{t('st.notifications.d')}</p></div>
              <Toggle on={false} disabled label={t('st.notifications')} />
            </div>
          </Panel>

          <Panel title={t('st.config')} subtitle={t('st.config.d')} actions={<HardDrive size={17} />}>
            <div className="path-box mono">~/.config/opensharkx3/</div>
            <div className="detail-list compact">
              <div><span>{t('pr.statefile')}</span><strong className="mono">state.json</strong></div>
              <div><span>{t('pr.profiles')}</span><strong className="mono">profiles.json</strong></div>
            </div>
          </Panel>
        </div>

        <div className="settings-side">
          <section className="about-card">
            <img src={tuxUrl} alt="Tux" />
            <h3>OpenShark<span>X3</span></h3>
            <p>{t('st.about.d')}</p>
            <div className="about-version">v{__APP_VERSION__} · Tauri 2 · React</div>
            <div className="about-meta">
              <div><span>{t('st.device')}</span><strong>Attack Shark X3 · 1d57:fa60</strong></div>
              <div><span>{t('st.license')}</span><strong>MIT</strong></div>
            </div>
            <a className="btn full" href="https://github.com/clevim/OpenSharkX11" target="_blank" rel="noreferrer">
              <GitBranch size={15} />{t('st.github')}<ExternalLink size={13} />
            </a>
            <p className="about-credits">{t('st.credits')}</p>
          </section>
          <button type="button" className="btn ghost danger full" onClick={() => setConfirmReset(true)}>
            <RotateCcw size={15} />{t('st.reset')}
          </button>
        </div>
      </div>

      <ConfirmDialog
        open={confirmReset}
        title={t('st.reset')}
        message={t('st.resetConfirm')}
        confirmLabel={t('app.reset')}
        cancelLabel={t('app.cancel')}
        onConfirm={reset}
        onCancel={() => setConfirmReset(false)}
      />
    </div>
  )
}
