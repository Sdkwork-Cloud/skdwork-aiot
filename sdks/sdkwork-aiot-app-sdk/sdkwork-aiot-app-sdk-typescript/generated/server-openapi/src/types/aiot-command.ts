import type { AiotCommandResult } from './aiot-command-result';
import type { JsonValue } from './json-value';
import type { MediaResource } from './media-resource';

export interface AiotCommand {
  commandId: string;
  deviceId: string;
  sessionId?: string;
  capabilityName: string;
  commandName: string;
  requestPayload: JsonValue;
  requestMediaResourceId?: string;
  requestObjectBlobId?: string;
  requestMedia?: MediaResource;
  status: string;
  traceId?: string;
  timeoutAt?: string;
  ackAt?: string;
  resultAt?: string;
  createdAt: string;
  result?: AiotCommandResult;
}
