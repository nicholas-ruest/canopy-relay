export type Health = 'healthy' | 'watch' | 'critical';
export type Severity = 'low' | 'medium' | 'high' | 'critical';

export type HabitatZone = {
  id: string;
  name: string;
  region: string;
  health: Health;
  canopy_cover: number;
  moisture: number;
  anomaly_count: number;
  coordinates: { x: number; y: number };
};

export type Sensor = {
  id: string;
  label: string;
  modality: string;
  zone_id: string;
  trust_score: number;
  calibration_status: 'verified' | 'drift' | 'quarantined';
  signature_verified: boolean;
  last_seen: string;
  value: number;
  unit: string;
};

export type Anomaly = {
  id: string;
  title: string;
  zone_id: string;
  severity: Severity;
  confidence: number;
  status: 'open' | 'reviewing' | 'resolved';
  detected_at: string;
  evidence: string[];
};

export type Intervention = {
  id: string;
  anomaly_id: string;
  action: string;
  rationale: string;
  risk_level: 'low' | 'guarded' | 'restricted';
  status: 'proposed' | 'authorized' | 'denied' | 'executed';
  policy_reason: string;
  created_at: string;
  correlation_id: string;
};

export type MeshLink = {
  id: string;
  from: string;
  to: string;
  state: 'online' | 'delayed' | 'offline';
  latency_ms: number;
  queue_depth: number;
  transport: string;
  last_sync: string;
};

export type AuditEntry = {
  id: string;
  event_type: string;
  summary: string;
  actor: string;
  timestamp: string;
  hash: string;
  correlation_id: string;
};

export type UpstreamStatus = {
  name: string;
  status: 'ready' | 'degraded' | 'offline';
  last_operation: string;
};

export type Dashboard = {
  project_name: string;
  generated_at: string;
  readiness: 'ready' | 'degraded';
  correlation_id: string;
  summary: {
    habitats_online: number;
    trusted_sensors: number;
    open_anomalies: number;
    pending_interventions: number;
  };
  habitat_zones: HabitatZone[];
  sensors: Sensor[];
  anomalies: Anomaly[];
  interventions: Intervention[];
  mesh_links: MeshLink[];
  audit_entries: AuditEntry[];
  upstreams: UpstreamStatus[];
};

export type ObservationRequest = {
  zone_id: string;
  sensor_id: string;
  signal_kind: string;
  value: number;
  unit: string;
};

export type ObservationResult = {
  observation_id: string;
  disposition: 'nominal' | 'anomaly';
  anomaly: Anomaly | null;
  correlation_id: string;
  upstream_execution: string[];
};

export type InterventionRequest = {
  anomaly_id: string;
  action: string;
  rationale: string;
  risk_level: Intervention['risk_level'];
};

export const API_PATHS = {
  dashboard: '/api/v1/dashboard',
  observations: '/api/v1/observations',
  interventions: '/api/v1/interventions',
  audit: '/api/v1/audit',
} as const;

const API = import.meta.env.VITE_API_URL ?? 'http://127.0.0.1:18080';

const headers = (token: string) => ({
  Authorization: `Bearer ${token}`,
  'Content-Type': 'application/json',
  'X-Correlation-Id': crypto.randomUUID(),
});

async function decode<T>(response: Response): Promise<T> {
  const body = (await response.json()) as T | { data?: T; code?: string; message?: string };
  if (!response.ok) {
    const problem = body as { code?: string; message?: string };
    throw new Error(problem.message ?? problem.code ?? `Request failed (${response.status})`);
  }
  return (body as { data?: T }).data ?? (body as T);
}

async function request<T>(path: string, token: string, init?: RequestInit): Promise<T> {
  return decode<T>(
    await fetch(`${API}${path}`, {
      ...init,
      headers: { ...headers(token), ...init?.headers },
    }),
  );
}

export function getDashboard(token: string): Promise<Dashboard> {
  return request(API_PATHS.dashboard, token);
}

export function submitObservation(token: string, input: ObservationRequest): Promise<ObservationResult> {
  const trustReceipt = crypto.randomUUID().replaceAll('-', '').repeat(2);
  return request(API_PATHS.observations, token, {
    method: 'POST',
    headers: { 'X-Device-Trust-Receipt': trustReceipt },
    body: JSON.stringify(input),
  });
}

export function createIntervention(token: string, input: InterventionRequest): Promise<Intervention> {
  return request(API_PATHS.interventions, token, {
    method: 'POST',
    body: JSON.stringify(input),
  });
}

export function listInterventions(token: string): Promise<Intervention[]> {
  return request(API_PATHS.interventions, token);
}

export function listAudit(token: string): Promise<AuditEntry[]> {
  return request(API_PATHS.audit, token);
}

export function getAuditEntry(token: string, id: string): Promise<AuditEntry> {
  return request(`${API_PATHS.audit}/${encodeURIComponent(id)}`, token);
}
