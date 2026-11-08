import { type CSSProperties, type FormEvent, useMemo, useState } from 'react';
import {
  createIntervention,
  getDashboard,
  submitObservation,
  type Dashboard,
  type Intervention,
  type ObservationResult,
} from './client';

type View = 'habitat' | 'trust' | 'anomalies' | 'interventions' | 'mesh' | 'audit';

const views: { id: View; label: string; eyebrow: string }[] = [
  { id: 'habitat', label: 'Habitat Map', eyebrow: 'Living system' },
  { id: 'trust', label: 'Sensor Trust', eyebrow: 'Evidence quality' },
  { id: 'anomalies', label: 'Anomaly Workbench', eyebrow: 'Signal review' },
  { id: 'interventions', label: 'Intervention Planner', eyebrow: 'Governed action' },
  { id: 'mesh', label: 'Offline Mesh', eyebrow: 'Field relay' },
  { id: 'audit', label: 'Audit Vault', eyebrow: 'Durable proof' },
];

const formatTime = (value: string) => {
  const date = new Date(value);
  return Number.isNaN(date.valueOf()) ? value : date.toLocaleString([], { dateStyle: 'medium', timeStyle: 'short' });
};

const percent = (value: number) => `${Math.round(value * 100)}%`;

export function App() {
  const [view, setView] = useState<View>('habitat');
  const [token, setToken] = useState('');
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [selectedZone, setSelectedZone] = useState('');
  const [pending, setPending] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [observation, setObservation] = useState<ObservationResult | null>(null);
  const [observationZone, setObservationZone] = useState('');
  const [observationSensor, setObservationSensor] = useState('');
  const [signalKind, setSignalKind] = useState('canopy-moisture');
  const [signalValue, setSignalValue] = useState('0.18');
  const [riskLevel, setRiskLevel] = useState<Intervention['risk_level']>('guarded');
  const [action, setAction] = useState('deploy-mesh-relay');
  const [rationale, setRationale] = useState('Restore observability while preserving the habitat exclusion boundary.');
  const [createdIntervention, setCreatedIntervention] = useState<Intervention | null>(null);

  const activeView = views.find((item) => item.id === view)!;
  const currentZoneId = selectedZone || dashboard?.habitat_zones[0]?.id || '';
  const currentZone = dashboard?.habitat_zones.find((zone) => zone.id === currentZoneId);
  const zoneSensors = dashboard?.sensors.filter((sensor) => sensor.zone_id === currentZoneId) ?? [];
  const currentObservationZone = observationZone || dashboard?.habitat_zones[0]?.id || '';
  const compatibleSensors = dashboard?.sensors.filter((sensor) => sensor.zone_id === currentObservationZone) ?? [];
  const currentObservationSensor = observationSensor || compatibleSensors[0]?.id || '';
  const openAnomalies = dashboard?.anomalies.filter((item) => item.status !== 'resolved') ?? [];
  const selectedAnomalyId = openAnomalies[0]?.id || '';
  const trustworthy = dashboard?.sensors.filter((sensor) => sensor.signature_verified && sensor.calibration_status === 'verified').length ?? 0;
  const trustAverage = useMemo(() => {
    if (!dashboard?.sensors.length) return 0;
    return dashboard.sensors.reduce((sum, sensor) => sum + sensor.trust_score, 0) / dashboard.sensors.length;
  }, [dashboard]);

  async function refresh(message = 'Live habitat state synchronized.') {
    setPending(true);
    setError('');
    setNotice('');
    try {
      const next = await getDashboard(token);
      setDashboard(next);
      setSelectedZone((current) => current || next.habitat_zones[0]?.id || '');
      setNotice(message);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setPending(false);
    }
  }

  async function analyze(event: FormEvent) {
    event.preventDefault();
    setPending(true);
    setError('');
    setNotice('');
    try {
      const result = await submitObservation(token, {
        zone_id: currentObservationZone,
        sensor_id: currentObservationSensor,
        signal_kind: signalKind,
        value: Number(signalValue),
        unit: signalKind === 'temperature' ? 'celsius' : 'ratio',
      });
      setObservation(result);
      await refresh(`Observation ${result.observation_id} analyzed through the live upstream pipeline.`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
      setPending(false);
    }
  }

  async function plan(event: FormEvent) {
    event.preventDefault();
    setPending(true);
    setError('');
    setNotice('');
    try {
      const result = await createIntervention(token, {
        anomaly_id: selectedAnomalyId,
        action,
        rationale,
        risk_level: riskLevel,
      });
      setCreatedIntervention(result);
      await refresh(`Intervention ${result.id} authorized and written to the durable audit vault.`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
      setPending(false);
    }
  }

  return (
    <div className="app-shell">
      <a className="skip-link" href="#workspace">Skip to workspace</a>
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">CR</span>
          <div><b>Canopy Relay</b><small>Habitat operations</small></div>
        </div>
        <nav aria-label="Canopy Relay views">
          {views.map((item, index) => (
            <button
              type="button"
              key={item.id}
              className={view === item.id ? 'nav-item active' : 'nav-item'}
              aria-current={view === item.id ? 'page' : undefined}
              onClick={() => setView(item.id)}
            >
              <span>{String(index + 1).padStart(2, '0')}</span>
              <span><b>{item.label}</b><small>{item.eyebrow}</small></span>
            </button>
          ))}
        </nav>
        <div className="sidebar-foot">
          <span className={dashboard?.readiness === 'ready' ? 'status-dot ready' : 'status-dot'} />
          <div><b>{dashboard ? dashboard.readiness : 'Disconnected'}</b><small>Source-bound field state</small></div>
        </div>
      </aside>

      <main id="workspace" className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">{activeView.eyebrow}</p>
            <h1>{activeView.label}</h1>
          </div>
          <form className="session-bar" onSubmit={(event) => { event.preventDefault(); void refresh(); }}>
            <label>
              <span>Operator session</span>
              <input
                aria-label="Session token"
                type="password"
                autoComplete="current-password"
                value={token}
                onChange={(event) => setToken(event.target.value)}
                placeholder="Bearer token"
                required
              />
            </label>
            <button type="submit" className="button primary" disabled={pending}>
              {pending ? 'Synchronizing...' : dashboard ? 'Refresh live state' : 'Load live habitat'}
            </button>
          </form>
        </header>

        <div className="announcements" aria-live="polite">
          {error && <div className="banner error" role="alert"><b>Request stopped.</b> {error}</div>}
          {notice && <div className="banner success" role="status"><b>Verified.</b> {notice}</div>}
        </div>

        {!dashboard && !pending && (
          <section className="welcome empty-state">
            <p className="eyebrow">Field link required</p>
            <h2>Bring the living system into view.</h2>
            <p>Connect an operator session to read signed sensor evidence, habitat state, offline relay queues, and durable intervention history from the Rust service.</p>
            <div className="welcome-proof">
              <span>Typed API</span><span>Policy before write</span><span>Restart-durable audit</span>
            </div>
          </section>
        )}

        {pending && !dashboard && <section className="loading-state" aria-label="Loading habitat state"><i /><i /><i /><p>Reconstructing habitat state from signed evidence...</p></section>}

        {dashboard && view === 'habitat' && (
          <section className="view-stack">
            <div className="metric-grid" aria-label="Habitat summary">
              <article><small>Habitats online</small><strong>{dashboard.summary.habitats_online}</strong><span>federated zones</span></article>
              <article><small>Trusted sensors</small><strong>{dashboard.summary.trusted_sensors}</strong><span>{dashboard.sensors.length} reporting</span></article>
              <article><small>Open anomalies</small><strong>{dashboard.summary.open_anomalies}</strong><span>need operator review</span></article>
              <article><small>Pending actions</small><strong>{dashboard.summary.pending_interventions}</strong><span>policy governed</span></article>
            </div>
            <div className="map-layout">
              <article className="panel habitat-map">
                <div className="panel-heading"><div><p className="eyebrow">Federated biome view</p><h2>Habitat pulse</h2></div><span className="updated">Updated {formatTime(dashboard.generated_at)}</span></div>
                <div className="map-canvas" aria-label="Habitat map">
                  <div className="contour contour-one" /><div className="contour contour-two" /><div className="river" />
                  {dashboard.habitat_zones.map((zone) => (
                    <button
                      type="button"
                      key={zone.id}
                      aria-label={`${zone.name}, ${zone.health}`}
                      className={`map-node ${zone.health} ${zone.id === currentZoneId ? 'selected' : ''}`}
                      style={{ left: `${zone.coordinates.x}%`, top: `${zone.coordinates.y}%` }}
                      onClick={() => setSelectedZone(zone.id)}
                    ><span />{zone.name}</button>
                  ))}
                  {!dashboard.habitat_zones.length && <p className="map-empty">No habitat zones have reported yet.</p>}
                </div>
              </article>
              <aside className="panel zone-detail">
                <p className="eyebrow">Selected habitat</p>
                {currentZone ? <>
                  <div className="detail-title"><h2>{currentZone.name}</h2><span className={`pill ${currentZone.health}`}>{currentZone.health}</span></div>
                  <p className="subtle">{currentZone.region}</p>
                  <div className="ring-row">
                    <div className="ring" style={{ '--value': `${currentZone.canopy_cover * 360}deg` } as CSSProperties}><b>{percent(currentZone.canopy_cover)}</b><small>canopy</small></div>
                    <div className="ring moisture" style={{ '--value': `${currentZone.moisture * 360}deg` } as CSSProperties}><b>{percent(currentZone.moisture)}</b><small>moisture</small></div>
                  </div>
                  <h3>Active instruments</h3>
                  <div className="instrument-list">
                    {zoneSensors.map((sensor) => <div key={sensor.id}><span className={`status-dot ${sensor.calibration_status === 'verified' ? 'ready' : ''}`} /><span><b>{sensor.label}</b><small>{sensor.modality}</small></span><strong>{sensor.value} {sensor.unit}</strong></div>)}
                    {!zoneSensors.length && <p className="inline-empty">No instruments are assigned to this zone.</p>}
                  </div>
                </> : <p className="inline-empty">Select a reporting habitat.</p>}
              </aside>
            </div>
            <article className="panel upstream-strip">
              <div><p className="eyebrow">Runtime composition</p><h2>Executed boundaries</h2></div>
              <div className="upstream-list">{dashboard.upstreams.map((upstream) => <div key={upstream.name}><span className={`status-dot ${upstream.status === 'ready' ? 'ready' : ''}`} /><span><b>{upstream.name}</b><small>{upstream.last_operation}</small></span></div>)}</div>
            </article>
          </section>
        )}

        {dashboard && view === 'trust' && (
          <section className="view-stack">
            <div className="trust-hero">
              <div className="trust-score"><span>Network trust</span><strong>{percent(trustAverage)}</strong><small>{trustworthy} cryptographically verified</small></div>
              <div><p className="eyebrow">Calibration lineage</p><h2>Evidence earns authority.</h2><p>Every reading is scored by provenance, calibration freshness, signature verification, and cross-sensor agreement before it can influence an intervention.</p></div>
            </div>
            <article className="panel table-panel">
              <div className="panel-heading"><div><p className="eyebrow">Sensor registry</p><h2>Trust and calibration</h2></div><span className="updated">{dashboard.sensors.length} instruments</span></div>
              {dashboard.sensors.length ? <div className="table-scroll"><table><thead><tr><th>Instrument</th><th>Modality</th><th>Trust</th><th>Calibration</th><th>Signature</th><th>Last evidence</th></tr></thead><tbody>{dashboard.sensors.map((sensor) => <tr key={sensor.id}><td><b>{sensor.label}</b><small>{sensor.zone_id}</small></td><td>{sensor.modality}</td><td><div className="trust-bar"><i style={{ width: percent(sensor.trust_score) }} /><span>{percent(sensor.trust_score)}</span></div></td><td><span className={`pill ${sensor.calibration_status}`}>{sensor.calibration_status}</span></td><td>{sensor.signature_verified ? 'Verified' : 'Unverified'}</td><td>{formatTime(sensor.last_seen)}</td></tr>)}</tbody></table></div> : <p className="inline-empty">No sensor evidence has arrived.</p>}
            </article>
          </section>
        )}

        {dashboard && view === 'anomalies' && (
          <section className="workbench-layout">
            <div className="view-stack">
              <article className="panel">
                <div className="panel-heading"><div><p className="eyebrow">Open signals</p><h2>Anomaly queue</h2></div><span className="updated">{openAnomalies.length} active</span></div>
                <div className="anomaly-list">
                  {openAnomalies.map((item) => <article key={item.id} className="anomaly-card"><div><span className={`pill ${item.severity}`}>{item.severity}</span><small>{formatTime(item.detected_at)}</small></div><h3>{item.title}</h3><p>{item.evidence.join(' · ')}</p><footer><span>{item.zone_id}</span><b>{percent(item.confidence)} confidence</b></footer></article>)}
                  {!openAnomalies.length && <div className="inline-empty"><b>No unresolved anomalies.</b><br />Submit a reading to exercise the analysis path.</div>}
                </div>
              </article>
            </div>
            <form className="panel action-form" onSubmit={analyze} aria-label="Analyze live reading">
              <p className="eyebrow">Live upstream exercise</p><h2>Analyze a field reading</h2>
              <p className="subtle">The reading passes through validation, sensor trust, temporal analysis, anomaly classification, persistence, and correlated audit.</p>
              <label>Habitat zone<select aria-label="Observation habitat" value={currentObservationZone} onChange={(event) => { setObservationZone(event.target.value); setObservationSensor(''); }}>{dashboard.habitat_zones.map((zone) => <option key={zone.id} value={zone.id}>{zone.name}</option>)}</select></label>
              <label>Signed instrument<select aria-label="Observation sensor" value={currentObservationSensor} onChange={(event) => setObservationSensor(event.target.value)}>{compatibleSensors.map((sensor) => <option key={sensor.id} value={sensor.id}>{sensor.label}</option>)}</select></label>
              <div className="field-pair"><label>Signal<select aria-label="Signal kind" value={signalKind} onChange={(event) => setSignalKind(event.target.value)}><option value="canopy-moisture">Canopy moisture</option><option value="temperature">Temperature</option><option value="acoustic-activity">Acoustic activity</option></select></label><label>Reading<input aria-label="Signal value" type="number" step="0.01" value={signalValue} onChange={(event) => setSignalValue(event.target.value)} required /></label></div>
              <button className="button primary full" disabled={pending || !currentObservationSensor}>Analyze live reading</button>
              {observation && <div className={`result-card ${observation.disposition}`} role="status"><span>{observation.disposition}</span><b>{observation.anomaly?.title ?? 'Reading remains inside calibrated bounds'}</b><small id="observation-id">{observation.observation_id}</small><div>{observation.upstream_execution.map((name) => <i key={name}>{name}</i>)}</div></div>}
            </form>
          </section>
        )}

        {dashboard && view === 'interventions' && (
          <section className="workbench-layout intervention-layout">
            <form className="panel action-form" onSubmit={plan} aria-label="Request governed intervention">
              <p className="eyebrow">Policy before effect</p><h2>Plan an intervention</h2>
              <label>Anomaly<select aria-label="Intervention anomaly" value={selectedAnomalyId} disabled={!selectedAnomalyId}>{openAnomalies.map((item) => <option key={item.id} value={item.id}>{item.title}</option>)}</select></label>
              <label>Response pattern<select aria-label="Intervention action" value={action} onChange={(event) => setAction(event.target.value)}><option value="deploy-mesh-relay">Deploy mesh relay</option><option value="increase-sampling">Increase sampling cadence</option><option value="dispatch-field-review">Dispatch field review</option><option value="open-firebreak">Open automated firebreak</option></select></label>
              <label>Risk authority<select aria-label="Risk authority" value={riskLevel} onChange={(event) => setRiskLevel(event.target.value as Intervention['risk_level'])}><option value="low">Low · observe only</option><option value="guarded">Guarded · reversible field action</option><option value="restricted">Restricted · protected habitat effect</option></select></label>
              <label>Operator rationale<textarea aria-label="Intervention rationale" value={rationale} onChange={(event) => setRationale(event.target.value)} minLength={24} required /></label>
              <div className="policy-note"><b>Fail-closed boundary</b><span>Restricted effects require an independent ecological approver. Denials happen before persistence or downstream execution.</span></div>
              <button className="button primary full" disabled={pending || !selectedAnomalyId}>Request intervention</button>
              {createdIntervention && <div className="result-card nominal" role="status"><span>{createdIntervention.status}</span><b>{createdIntervention.action}</b><small id="intervention-id">{createdIntervention.id}</small></div>}
            </form>
            <article className="panel">
              <div className="panel-heading"><div><p className="eyebrow">Decision history</p><h2>Intervention register</h2></div><span className="updated">{dashboard.interventions.length} durable</span></div>
              <div className="timeline">{dashboard.interventions.map((item) => <article key={item.id}><span className={`timeline-dot ${item.status}`} /><div><div><span className={`pill ${item.status}`}>{item.status}</span><small>{formatTime(item.created_at)}</small></div><h3>{item.action}</h3><p>{item.rationale}</p><footer><code>{item.id}</code><span>{item.policy_reason}</span></footer></div></article>)}{!dashboard.interventions.length && <p className="inline-empty">No interventions are recorded yet.</p>}</div>
            </article>
          </section>
        )}

        {dashboard && view === 'mesh' && (
          <section className="view-stack">
            <article className="panel mesh-hero">
              <div><p className="eyebrow">Store-and-forward fabric</p><h2>The field keeps moving when the network does not.</h2><p>Signed semantic envelopes queue at the edge, reject replays, and reconcile when a trusted path returns.</p></div>
              <div className="mesh-orbit" aria-hidden="true"><i /><i /><i /><span>relay</span></div>
            </article>
            <div className="mesh-grid">{dashboard.mesh_links.map((link) => <article className="panel mesh-card" key={link.id}><header><span className={`status-dot ${link.state === 'online' ? 'ready' : ''}`} /><span className={`pill ${link.state}`}>{link.state}</span></header><h3>{link.from} <span>→</span> {link.to}</h3><dl><div><dt>Transport</dt><dd>{link.transport}</dd></div><div><dt>Latency</dt><dd>{link.latency_ms} ms</dd></div><div><dt>Queued</dt><dd>{link.queue_depth}</dd></div><div><dt>Last sync</dt><dd>{formatTime(link.last_sync)}</dd></div></dl></article>)}{!dashboard.mesh_links.length && <article className="panel inline-empty">No mesh links are registered.</article>}</div>
          </section>
        )}

        {dashboard && view === 'audit' && (
          <section className="view-stack">
            <article className="vault-hero"><div><p className="eyebrow">Restart-durable evidence</p><h2>Audit Vault</h2><p>Read-back receipts bind operator intent, policy decisions, upstream execution, and persisted outcomes to one correlation chain.</p></div><button className="button secondary" type="button" onClick={() => void refresh('Durable audit read back from the backend.')}>Read back vault</button></article>
            <article className="panel audit-panel">
              <div className="panel-heading"><div><p className="eyebrow">Hash-chained events</p><h2>Evidence ledger</h2></div><span className="updated">{dashboard.audit_entries.length} receipts</span></div>
              <div className="audit-list">{dashboard.audit_entries.map((entry) => <article key={entry.id}><time dateTime={entry.timestamp}>{formatTime(entry.timestamp)}</time><span className="audit-line" /><div><span className="pill neutral">{entry.event_type}</span><h3>{entry.summary}</h3><p>{entry.actor} · correlation {entry.correlation_id}</p><code>{entry.hash}</code><small>{entry.id}</small></div></article>)}{!dashboard.audit_entries.length && <p className="inline-empty">The vault is empty. Analyze a reading or authorize an intervention to create evidence.</p>}</div>
            </article>
          </section>
        )}
      </main>
    </div>
  );
}
