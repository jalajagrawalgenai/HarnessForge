import axios from 'axios';

export interface HarnessRunResult {
    agent_id: string;
    success: boolean;
    observation_count: number;
    detection_count: number;
    intervention_count: number;
    health_score?: number;
}

export class ForgeClient {
    private baseUrl: string;
    private apiKey: string;

    constructor(baseUrl: string, apiKey: string) {
        this.baseUrl = baseUrl;
        this.apiKey = apiKey;
    }

    private headers() {
        return { 'Authorization': `Bearer ${this.apiKey}`, 'Content-Type': 'application/json' };
    }

    async runAgent(task: string, preset: string = 'solo'): Promise<HarnessRunResult> {
        const { data } = await axios.post(`${this.baseUrl}/v1/sessions`, {
            task,
            preset,
            agent_type: 'solo',
        }, { headers: this.headers() });
        return data;
    }

    async listSessions(): Promise<any[]> {
        const { data } = await axios.get(`${this.baseUrl}/v1/sessions`, { headers: this.headers() });
        return data;
    }

    async getSessionHealth(sessionId: string): Promise<any> {
        const { data } = await axios.get(`${this.baseUrl}/sessions/${sessionId}/health`, { headers: this.headers() });
        return data;
    }

    async replaySession(sessionId: string): Promise<any> {
        const { data } = await axios.get(`${this.baseUrl}/v1/sessions/${sessionId}/audit/replay`, { headers: this.headers() });
        return data;
    }

    async getDetectorEfficacy(): Promise<any> {
        const { data } = await axios.get(`${this.baseUrl}/v1/harness/detectors/efficacy`, { headers: this.headers() });
        return data;
    }

    dispose() {
        // Cleanup
    }
}
