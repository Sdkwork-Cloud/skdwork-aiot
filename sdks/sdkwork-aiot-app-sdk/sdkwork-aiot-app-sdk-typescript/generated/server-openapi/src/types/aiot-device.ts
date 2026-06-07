import type { JsonValue } from './json-value';
import type { MediaResource } from './media-resource';

export interface AiotDevice {
  /** Stable int64 identifier serialized as a string. */
  id: string;
  tenantId: string;
  organizationId: string;
  deviceId: string;
  displayName: string;
  /** Product int64 identifier serialized as a string. */
  productId: string;
  clientId?: string;
  chipFamily?: string;
  avatar?: MediaResource;
  status: string;
  metadata?: JsonValue;
  lastSeenAt?: string;
}
