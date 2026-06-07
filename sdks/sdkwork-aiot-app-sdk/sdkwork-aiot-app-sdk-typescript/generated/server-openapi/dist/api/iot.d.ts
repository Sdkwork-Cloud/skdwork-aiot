import type { HttpClient } from '../http/client';
import type { AiotCommandCreateRequest, AiotCommandResponse, AiotDeviceListResponse, AiotDeviceResponse, AiotEventListResponse, AiotTwinResponse } from '../types';
export interface IotDevicesEventsListParams {
    xSdkworkTenantId: string;
    xSdkworkOrganizationId: string;
    xSdkworkUserId?: string;
    xSdkworkDataScope?: string;
    xSdkworkPermissionScope: string;
}
export declare class IotDevicesEventsApi {
    private client;
    constructor(client: HttpClient);
    /** List device events */
    list(deviceId: string, params: IotDevicesEventsListParams): Promise<AiotEventListResponse>;
}
export interface IotDevicesTwinRetrieveParams {
    xSdkworkTenantId: string;
    xSdkworkOrganizationId: string;
    xSdkworkUserId?: string;
    xSdkworkDataScope?: string;
    xSdkworkPermissionScope: string;
}
export declare class IotDevicesTwinApi {
    private client;
    constructor(client: HttpClient);
    /** Retrieve device twin */
    retrieve(deviceId: string, params: IotDevicesTwinRetrieveParams): Promise<AiotTwinResponse>;
}
export interface IotDevicesCommandsCreateParams {
    xSdkworkTenantId: string;
    xSdkworkOrganizationId: string;
    xSdkworkUserId?: string;
    xSdkworkDataScope?: string;
    xSdkworkPermissionScope: string;
    idempotencyKey?: string;
}
export declare class IotDevicesCommandsApi {
    private client;
    constructor(client: HttpClient);
    /** Create a device command */
    create(deviceId: string, body: AiotCommandCreateRequest, params: IotDevicesCommandsCreateParams): Promise<AiotCommandResponse>;
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
export declare class IotDevicesApi {
    private client;
    readonly commands: IotDevicesCommandsApi;
    readonly twin: IotDevicesTwinApi;
    readonly events: IotDevicesEventsApi;
    constructor(client: HttpClient);
    /** List user-visible AIoT devices */
    list(params: IotDevicesListParams): Promise<AiotDeviceListResponse>;
    /** Retrieve one AIoT device */
    retrieve(deviceId: string, params: IotDevicesRetrieveParams): Promise<AiotDeviceResponse>;
}
export declare class IotApi {
    private client;
    readonly devices: IotDevicesApi;
    constructor(client: HttpClient);
}
export declare function createIotApi(client: HttpClient): IotApi;
//# sourceMappingURL=iot.d.ts.map