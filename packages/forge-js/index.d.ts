declare module '@forgesdk/core' {
    export interface HarnessConfig {
        preset?: 'solo' | 'langgraph' | 'crewai' | 'autogen';
        observers?: string[];
        detectors?: string[];
        strategies?: string[];
        dryRun?: boolean;
        simulation?: boolean;
    }
    export interface SessionResult {
        sessionId: string;
        agentId: string;
        success: boolean;
        observationCount: number;
        detectionCount: number;
        interventionCount: number;
    }
    export interface HealthScore {
        overall: number;
        dimensions: Record<string, number>;
    }
    export function createHarness(config?: HarnessConfig): Harness;
    export class Harness {
        run(task: string): Promise<SessionResult>;
        getHealth(): Promise<HealthScore>;
    }
    export function listDetectors(): string[];
    export function listStrategies(): string[];
    export function getVersion(): string;
}
