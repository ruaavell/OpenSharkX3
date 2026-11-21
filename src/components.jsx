/* ============ reusable premium UI components ============ */
import React, { useEffect } from 'react'
import {
  LayoutDashboard, Crosshair, Gauge, MousePointer2, Lightbulb, ListVideo,
  FolderOpen, Stethoscope, Settings, Sun, Moon, Minus, Square, X,
  BatteryMedium, BatteryCharging, Activity, Radio, Usb, Bluetooth,
  ChevronRight, CircleAlert, Info, Wifi, PlugZap,
} from 'lucide-react'
import tuxUrl from './assets/tux.png'

export const NAV_ICONS = {
  dashboard: LayoutDashboard,
  dpi: Crosshair,
  performance: Gauge,
  buttons: MousePointer2,
  macros: ListVideo,
  profiles: FolderOpen,
  diagnostics: Stethoscope,
  settings: Settings,
}

export function Panel({ title, subtitle, badge, actions, children, className = '', flush = false }) {
  return (
    <section className={'panel ' + (flush ? 'flush ' : '') + className}>
      {(title || actions || badge) && (
        <header className="panel-head">
          <div className="panel-title-wrap">
            <h3>{title}</h3>
            {subtitle && <p>{subtitle}</p>}
          </div>
          <div className="panel-actions">
            {badge}
            {actions}
          </div>
        </header>
      )}
      <div className="panel-body">{children}</div>
    </section>
  )
}

export function Badge({ children, tone = 'neutral', icon: Icon }) {
  return (
    <span className={'badge badge-' + tone}>
      {Icon && <Icon size={12} strokeWidth={2} />}
      {children}
    </span>
  )
}

const PROV_KEYS = {
  local: 'prov.local', transmitted: 'prov.transmitted',
  acknowledged: 'prov.acknowledged', ack: 'prov.acknowledged',
  verified: 'prov.verified',
}

export function ProvenanceBadge({ tier = 'local', t }) {
  const key = PROV_KEYS[tier] || 'prov.local'
  return <span className={'prov prov-' + (tier === 'ack' ? 'acknowledged' : tier)}>{t ? t(key) : key}</span>
}

export function receiptTier(r) {
  if (!r) return 'local'
  if (r.verified) return 'verified'
  if (r.acknowledged) return 'acknowledged'
  if (!r.simulated) return 'transmitted'
  return 'local'
}

export function Toggle({ on, onClick, disabled, label }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={!!on}
      aria-label={label}
      disabled={disabled}
      className={'toggle' + (on ? ' on' : '') + (disabled ? ' disabled' : '')}
      onClick={() => !disabled && onClick?.()}
    >
      <i />
    </button>
  )
}

export function Seg({ options, value, onChange, live, size = 'md' }) {
  return (
    <div className={'seg seg-' + size + (live ? ' live' : '')} role="group">
      {options.map(o => {
        const v = o.v ?? o
        const disabled = !!o.disabled
        return (
          <button
            type="button"
            key={String(v)}
            className={value === v ? 'on' : ''}
            disabled={disabled}
            title={o.title}
            onClick={() => !disabled && onChange(v)}
          >
            {o.l ?? o}
          </button>
        )
      })}
    </div>
  )
}

export function SensorToggle({ title, description, on, onChange, disabled, experimental, unsupported, t }) {
  return (
    <div className={'sensor-row' + (disabled ? ' is-disabled' : '')}>
      <div className="sensor-copy">
        <div className="sensor-title">
          <span>{title}</span>
          {experimental && <Badge tone="warn">{t('app.experimental')}</Badge>}
          {unsupported && <Badge tone="neutral">{t('app.notAvailable')}</Badge>}
        </div>
        <p>{description}</p>
      </div>
      <Toggle on={on} onClick={onChange} disabled={disabled} label={title} />
    </div>
  )
}

export function Tooltip({ text, children }) {
  return (
    <span className="tooltip" data-tip={text} tabIndex={0}>
      {children ?? <Info size={14} />}
    </span>
  )
}

export function ConnectionBadge({ connected, connLabel, t, compact = false }) {
  const Icon = !connected ? Radio : connLabel === 'Bluetooth' ? Bluetooth : connLabel === 'USB-C' ? Usb : Wifi
  return (
    <span className={'connection-badge' + (connected ? ' on' : '') + (compact ? ' compact' : '')}>
      <span className="conn-dot" />
      <Icon size={14} />
      <span>{connected ? connLabel : t('app.disconnected')}</span>
    </span>
  )
}

