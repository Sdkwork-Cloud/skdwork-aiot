import type { JsonValue } from './json-value';
import type { MediaResource } from './media-resource';
export interface AiotCommandResult {
    resultCode?: string;
    resultPayload?: JsonValue;
    resultMediaResourceId?: string;
    resultObjectBlobId?: string;
    resultMedia?: MediaResource;
    occurredAt?: string;
}
//# sourceMappingURL=aiot-command-result.d.ts.map