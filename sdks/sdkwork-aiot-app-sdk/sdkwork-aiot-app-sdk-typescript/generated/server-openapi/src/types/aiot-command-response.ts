import type { AiotCommand } from './aiot-command';

export interface AiotCommandResponse {
  code: string;
  msg?: string;
  data: AiotCommand;
}
