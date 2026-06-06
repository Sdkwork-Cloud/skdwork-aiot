/**
 * Generated SDK placeholder.
 *
 * This package boundary is reserved for SDKWork OpenAPI generation from
 * ../openapi/sdkwork-aiot-backend-sdk.openapi.json. Do not add handwritten
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

export type SdkworkAiotBackendProblem = ProblemDetails;

export const SDKWORK_AIOT_BACKEND_PROBLEM_CODES = [
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
  "api.command.not_found",
  "api.capability_model.not_found",
  "api.device.credential.not_found",
  "api.device.session.not_found",
  "api.firmware.artifact.duplicate_id",
  "api.firmware.artifact.invalid_reference",
  "api.firmware.artifact.not_found",
  "api.firmware.rollout.duplicate_id",
  "api.firmware.rollout.not_found",
] as const;

export type SdkworkAiotBackendProblemCode =
  (typeof SDKWORK_AIOT_BACKEND_PROBLEM_CODES)[number];

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

export function isSdkworkAiotBackendProblemCode(
  value: unknown
): value is SdkworkAiotBackendProblemCode {
  return (
    typeof value === "string" &&
    (SDKWORK_AIOT_BACKEND_PROBLEM_CODES as readonly string[]).includes(value)
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

export interface StandardCollectionResponse<T = unknown> {
  code: string;
  msg?: string;
  data: T[];
}

export interface StandardResourceResponse<T = unknown> {
  code: string;
  msg?: string;
  data: T;
}

export interface AiotDeviceCreateRequest {
  deviceId: string;
  displayName: string;
  productId: string;
  clientId?: string;
  chipFamily?: string;
}

export interface AiotDeviceUpdateRequest {
  displayName?: string;
  status?: string;
  metadata?: unknown;
}

export interface AiotCredentialCreateRequest {
  credentialType:
    | "bearer_token"
    | "hmac"
    | "mtls_x509"
    | "hardware_attestation";
  expiresAt?: string;
}

export interface AiotDeviceCredential {
  credentialId: string;
  deviceId: string;
  credentialType: string;
  status: string;
  expiresAt?: string;
  createdAt: string;
  revokedAt?: string;
}

export interface AiotTwinUpdateRequest {
  desired?: Record<string, unknown>;
  reported?: Record<string, unknown>;
}

export interface AiotDeviceSession {
  sessionId: string;
  deviceId: string;
  status: string;
  connectedAt?: string;
  disconnectedAt?: string;
  transport: string;
}

export interface AiotDeviceCapability {
  capabilityName: string;
  capabilityKind: string;
  status: string;
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

export interface AiotCommandListResponse {
  code: string;
  msg?: string;
  data: AiotCommand[];
}

export interface AiotFirmwareArtifactCreateRequest {
  artifactKey: string;
  version: string;
  resource: MediaResource;
  sha256: string;
  signature?: string;
  targetChipFamily?: string;
  targetRuntimeProfile?: string;
}

export interface AiotFirmwareArtifactUpdateRequest {
  artifactKey?: string;
  version?: string;
  resource?: MediaResource;
  mediaResourceId?: string;
  objectBlobId?: string;
  sha256?: string;
  signature?: string;
  targetChipFamily?: string;
  targetRuntimeProfile?: string;
  status?: string;
}

export interface AiotFirmwareRolloutCreateRequest {
  artifactId: string;
  targetPolicy: unknown;
}

export interface AiotFirmwareRolloutUpdateRequest {
  targetPolicy?: unknown;
  status?: string;
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

export interface AiotFirmwareArtifact {
  artifactId: string;
  artifactKey: string;
  version: string;
  resource: MediaResource;
  mediaResourceId: string;
  objectBlobId?: string;
  sha256: string;
  signature?: string;
  targetChipFamily?: string;
  targetRuntimeProfile?: string;
  status: string;
}

export interface AiotFirmwareArtifactResponse {
  code: string;
  msg?: string;
  data: AiotFirmwareArtifact;
}

export interface AiotFirmwareRollout {
  rolloutId: string;
  artifactId: string;
  targetPolicy: unknown;
  status: string;
}

export interface AiotFirmwareRolloutResponse {
  code: string;
  msg?: string;
  data: AiotFirmwareRollout;
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

export interface AiotRuntimeCapacityPolicyResponse {
  code: string;
  msg?: string;
  data: {
    nodeId: string;
    maxConnectionsPerNode: string;
    maxSessionsPerTenant: string;
    maxInflightPerDevice: number;
    sessionLeaseTtlSeconds: number;
    sessionLeaseRenewSeconds: number;
    outboxMaxAttempts: number;
    deadLetterAfterAttempts: number;
    backpressure: {
      warnLag: string;
      rejectLag: string;
      deadLetterLag: string;
    };
    orderedDeviceCommands?: boolean;
    idempotentIngest?: boolean;
  };
}

export interface SdkworkAiotBackendClient {
  iot: {
    products: {
      list: () => Promise<StandardCollectionResponse>;
    };
    hardwareProfiles: {
      list: () => Promise<StandardCollectionResponse>;
    };
    protocolProfiles: {
      list: () => Promise<StandardCollectionResponse>;
    };
    capabilityModels: {
      retrieve: (capabilityModelId: string) => Promise<StandardResourceResponse>;
    };
    devices: {
      list: () => Promise<StandardCollectionResponse>;
      create: (
        request: AiotDeviceCreateRequest
      ) => Promise<StandardResourceResponse>;
      retrieve: (deviceId: string) => Promise<StandardResourceResponse>;
      update: (
        deviceId: string,
        request: AiotDeviceUpdateRequest
      ) => Promise<StandardResourceResponse>;
      delete: (deviceId: string) => Promise<void>;
      sessions: {
        list: (
          deviceId: string
        ) => Promise<StandardCollectionResponse<AiotDeviceSession>>;
        disconnect: (deviceId: string, sessionId: string) => Promise<void>;
      };
      capabilities: {
        list: (
          deviceId: string
        ) => Promise<StandardCollectionResponse<AiotDeviceCapability>>;
      };
      commands: {
        list: (deviceId: string) => Promise<AiotCommandListResponse>;
        cancel: (
          deviceId: string,
          commandId: string
        ) => Promise<StandardResourceResponse<AiotCommand>>;
      };
      twin: {
        retrieve: (deviceId: string) => Promise<StandardResourceResponse>;
        update: (
          deviceId: string,
          request: AiotTwinUpdateRequest
        ) => Promise<StandardResourceResponse>;
      };
      credentials: {
        list: (
          deviceId: string
        ) => Promise<StandardCollectionResponse<AiotDeviceCredential>>;
        retrieve: (
          deviceId: string,
          credentialId: string
        ) => Promise<StandardResourceResponse<AiotDeviceCredential>>;
        create: (
          deviceId: string,
          request: AiotCredentialCreateRequest
        ) => Promise<StandardResourceResponse>;
        revoke: (deviceId: string, credentialId: string) => Promise<void>;
      };
    };
    firmwareArtifacts: {
      list: () => Promise<StandardCollectionResponse<AiotFirmwareArtifact>>;
      create: (
        request: AiotFirmwareArtifactCreateRequest
      ) => Promise<AiotFirmwareArtifactResponse>;
      retrieve: (artifactId: string) => Promise<AiotFirmwareArtifactResponse>;
      update: (
        artifactId: string,
        request: AiotFirmwareArtifactUpdateRequest
      ) => Promise<AiotFirmwareArtifactResponse>;
      delete: (artifactId: string) => Promise<void>;
    };
    firmwareRollouts: {
      list: () => Promise<StandardCollectionResponse<AiotFirmwareRollout>>;
      create: (
        request: AiotFirmwareRolloutCreateRequest
      ) => Promise<AiotFirmwareRolloutResponse>;
      retrieve: (rolloutId: string) => Promise<AiotFirmwareRolloutResponse>;
      update: (
        rolloutId: string,
        request: AiotFirmwareRolloutUpdateRequest
      ) => Promise<AiotFirmwareRolloutResponse>;
      delete: (rolloutId: string) => Promise<void>;
    };
    events: {
      list: () => Promise<AiotEventListResponse>;
    };
    protocolAdapters: {
      list: () => Promise<StandardCollectionResponse>;
    };
    runtime: {
      capacity: {
        retrieve: () => Promise<AiotRuntimeCapacityPolicyResponse>;
      };
    };
  };
}
