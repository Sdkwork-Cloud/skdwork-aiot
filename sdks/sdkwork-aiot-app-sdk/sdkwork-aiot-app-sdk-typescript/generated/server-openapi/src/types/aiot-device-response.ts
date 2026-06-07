import type { AiotDevice } from './aiot-device';

export interface AiotDeviceResponse {
  code: string;
  msg?: string;
  data: AiotDevice;
}
