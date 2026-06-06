/**
 * Generated SDK placeholder.
 *
 * This package boundary is reserved for SDKWork OpenAPI generation from
 * ../openapi/sdkwork-aiot-app-sdk.openapi.json. Do not add handwritten
 * transport logic here; update OpenAPI and regenerate the SDK instead.
 */
export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail?: string;
  traceId?: string;
  code?: string;
  [key: string]: unknown;
}

export type SdkworkAiotAppProblem = ProblemDetails;

export const SDKWORK_AIOT_APP_PROBLEM_CODES = [
  "api.auth.missing_dual_token",
  "api.context.invalid_data_scope",
  "api.context.invalid_organization_id",
  "api.context.invalid_tenant_id",
  "api.context.invalid_user_id",
  "api.context.missing",
  "api.device.duplicate_device_id",
  "api.device.invalid_product_id",
  "api.device.not_found",
  "api.permission.denied",
  "api.request.body.required",
  "api.request.invalid_field",
  "api.request.invalid_json",
  "api.request.invalid_json_object",
  "api.route.unsupported",
  "api.storage.read_failed",
  "api.storage.read_write_failed",
  "api.storage.write_failed",
  "api.command.duplicate_command_id",
] as const;

export type SdkworkAiotAppProblemCode =
  (typeof SDKWORK_AIOT_APP_PROBLEM_CODES)[number];

export function isProblemDetails(value: unknown): value is ProblemDetails {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as { [key: string]: unknown };
  return (
    typeof candidate.type === "string" &&
    typeof candidate.title === "string" &&
    typeof candidate.status === "number"
  );
}

export function isSdkworkAiotAppProblemCode(
  value: unknown
): value is SdkworkAiotAppProblemCode {
  return (
    typeof value === "string" &&
    (SDKWORK_AIOT_APP_PROBLEM_CODES as readonly string[]).includes(value)
  );
}

const DEFAULT_PROBLEM_TYPE = "about:blank";
const DEFAULT_PROBLEM_TITLE = "Unknown error";
const DEFAULT_PROBLEM_STATUS = 500;

function normalizeProblemStatus(value: unknown): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return DEFAULT_PROBLEM_STATUS;
  }
  const status = Math.trunc(value);
  if (status < 100 || status > 599) {
    return DEFAULT_PROBLEM_STATUS;
  }
  return status;
}

export function normalizeProblemDetails(
  value: unknown,
  fallback: Partial<ProblemDetails> = {}
): ProblemDetails {
  const source: Record<string, unknown> =
    typeof value === "object" && value !== null
      ? (value as Record<string, unknown>)
      : {};
  const normalized: ProblemDetails = {
    type:
      typeof source.type === "string" && source.type.length > 0
        ? source.type
        : typeof fallback.type === "string" && fallback.type.length > 0
          ? fallback.type
          : DEFAULT_PROBLEM_TYPE,
    title:
      typeof source.title === "string" && source.title.length > 0
        ? source.title
        : typeof fallback.title === "string" && fallback.title.length > 0
          ? fallback.title
          : DEFAULT_PROBLEM_TITLE,
    status:
      source.status !== undefined
        ? normalizeProblemStatus(source.status)
        : normalizeProblemStatus(fallback.status),
  };

  const detail =
    typeof source.detail === "string"
      ? source.detail
      : typeof fallback.detail === "string"
        ? fallback.detail
        : undefined;
  if (detail !== undefined) {
    normalized.detail = detail;
  }

  const traceId =
    typeof source.traceId === "string"
      ? source.traceId
      : typeof fallback.traceId === "string"
        ? fallback.traceId
        : undefined;
  if (traceId !== undefined) {
    normalized.traceId = traceId;
  }

  const code =
    typeof source.code === "string"
      ? source.code
      : typeof fallback.code === "string"
        ? fallback.code
        : undefined;
  if (code !== undefined) {
    normalized.code = code;
  }

  for (const [key, fieldValue] of Object.entries(source)) {
    if (!(key in normalized)) {
      normalized[key] = fieldValue;
    }
  }
  return normalized;
}