export function BatteryCard({ pct, updatedAt, usb, t, compact = false }) {
  const known = pct != null
  const color = !known ? 'var(--text-faint)' : pct < 15 ? 'var(--danger)' : pct < 30 ? 'var(--warn)' : 'var(--good)'
  const Icon = usb ? BatteryCharging : BatteryMedium
  return (
    <section className={'battery-card' + (compact ? ' compact' : '')}>
      <div className="battery-head">
        <div className="battery-label">
          <span className="icon-tile"><Icon size={16} /></span>
          <span>{t('dash.battery')}</span>
        </div>
        <Badge tone={usb ? 'info' : 'neutral'}>{usb ? 'USB-C' : '2.4 GHz'}</Badge>
      </div>
      <div className="battery-reading" style={{ color }}>
        {known ? pct : '—'}<span className="unit">%</span>
      </div>
      <div className="battery-bar" aria-hidden="true">
        <i style={{ width: (known ? pct : 0) + '%', background: color }} />
      </div>
      <div className="battery-foot">
        {!known && <span>{t('dash.batteryUnavailable')}</span>}
        {known && usb && <span>{t('dash.chargingUnknown')}</span>}
        {known && !usb && <span>{t('app.lastUpdate')}: {formatTime(updatedAt)}</span>}
      </div>
      {!known && <p className="battery-hint">{t('dash.batteryUnavailableHint')}</p>}
    </section>
  )
}

export function PollingRateSelector({ value, onChange, disabled, disabledRates = [], t }) {
  const rates = [125, 250, 500, 1000]
  return (
    <div className="polling-selector" role="group" aria-label={t('perf.polling')}>
      {rates.map(hz => {
        const isDisabled = disabled || disabledRates.includes(hz)
        return (
        <button
          type="button"
          key={hz}
          className={'polling-option' + (value === hz ? ' on' : '')}
          disabled={isDisabled}
          onClick={() => !isDisabled && onChange(hz)}
        >
          <strong>{hz}</strong>
          <span>Hz</span>
        </button>
        )
      })}
    </div>
  )
}

export function DpiStageCard({ stage, index, active, onSelect, t, usb }) {
  const color = usb ? 'var(--accent)' : stage.color
  return (
    <button
      type="button"
      className={'dpi-stage-card' + (active ? ' on' : '')}
      style={{ '--stage': color }}
      onClick={() => onSelect(index)}
    >
      <span className="dpi-stage-index">{String(index + 1).padStart(2, '0')}</span>
      <span className="dpi-stage-main">
        <span className="dpi-stage-value">{stage.dpi.toLocaleString()}<small>DPI</small></span>
        <span className="dpi-stage-label">{t('dpi.stage')} {index + 1}</span>
      </span>
      <span className="dpi-stage-dot" />
      {active && <span className="dpi-stage-active">{t('dpi.active')}</span>}
    </button>
  )
}

export function ActivityLog({ entries = [], t, limit = 8 }) {
  const rows = entries.slice(-limit).reverse()
  if (!rows.length) return <p className="empty-inline">{t('dash.noActivity')}</p>
  return (
    <div className="activity-log">
      {rows.map((l, i) => (
        <div className="activity-row" key={i}>
          <span className="activity-time">{l.t}</span>
          <span className={'activity-tag ' + (l.cls || 'info')}>{l.tag}</span>
          <span className="activity-msg">{l.m}</span>
        </div>
      ))}
    </div>
  )
}

export function EmptyState({ icon: Icon = Info, title, message, action }) {
  return (
    <div className="empty-state">
      <span className="empty-icon"><Icon size={22} /></span>
      <strong>{title}</strong>
      {message && <p>{message}</p>}
      {action}
    </div>
  )
}

