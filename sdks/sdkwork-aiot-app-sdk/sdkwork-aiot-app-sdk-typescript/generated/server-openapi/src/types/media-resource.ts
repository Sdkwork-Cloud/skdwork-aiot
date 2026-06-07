import type { MediaAccess } from './media-access';
import type { MediaAiProvenance } from './media-ai-provenance';
import type { MediaChecksum } from './media-checksum';
import type { MediaKind } from './media-kind';
import type { MediaSource } from './media-source';

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
