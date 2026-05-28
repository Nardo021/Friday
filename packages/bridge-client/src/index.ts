import type { AgentEvent, FridaySession } from "@friday/agent-core";

export interface PendingApproval {
  approvalId: string;
  sessionId: string;
  command: string;
  risk: "low" | "medium" | "high";
}

export interface BridgeInfo {
  hostname: string;
  activeSessionCount: number;
  bridgePort: number;
}

export interface FridayBridgeClientOptions {
  baseUrl: string;
  token: string;
}

export class FridayBridgeClient {
  private baseUrl: string;
  private token: string;

  constructor(options: FridayBridgeClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/$/, "");
    this.token = options.token;
  }

  private headers(): HeadersInit {
    return {
      Authorization: `Bearer ${this.token}`,
      Accept: "application/json",
    };
  }

  async health(): Promise<{ ok: boolean; version: string }> {
    const res = await fetch(`${this.baseUrl}/health`);
    if (!res.ok) throw new Error(`Health check failed: ${res.status}`);
    return res.json() as Promise<{ ok: boolean; version: string }>;
  }

  async getInfo(): Promise<BridgeInfo> {
    const res = await fetch(`${this.baseUrl}/v1/info`, { headers: this.headers() });
    if (!res.ok) throw new Error(`Info failed: ${res.status}`);
    return res.json() as Promise<BridgeInfo>;
  }

  async listSessions(): Promise<FridaySession[]> {
    const res = await fetch(`${this.baseUrl}/v1/sessions`, { headers: this.headers() });
    if (!res.ok) throw new Error(`List sessions failed: ${res.status}`);
    return res.json() as Promise<FridaySession[]>;
  }

  async getSession(id: string): Promise<FridaySession> {
    const res = await fetch(`${this.baseUrl}/v1/sessions/${id}`, { headers: this.headers() });
    if (!res.ok) throw new Error(`Get session failed: ${res.status}`);
    return res.json() as Promise<FridaySession>;
  }

  async getEvents(sessionId: string, limit = 100): Promise<AgentEvent[]> {
    const res = await fetch(
      `${this.baseUrl}/v1/sessions/${sessionId}/events?limit=${limit}`,
      { headers: this.headers() },
    );
    if (!res.ok) throw new Error(`Get events failed: ${res.status}`);
    return res.json() as Promise<AgentEvent[]>;
  }

  async listPendingApprovals(): Promise<PendingApproval[]> {
    const res = await fetch(`${this.baseUrl}/v1/approvals/pending`, {
      headers: this.headers(),
    });
    if (!res.ok) throw new Error(`List approvals failed: ${res.status}`);
    return res.json() as Promise<PendingApproval[]>;
  }

  async approve(approvalId: string): Promise<void> {
    const res = await fetch(`${this.baseUrl}/v1/approvals/${approvalId}/approve`, {
      method: "POST",
      headers: this.headers(),
    });
    if (!res.ok) throw new Error(`Approve failed: ${res.status}`);
  }

  async reject(approvalId: string): Promise<void> {
    const res = await fetch(`${this.baseUrl}/v1/approvals/${approvalId}/reject`, {
      method: "POST",
      headers: this.headers(),
    });
    if (!res.ok) throw new Error(`Reject failed: ${res.status}`);
  }

  async stopSession(sessionId: string): Promise<void> {
    const res = await fetch(`${this.baseUrl}/v1/sessions/${sessionId}/stop`, {
      method: "POST",
      headers: this.headers(),
    });
    if (!res.ok) throw new Error(`Stop session failed: ${res.status}`);
  }

  connectWebSocket(onEvent: (event: AgentEvent) => void): WebSocket {
    const wsBase = this.baseUrl.replace(/^http/, "ws");
    const ws = new WebSocket(`${wsBase}/v1/ws?token=${encodeURIComponent(this.token)}`);
    ws.onmessage = (msg) => {
      try {
        const event = JSON.parse(String(msg.data)) as AgentEvent;
        onEvent(event);
      } catch {
        // ignore malformed payloads
      }
    };
    return ws;
  }
}

export { FridayBridgeClient as default };
