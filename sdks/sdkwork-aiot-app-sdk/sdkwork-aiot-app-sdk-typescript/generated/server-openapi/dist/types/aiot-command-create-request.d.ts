import type { JsonValue } from './json-value';
import type { MediaResource } from './media-resource';
export interface AiotCommandCreateRequest {
    capabilityName: string;
    commandName: string;
    payload: JsonValue;
    requestMediaResourceId?: string;
    requestObjectBlobId?: string;
    requestMedia?: MediaResource;
    sessionId?: string;
    traceId?: string;
    timeoutAt?: string;
    /** Legacy body fallback. Prefer Idempotency-Key request header. */
    idempotencyKey?: string;
}
//# sourceMappingURL=aiot-command-create-request.d.ts.map