export function ConfirmDialog({ open, title, message, confirmLabel, cancelLabel, onConfirm, onCancel, danger }) {
  useEffect(() => {
    if (!open) return
    const onKey = e => { if (e.key === 'Escape') onCancel?.() }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [open, onCancel])
  if (!open) return null
  return (
    <div className="dialog-backdrop" role="presentation" onMouseDown={e => { if (e.target === e.currentTarget) onCancel?.() }}>
      <div className="dialog" role="dialog" aria-modal="true" aria-labelledby="dialog-title">
        <span className={'dialog-icon' + (danger ? ' danger' : '')}><CircleAlert size={20} /></span>
        <h3 id="dialog-title">{title}</h3>
        <p>{message}</p>
        <div className="dialog-actions">
          <button type="button" className="btn ghost" onClick={onCancel}>{cancelLabel}</button>
          <button type="button" className={'btn ' + (danger ? 'danger-solid' : 'primary')} onClick={onConfirm}>{confirmLabel}</button>
        </div>
      </div>
    </div>
  )
}

export function AppSidebar({ t, section, setSection, connected, connLabel, writeMode, battPct }) {
  return (
    <aside className="sidebar">
      <div className="sidebar-brand" data-tauri-drag-region>
        <span className="brand-mark"><img src={tuxUrl} alt="Tux" draggable="false" /></span>
        <span className="brand-text">
          <strong>OpenShark<span>X3</span></strong>
          <small>ATTACK SHARK · LINUX</small>
        </span>
      </div>

      <div className="sidebar-label">{t('nav.control')}</div>
      <nav className="sidebar-nav" aria-label={t('nav.control')}>
        {Object.keys(NAV_ICONS).map(id => {
          const Icon = NAV_ICONS[id]
          return (
            <button
              type="button"
              key={id}
              className={'nav-item' + (section === id ? ' active' : '')}
              onClick={() => setSection(id)}
              aria-current={section === id ? 'page' : undefined}
            >
              <Icon size={18} strokeWidth={1.8} />
              <span>{t('nav.' + id)}</span>
              {section === id && <ChevronRight size={14} className="nav-arrow" />}
            </button>
          )
        })}
      </nav>

      <div className="sidebar-spacer" />

      <div className={'sidebar-connection' + (connected ? ' on' : '')}>
        <span className="conn-dot" />
        <div className="sidebar-connection-copy">
          <strong>{connected ? connLabel : t('app.disconnected')}</strong>
          <small>{writeMode === 'hardware' ? t('app.hardware') : t('app.readonly')}</small>
        </div>
        {connected && battPct != null && <span className="sidebar-batt">{battPct}%</span>}
      </div>

      <div className="sidebar-footer">
        <img src={tuxUrl} alt="" aria-hidden="true" />
        <span>OpenSharkX3 <small>v{__APP_VERSION__}</small></span>
        <PlugZap size={13} />
      </div>
    </aside>
  )
}

export function AppHeader({ t, section, connected, connLabel, theme, onToggleTheme, onWin }) {
  return (
    <header className="app-header" data-tauri-drag-region>
      <div className="header-copy">
        <span className="eyebrow">{t('app.tagline')}</span>
        <h1>{t('sec.' + section + '.h')}</h1>
        <p>{t('sec.' + section + '.d')}</p>
      </div>
      <div className="header-actions">
        <ConnectionBadge connected={connected} connLabel={connLabel} t={t} />
        <button type="button" className="icon-btn" title={t('tb.theme')} aria-label={t('tb.theme')} onClick={onToggleTheme}>
          {theme === 'dark' ? <Sun size={16} /> : <Moon size={16} />}
        </button>
        <div className="window-controls">
          <button type="button" className="win-btn" title={t('tb.min')} aria-label={t('tb.min')} onClick={() => onWin('min')}><Minus size={14} /></button>
          <button type="button" className="win-btn" title={t('tb.max')} aria-label={t('tb.max')} onClick={() => onWin('max')}><Square size={11} /></button>
          <button type="button" className="win-btn close" title={t('tb.close')} aria-label={t('tb.close')} onClick={() => onWin('close')}><X size={14} /></button>
        </div>
      </div>
    </header>
  )
}

export function StatusRail({
  t, connected, connLabel, state, deviceStatus, battPct, battUpdatedAt,
  lastMeasure, measuring, measurePolling, receipts,
}) {
  const usb = state.conn === 'usb'
  const effectivePolling = (state.conn === 'usb' || state.conn === 'bluetooth') ? 125 : state.polling
  const latest = receipts && receipts.length ? receipts[receipts.length - 1] : null
  return (
    <aside className="status-rail">
      <div className="rail-heading">
        <span>{t('app.status')}</span>
        {connected && <span className="live-pill"><i />{t('dash.live')}</span>}
      </div>

      <section className="rail-card">
        <div className="rail-card-head">
          <span>{t('app.device')}</span>
          <ConnectionBadge connected={connected} connLabel={connLabel} t={t} compact />
        </div>
        <div className="rail-rows">
          <div><span>{t('dash.transport')}</span><strong>{connected ? connLabel : '—'}</strong></div>
          <div><span>{t('dash.writeMode')}</span><strong>{deviceStatus?.writeMode || '—'}</strong></div>
          <div><span>{t('con.events')}</span><strong>{deviceStatus?.eventCount ?? '—'}</strong></div>
          <div><span>{t('dash.activeStage')}</span><strong>{state.activeStage + 1} · {state.dpi[state.activeStage]?.dpi?.toLocaleString()} DPI</strong></div>
        </div>
      </section>

      <BatteryCard pct={battPct} updatedAt={battUpdatedAt} usb={usb} t={t} compact />

      <section className="rail-card">
        <div className="rail-card-head">
          <span>{t('dash.polling')}</span>
          <Gauge size={14} />
        </div>
        <div className="rail-polling">
          <div><span>{t('dash.selected')}</span><strong>{effectivePolling} Hz</strong></div>
          <div><span>{t('dash.measured')}</span><strong>{lastMeasure?.measuredHz != null ? lastMeasure.measuredHz + ' Hz' : '—'}</strong></div>
        </div>
        <button type="button" className="btn sm full" onClick={measurePolling} disabled={measuring}>
          {measuring ? t('app.measuring') : t('app.measure')}
        </button>
        <p className="rail-hint">{t('dash.measureHint')}</p>
      </section>

      <section className="rail-card rail-activity">
        <div className="rail-card-head">
          <span>{t('app.activity')}</span>
          <Activity size={14} />
        </div>
        <ActivityLog entries={state.log} t={t} limit={6} />
      </section>

      {latest && (
        <section className="rail-card">
          <div className="rail-card-head"><span>{t('con.lastwrite') || 'Last write'}</span></div>
          <div className="rail-last-write">
            <span>{latest.label}</span>
            <ProvenanceBadge tier={receiptTier(latest)} t={t} />
          </div>
        </section>
      )}
    </aside>
  )
}

export function formatTime(ts) {
  if (!ts) return '—'
  try {
    return new Date(ts).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
  } catch (_) {
    return '—'
  }
}