export interface AiotDevice {
  id: string;
  tenantId: string;
  organizationId: string;
  deviceId: string;
  displayName: string;
  productId: string;
  clientId?: string;
  chipFamily?: string;
  avatar?: MediaResource;
  status: string;
  metadata?: unknown;
  lastSeenAt?: string;
}

export type MediaKind =
  | "image"
  | "video"
  | "audio"
  | "voice"
  | "document"
  | "archive"
  | "model"
  | "other";

export type MediaSource =
  | "object_storage"
  | "external_url"
  | "data_url"
  | "provider_asset"
  | "generated";

export interface MediaChecksum {
  algorithm: "sha256" | "md5" | "etag";
  value: string;
}

export interface MediaAccess {
  visibility: "private" | "tenant" | "organization" | "public" | "signed";
  expiresAt?: string;
}

export interface MediaAiProvenance {
  provenance?: "uploaded" | "generated" | "edited" | "imported";
  provider?: string;
  model?: string;
  promptId?: string;
  generationTaskId?: string;
  sourceMediaIds?: string[];
  seed?: string;
  moderationStatus?: "unknown" | "pending" | "approved" | "rejected" | "blocked";
  safetyLabels?: string[];
}

export interface MediaResource {
  id?: string;
  kind: MediaKind;
  source: MediaSource;
  url?: string;
  publicUrl?: string;
  uri?: string;
  objectBlobId?: string;
  bucketId?: string;
  objectKey?: string;
  objectVersion?: string;
  fileName?: string;
  mimeType?: string;
  sizeBytes?: string;
  checksum?: MediaChecksum;
  width?: number;
  height?: number;
  durationSeconds?: number;
  altText?: string;
  title?: string;
  access?: MediaAccess;
  ai?: MediaAiProvenance;
  metadata?: Record<string, unknown>;
}

export interface AiotDeviceListResponse {
  code: string;
  msg?: string;
  data: AiotDevice[];
}

export interface AiotDeviceResponse {
  code: string;
  msg?: string;
  data: AiotDevice;
}

export interface AiotCommandCreateRequest {
  capabilityName: string;
  commandName: string;
  payload: unknown;
  requestMediaResourceId?: string;
  requestObjectBlobId?: string;
  requestMedia?: MediaResource;
  sessionId?: string;
  traceId?: string;
  timeoutAt?: string;
  idempotencyKey?: string;
}

export interface AiotCommandResult {
  resultCode?: string;
  resultPayload?: unknown;
  resultMediaResourceId?: string;
  resultObjectBlobId?: string;
  resultMedia?: MediaResource;
  occurredAt?: string;
}

export interface AiotCommand {
  commandId: string;
  deviceId: string;
  sessionId?: string;
  capabilityName: string;
  commandName: string;
  requestPayload: unknown;
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

export interface AiotCommandResponse {
  code: string;
  msg?: string;
  data: AiotCommand;
}

export interface AiotTwinResponse {
  code: string;
  msg?: string;
  data: unknown;
}

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
  direction: "device_to_cloud" | "cloud_to_device";
  messageId?: string;
  correlationId?: string;
  traceId?: string;
  payloadHash?: string;
  mediaResourceId?: string;
  objectBlobId?: string;
  media?: MediaResource;
  payload: unknown;
  occurredAt: string;
}

export interface AiotEventListResponse {
  code: string;
  msg?: string;
  data: AiotEvent[];
}

export interface SdkworkAiotAppClient {
  iot: {
    devices: {
      list: () => Promise<AiotDeviceListResponse>;
      retrieve: (deviceId: string) => Promise<AiotDeviceResponse>;
      commands: {
        create: (
          deviceId: string,
          request: AiotCommandCreateRequest
        ) => Promise<AiotCommandResponse>;
      };
      twin: {
        retrieve: (deviceId: string) => Promise<AiotTwinResponse>;
      };
      events: {
        list: (deviceId: string) => Promise<AiotEventListResponse>;
      };
    };
  };
}
