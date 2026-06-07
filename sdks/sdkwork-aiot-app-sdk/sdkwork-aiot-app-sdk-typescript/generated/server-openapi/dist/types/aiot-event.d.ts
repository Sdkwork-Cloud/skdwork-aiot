import type { JsonValue } from './json-value';
import type { MediaResource } from './media-resource';
export interface AiotEvent {
    eventId: string;
    eventType: string;
    eventVersion: string;
    deviceId: string;
    protocolId: string;
    adapterId: string;
    messageClass: string;
    semanticType: string;
    transport: string;
    direction: 'device_to_cloud' | 'cloud_to_device';
    messageId?: string;
    correlationId?: string;
    traceId?: string;
    payloadHash?: string;
    mediaResourceId?: string;
    objectBlobId?: string;
    media?: MediaResource;
    payload: JsonValue;
    occurredAt: string;
}
//# sourceMappingURL=aiot-event.d.ts.map