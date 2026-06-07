import {
  createClient as createGeneratedAiotAppClient,
  SdkworkAppClient,
} from "../generated/server-openapi/src/index.js";
import type { SdkworkAppConfig } from "../generated/server-openapi/src/types/common.js";

export { SdkworkAppClient, createGeneratedAiotAppClient };
export * from "../generated/server-openapi/src/types/index.js";
export * from "../generated/server-openapi/src/api/index.js";
export * from "../generated/server-openapi/src/http/index.js";
export * from "../generated/server-openapi/src/auth/index.js";
export type { SdkworkAppConfig } from "../generated/server-openapi/src/types/common.js";

export type SdkworkAiotAppClient = SdkworkAppClient;
export type SdkworkAiotAppClientConfig = SdkworkAppConfig;

export function createAiotAppClient(
  config: SdkworkAppConfig,
): SdkworkAiotAppClient {
  return createGeneratedAiotAppClient(config);
}

export function createClient(config: SdkworkAppConfig): SdkworkAiotAppClient {
  return createAiotAppClient(config);
}
