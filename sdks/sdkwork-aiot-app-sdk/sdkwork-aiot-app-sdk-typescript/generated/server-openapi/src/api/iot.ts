import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { AiotCommandCreateRequest, AiotCommandResponse, AiotDeviceListResponse, AiotDeviceResponse, AiotEventListResponse, AiotTwinResponse } from '../types';


export interface IotDevicesEventsListParams {
  xSdkworkTenantId: string;
  xSdkworkOrganizationId: string;
  xSdkworkUserId?: string;
  xSdkworkDataScope?: string;
  xSdkworkPermissionScope: string;
}

export class IotDevicesEventsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List device events */
  async list(deviceId: string, params: IotDevicesEventsListParams): Promise<AiotEventListResponse> {
    const requestHeaders = buildRequestHeaders(
      {
        'X-Sdkwork-Tenant-Id': { value: params.xSdkworkTenantId, style: 'simple', explode: false },
        'X-Sdkwork-Organization-Id': { value: params.xSdkworkOrganizationId, style: 'simple', explode: false },
        'X-Sdkwork-User-Id': { value: params.xSdkworkUserId, style: 'simple', explode: false },
        'X-Sdkwork-Data-Scope': { value: params.xSdkworkDataScope, style: 'simple', explode: false },
        'X-Sdkwork-Permission-Scope': { value: params.xSdkworkPermissionScope, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.get<AiotEventListResponse>(appApiPath(`/iot/devices/${serializePathParameter(deviceId, { name: 'deviceId', style: 'simple', explode: false })}/events`), undefined, requestHeaders);
  }
}

export interface IotDevicesTwinRetrieveParams {
  xSdkworkTenantId: string;
  xSdkworkOrganizationId: string;
  xSdkworkUserId?: string;
  xSdkworkDataScope?: string;
  xSdkworkPermissionScope: string;
}

export class IotDevicesTwinApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve device twin */
  async retrieve(deviceId: string, params: IotDevicesTwinRetrieveParams): Promise<AiotTwinResponse> {
    const requestHeaders = buildRequestHeaders(
      {
        'X-Sdkwork-Tenant-Id': { value: params.xSdkworkTenantId, style: 'simple', explode: false },
        'X-Sdkwork-Organization-Id': { value: params.xSdkworkOrganizationId, style: 'simple', explode: false },
        'X-Sdkwork-User-Id': { value: params.xSdkworkUserId, style: 'simple', explode: false },
        'X-Sdkwork-Data-Scope': { value: params.xSdkworkDataScope, style: 'simple', explode: false },
        'X-Sdkwork-Permission-Scope': { value: params.xSdkworkPermissionScope, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.get<AiotTwinResponse>(appApiPath(`/iot/devices/${serializePathParameter(deviceId, { name: 'deviceId', style: 'simple', explode: false })}/twin`), undefined, requestHeaders);
  }
}

export interface IotDevicesCommandsCreateParams {
  xSdkworkTenantId: string;
  xSdkworkOrganizationId: string;
  xSdkworkUserId?: string;
  xSdkworkDataScope?: string;
  xSdkworkPermissionScope: string;
  idempotencyKey?: string;
}

export class IotDevicesCommandsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a device command */
  async create(deviceId: string, body: AiotCommandCreateRequest, params: IotDevicesCommandsCreateParams): Promise<AiotCommandResponse> {
    const requestHeaders = buildRequestHeaders(
      {
        'X-Sdkwork-Tenant-Id': { value: params.xSdkworkTenantId, style: 'simple', explode: false },
        'X-Sdkwork-Organization-Id': { value: params.xSdkworkOrganizationId, style: 'simple', explode: false },
        'X-Sdkwork-User-Id': { value: params.xSdkworkUserId, style: 'simple', explode: false },
        'X-Sdkwork-Data-Scope': { value: params.xSdkworkDataScope, style: 'simple', explode: false },
        'X-Sdkwork-Permission-Scope': { value: params.xSdkworkPermissionScope, style: 'simple', explode: false },
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<AiotCommandResponse>(appApiPath(`/iot/devices/${serializePathParameter(deviceId, { name: 'deviceId', style: 'simple', explode: false })}/commands`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface IotDevicesListParams {
  xSdkworkTenantId: string;
  xSdkworkOrganizationId: string;
  xSdkworkUserId?: string;
  xSdkworkDataScope?: string;
  xSdkworkPermissionScope: string;
}

export interface IotDevicesRetrieveParams {
  xSdkworkTenantId: string;
  xSdkworkOrganizationId: string;
  xSdkworkUserId?: string;
  xSdkworkDataScope?: string;
  xSdkworkPermissionScope: string;
}

export class IotDevicesApi {
  private client: HttpClient;
  public readonly commands: IotDevicesCommandsApi;
  public readonly twin: IotDevicesTwinApi;
  public readonly events: IotDevicesEventsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.commands = new IotDevicesCommandsApi(client);
    this.twin = new IotDevicesTwinApi(client);
    this.events = new IotDevicesEventsApi(client);
  }


/** List user-visible AIoT devices */
  async list(params: IotDevicesListParams): Promise<AiotDeviceListResponse> {
    const requestHeaders = buildRequestHeaders(
      {
        'X-Sdkwork-Tenant-Id': { value: params.xSdkworkTenantId, style: 'simple', explode: false },
        'X-Sdkwork-Organization-Id': { value: params.xSdkworkOrganizationId, style: 'simple', explode: false },
        'X-Sdkwork-User-Id': { value: params.xSdkworkUserId, style: 'simple', explode: false },
        'X-Sdkwork-Data-Scope': { value: params.xSdkworkDataScope, style: 'simple', explode: false },
        'X-Sdkwork-Permission-Scope': { value: params.xSdkworkPermissionScope, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.get<AiotDeviceListResponse>(appApiPath(`/iot/devices`), undefined, requestHeaders);
  }

/** Retrieve one AIoT device */
  async retrieve(deviceId: string, params: IotDevicesRetrieveParams): Promise<AiotDeviceResponse> {
    const requestHeaders = buildRequestHeaders(
      {
        'X-Sdkwork-Tenant-Id': { value: params.xSdkworkTenantId, style: 'simple', explode: false },
        'X-Sdkwork-Organization-Id': { value: params.xSdkworkOrganizationId, style: 'simple', explode: false },
        'X-Sdkwork-User-Id': { value: params.xSdkworkUserId, style: 'simple', explode: false },
        'X-Sdkwork-Data-Scope': { value: params.xSdkworkDataScope, style: 'simple', explode: false },
        'X-Sdkwork-Permission-Scope': { value: params.xSdkworkPermissionScope, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.get<AiotDeviceResponse>(appApiPath(`/iot/devices/${serializePathParameter(deviceId, { name: 'deviceId', style: 'simple', explode: false })}`), undefined, requestHeaders);
  }
}

export class IotApi {
  private client: HttpClient;
  public readonly devices: IotDevicesApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.devices = new IotDevicesApi(client);
  }

}

export function createIotApi(client: HttpClient): IotApi {
  return new IotApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}

interface PathParameterSpec {
  name: string;
  style: string;
  explode: boolean;
}

function serializePathParameter(value: unknown, spec: PathParameterSpec): string {
  if (value === undefined || value === null) {
    return '';
  }

  const style = spec.style || 'simple';
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === 'object') {
    return serializePathObject(spec.name, value as Record<string, unknown>, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}

function serializePathArray(name: string, values: unknown[], style: string, explode: boolean): string {
  const serialized = values
    .filter((item) => item !== undefined && item !== null)
    .map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === 'matrix') {
    return explode
      ? serialized.map((item) => `;${name}=${item}`).join('')
      : `;${name}=${serialized.join(',')}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? '.' : ',');
}

function serializePathObject(name: string, value: Record<string, unknown>, style: string, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === 'matrix') {
    return explode
      ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
      : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
  }
  const serialized = explode
    ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
    : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
  return pathPrefix(name, style, true) + serialized;
}

function pathPrefix(name: string, style: string, _objectValue: boolean): string {
  if (style === 'label') return '.';
  if (style === 'matrix') return `;${name}`;
  return '';
}

function encodePathValue(value: string): string {
  return encodeURIComponent(value);
}

function serializePathPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}

function buildRequestHeaders(
  headers: Record<string, HeaderParameterSpec | undefined>,
  cookies: Record<string, HeaderParameterSpec | undefined> = {},
): Record<string, string> | undefined {
  const requestHeaders: Record<string, string> = {};

  for (const [name, parameter] of Object.entries(headers)) {
    const serialized = serializeParameterValue(parameter);
    if (serialized !== undefined) {
      requestHeaders[name] = serialized;
    }
  }

  const cookieHeader = buildCookieHeader(cookies);
  if (cookieHeader) {
    requestHeaders.Cookie = requestHeaders.Cookie
      ? `${requestHeaders.Cookie}; ${cookieHeader}`
      : cookieHeader;
  }

  return Object.keys(requestHeaders).length > 0 ? requestHeaders : undefined;
}

interface HeaderParameterSpec {
  value: unknown;
  style: string;
  explode: boolean;
  contentType?: string;
}

function buildCookieHeader(cookies: Record<string, HeaderParameterSpec | undefined>): string | undefined {
  const pairs: string[] = [];
  for (const [name, parameter] of Object.entries(cookies)) {
    const serialized = serializeParameterValue(parameter);
    if (serialized !== undefined) {
      pairs.push(`${encodeURIComponent(name)}=${encodeURIComponent(serialized)}`);
    }
  }
  return pairs.length > 0 ? pairs.join('; ') : undefined;
}

function serializeParameterValue(parameter: HeaderParameterSpec | undefined): string | undefined {
  const value = parameter?.value;
  if (value === undefined || value === null) {
    return undefined;
  }
  if (parameter?.contentType) {
    return JSON.stringify(value);
  }
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (Array.isArray(value)) {
    return value.map((item) => serializeHeaderPrimitive(item)).join(',');
  }
  if (typeof value === 'object' && value !== null) {
    return serializeHeaderObject(value as Record<string, unknown>, parameter?.explode === true);
  }
  return serializeHeaderPrimitive(value);
}

function serializeHeaderObject(value: Record<string, unknown>, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (explode) {
    return entries.map(([key, entryValue]) => `${key}=${serializeHeaderPrimitive(entryValue)}`).join(',');
  }
  return entries.flatMap(([key, entryValue]) => [key, serializeHeaderPrimitive(entryValue)]).join(',');
}

function serializeHeaderPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  return String(value);
}
